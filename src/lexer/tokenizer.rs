use regex::Regex;
use super::token::{Token, TokenKind};

// ---------------------------------------------------------------------------
// Handler type
// ---------------------------------------------------------------------------

type Handler = fn(&mut Lexer, &Regex);

struct Pattern {
    regex:   Regex,
    handler: Handler,
}

// ---------------------------------------------------------------------------
// Lexer
// ---------------------------------------------------------------------------

pub struct Lexer {
    patterns:   Vec<Pattern>,
    pub tokens: Vec<Token>,
    source_str: String,
    pos:        usize,
    line:       usize,
}

impl Lexer {
    fn new(source: &str) -> Self {
        Self {
            patterns:   build_patterns(),
            tokens:     Vec::new(),
            source_str: source.to_owned(),
            pos:        0,
            line:       1,
        }
    }

    fn at_eof(&self) -> bool {
        self.pos >= self.source_str.len()
    }

    fn remainder(&self) -> &str {
        &self.source_str[self.pos..]
    }

    fn advance_n(&mut self, n: usize) {
        self.pos += n;
    }

    fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn tokenize(source: &str) -> Vec<Token> {
    let mut lex = Lexer::new(source);

    while !lex.at_eof() {
        let mut matched = false;

        for i in 0..lex.patterns.len() {
            let remainder = &lex.source_str[lex.pos..];

            if let Some(m) = lex.patterns[i].regex.find(remainder) {
                if m.start() == 0 {
                    let handler = lex.patterns[i].handler;
                    // SAFETY: the regex is never mutated; the pointer remains
                    // valid for the duration of the handler call.
                    let regex_ptr: *const Regex = &lex.patterns[i].regex;
                    handler(&mut lex, unsafe { &*regex_ptr });
                    matched = true;
                    break;
                }
            }
        }

        if !matched {
            panic!(
                "lexer error: unrecognized token near '{}'",
                lex.remainder().chars().take(20).collect::<String>()
            );
        }
    }

    lex.push(Token::new(TokenKind::Eof, "EOF"));
    lex.tokens
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

fn skip_handler(lex: &mut Lexer, regex: &Regex) {
    if let Some(m) = regex.find(lex.remainder()) {
        lex.advance_n(m.end());
    }
}

fn comment_handler(lex: &mut Lexer, regex: &Regex) {
    if let Some(m) = regex.find(lex.remainder()) {
        lex.advance_n(m.end());
        lex.line += 1;
    }
}

fn string_handler(lex: &mut Lexer, regex: &Regex) {
    if let Some(m) = regex.find(lex.remainder()) {
        let literal = m.as_str().to_owned();
        lex.push(Token::new(TokenKind::String, &literal));
        lex.advance_n(literal.len());
    }
}

fn number_handler(lex: &mut Lexer, regex: &Regex) {
    if let Some(m) = regex.find(lex.remainder()) {
        let num = m.as_str().to_owned();
        lex.push(Token::new(TokenKind::Number, &num));
        lex.advance_n(num.len());
    }
}

fn symbol_handler(lex: &mut Lexer, regex: &Regex) {
    if let Some(m) = regex.find(lex.remainder()) {
        let word = m.as_str().to_owned();
        let kind = TokenKind::from_keyword(&word).unwrap_or(TokenKind::Identifier);
        lex.push(Token::new(kind, &word));
        lex.advance_n(word.len());
    }
}

macro_rules! default_handler {
    ($name:ident, $kind:expr, $value:expr) => {
        fn $name(lex: &mut Lexer, _regex: &Regex) {
            lex.advance_n($value.len());
            lex.push(Token::new($kind, $value));
        }
    };
}

default_handler!(open_bracket_handler,       TokenKind::OpenBracket,       "[");
default_handler!(close_bracket_handler,      TokenKind::CloseBracket,      "]");
default_handler!(open_curly_handler,         TokenKind::OpenCurly,         "{");
default_handler!(close_curly_handler,        TokenKind::CloseCurly,        "}");
default_handler!(open_paren_handler,         TokenKind::OpenParen,         "(");
default_handler!(close_paren_handler,        TokenKind::CloseParen,        ")");
default_handler!(equals_handler,             TokenKind::Equals,            "==");
default_handler!(not_equals_handler,         TokenKind::NotEquals,         "!=");
default_handler!(assignment_handler,         TokenKind::Assignment,        "=");
default_handler!(not_handler,                TokenKind::Not,               "!");
default_handler!(less_equals_handler,        TokenKind::LessEquals,        "<=");
default_handler!(less_handler,               TokenKind::Less,              "<");
default_handler!(greater_equals_handler,     TokenKind::GreaterEquals,     ">=");
default_handler!(greater_handler,            TokenKind::Greater,           ">");
default_handler!(or_handler,                 TokenKind::Or,                "||");
default_handler!(and_handler,                TokenKind::And,               "&&");
default_handler!(dot_dot_handler,            TokenKind::DotDot,            "..");
default_handler!(dot_handler,                TokenKind::Dot,               ".");
default_handler!(semi_colon_handler,         TokenKind::SemiColon,         ";");
default_handler!(colon_handler,              TokenKind::Colon,             ":");
default_handler!(nullish_assignment_handler, TokenKind::NullishAssignment, "??=");
default_handler!(question_handler,           TokenKind::Question,          "?");
default_handler!(comma_handler,              TokenKind::Comma,             ",");
default_handler!(plus_plus_handler,          TokenKind::PlusPlus,          "++");
default_handler!(minus_minus_handler,        TokenKind::MinusMinus,        "--");
default_handler!(plus_equals_handler,        TokenKind::PlusEquals,        "+=");
default_handler!(minus_equals_handler,       TokenKind::MinusEquals,       "-=");
default_handler!(plus_handler,               TokenKind::Plus,              "+");
default_handler!(dash_handler,               TokenKind::Dash,              "-");
default_handler!(slash_handler,              TokenKind::Slash,             "/");
default_handler!(star_handler,               TokenKind::Star,              "*");
default_handler!(percent_handler,            TokenKind::Percent,           "%");

// ---------------------------------------------------------------------------
// Pattern table — order matters (longer / more-specific patterns first)
// ---------------------------------------------------------------------------

fn build_patterns() -> Vec<Pattern> {
    macro_rules! pat {
        ($re:expr, $h:expr) => {
            Pattern { regex: Regex::new($re).unwrap(), handler: $h }
        };
    }

    vec![
        pat!(r"^\s+",                     skip_handler),
        pat!(r"^//.*",                    comment_handler),
        pat!(r#"^"[^"]*""#,               string_handler),
        pat!(r"^[0-9]+(\.[0-9]+)?",       number_handler),
        pat!(r"^[a-zA-Z_][a-zA-Z0-9_]*",  symbol_handler),
        pat!(r"^\[",                      open_bracket_handler),
        pat!(r"^\]",                      close_bracket_handler),
        pat!(r"^\{",                      open_curly_handler),
        pat!(r"^\}",                      close_curly_handler),
        pat!(r"^\(",                      open_paren_handler),
        pat!(r"^\)",                      close_paren_handler),
        pat!(r"^==",                      equals_handler),
        pat!(r"^!=",                      not_equals_handler),
        pat!(r"^=",                       assignment_handler),
        pat!(r"^!",                       not_handler),
        pat!(r"^<=",                      less_equals_handler),
        pat!(r"^<",                       less_handler),
        pat!(r"^>=",                      greater_equals_handler),
        pat!(r"^>",                       greater_handler),
        pat!(r"^\|\|",                    or_handler),
        pat!(r"^&&",                      and_handler),
        pat!(r"^\.\.",                    dot_dot_handler),
        pat!(r"^\.",                      dot_handler),
        pat!(r"^;",                       semi_colon_handler),
        pat!(r"^:",                       colon_handler),
        pat!(r"^\?\?=",                   nullish_assignment_handler),
        pat!(r"^\?",                      question_handler),
        pat!(r"^,",                       comma_handler),
        pat!(r"^\+\+",                    plus_plus_handler),
        pat!(r"^--",                      minus_minus_handler),
        pat!(r"^\+=",                     plus_equals_handler),
        pat!(r"^-=",                      minus_equals_handler),
        pat!(r"^\+",                      plus_handler),
        pat!(r"^-",                       dash_handler),
        pat!(r"^/",                       slash_handler),
        pat!(r"^\*",                      star_handler),
        pat!(r"^%",                       percent_handler),
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_expression() {
        let tokens = tokenize("x = 1 + 2;");
        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(matches!(kinds[0], TokenKind::Identifier));
        assert!(matches!(kinds[1], TokenKind::Assignment));
        assert!(matches!(kinds[2], TokenKind::Number));
        assert!(matches!(kinds[3], TokenKind::Plus));
        assert!(matches!(kinds[4], TokenKind::Number));
        assert!(matches!(kinds[5], TokenKind::SemiColon));
        assert!(matches!(kinds[6], TokenKind::Eof));
    }

    #[test]
    fn test_string_literal() {
        let tokens = tokenize(r#""hello""#);
        assert!(matches!(tokens[0].kind, TokenKind::String));
        assert_eq!(tokens[0].value, r#""hello""#);
    }

    #[test]
    fn test_keywords() {
        let tokens = tokenize("let x = null;");
        assert!(matches!(tokens[0].kind, TokenKind::Let));
        assert!(matches!(tokens[1].kind, TokenKind::Identifier));
        assert!(matches!(tokens[2].kind, TokenKind::Assignment));
        assert!(matches!(tokens[3].kind, TokenKind::Null));
    }

    #[test]
    fn test_comment_skipped() {
        let tokens = tokenize("x // this is a comment\ny");
        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert_eq!(kinds.len(), 3);
        assert!(matches!(kinds[0], TokenKind::Identifier));
        assert!(matches!(kinds[1], TokenKind::Identifier));
        assert!(matches!(kinds[2], TokenKind::Eof));
    }

    #[test]
    fn test_float_number() {
        let tokens = tokenize("3.14");
        assert!(matches!(tokens[0].kind, TokenKind::Number));
        assert_eq!(tokens[0].value, "3.14");
    }

    #[test]
    fn test_compound_operators() {
        let src = "a++ b-- c += 1 d -= 2 e != f e == f";
        let tokens = tokenize(src);
        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(matches!(kinds[1], TokenKind::PlusPlus));
        assert!(matches!(kinds[3], TokenKind::MinusMinus));
        assert!(matches!(kinds[5], TokenKind::PlusEquals));
        assert!(matches!(kinds[8], TokenKind::MinusEquals));
        assert!(matches!(kinds[11], TokenKind::NotEquals));
        assert!(matches!(kinds[14], TokenKind::Equals));
    }
}
