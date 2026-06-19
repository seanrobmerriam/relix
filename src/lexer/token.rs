use std::fmt;

/// Enumerates every distinct lexical token the Relix lexer can produce.
///
/// `TokenKind` covers literals, identifiers, operators, punctuation, and
/// reserved keywords. Use [`TokenKind::from_keyword`] to resolve an identifier
/// string to its keyword variant (if any).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// End-of-file sentinel.
    Eof,

    // ── Literals ──────────────────────────────────────────────────────

    /// The `null` literal.
    Null,
    /// The `true` literal.
    True,
    /// The `false` literal.
    False,
    /// A numeric literal (integer or floating-point).
    Number,
    /// A double-quoted string literal.
    String,
    /// An identifier (variable name, function name, etc.).
    Identifier,

    // ── Grouping & braces ─────────────────────────────────────────────

    /// `[`
    OpenBracket,
    /// `]`
    CloseBracket,
    /// `{`
    OpenCurly,
    /// `}`
    CloseCurly,
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,

    // ── Equivalence ───────────────────────────────────────────────────

    /// `=`
    Assignment,
    /// `==`
    Equals,
    /// `!=`
    NotEquals,
    /// `!`
    Not,

    // ── Conditional / comparison ──────────────────────────────────────

    /// `<`
    Less,
    /// `<=`
    LessEquals,
    /// `>`
    Greater,
    /// `>=`
    GreaterEquals,

    // ── Logical ───────────────────────────────────────────────────────

    /// `||`
    Or,
    /// `&&`
    And,

    // ── Symbols / punctuation ─────────────────────────────────────────

    /// `.`
    Dot,
    /// `..`
    DotDot,
    /// `;`
    SemiColon,
    /// `:`
    Colon,
    /// `?`
    Question,
    /// `,`
    Comma,

    // ── Shorthand operators ───────────────────────────────────────────

    /// `++`
    PlusPlus,
    /// `--`
    MinusMinus,
    /// `+=`
    PlusEquals,
    /// `-=`
    MinusEquals,
    /// `??=`
    NullishAssignment,

    // ── Arithmetic ────────────────────────────────────────────────────

    /// `+`
    Plus,
    /// `-`
    Dash,
    /// `/`
    Slash,
    /// `*`
    Star,
    /// `%`
    Percent,

    // ── Reserved keywords ─────────────────────────────────────────────

    /// `let`
    Let,
    /// `const`
    Const,
    /// `class`
    Class,
    /// `new`
    New,
    /// `import`
    Import,
    /// `from`
    From,
    /// `fn`
    Fn,
    /// `if`
    If,
    /// `else`
    Else,
    /// `foreach`
    Foreach,
    /// `while`
    While,
    /// `for`
    For,
    /// `export`
    Export,
    /// `typeof`
    Typeof,
    /// `in`
    In,
    /// `return`
    Return,
}

impl TokenKind {
    /// Returns a static string representation of this token kind.
    ///
    /// The returned string is a snake_case label suitable for diagnostics and
    /// debug output.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Eof               => "eof",
            Self::Null              => "null",
            Self::True              => "true",
            Self::False             => "false",
            Self::Number            => "number",
            Self::String            => "string",
            Self::Identifier        => "identifier",
            Self::OpenBracket       => "open_bracket",
            Self::CloseBracket      => "close_bracket",
            Self::OpenCurly         => "open_curly",
            Self::CloseCurly        => "close_curly",
            Self::OpenParen         => "open_paren",
            Self::CloseParen        => "close_paren",
            Self::Assignment        => "assignment",
            Self::Equals            => "equals",
            Self::NotEquals         => "not_equals",
            Self::Not               => "not",
            Self::Less              => "less",
            Self::LessEquals        => "less_equals",
            Self::Greater           => "greater",
            Self::GreaterEquals     => "greater_equals",
            Self::Or                => "or",
            Self::And               => "and",
            Self::Dot               => "dot",
            Self::DotDot            => "dot_dot",
            Self::SemiColon         => "semi_colon",
            Self::Colon             => "colon",
            Self::Question          => "question",
            Self::Comma             => "comma",
            Self::PlusPlus          => "plus_plus",
            Self::MinusMinus        => "minus_minus",
            Self::PlusEquals        => "plus_equals",
            Self::MinusEquals       => "minus_equals",
            Self::NullishAssignment => "nullish_assignment",
            Self::Plus              => "plus",
            Self::Dash              => "dash",
            Self::Slash             => "slash",
            Self::Star              => "star",
            Self::Percent           => "percent",
            Self::Let               => "let",
            Self::Const             => "const",
            Self::Class             => "class",
            Self::New               => "new",
            Self::Import            => "import",
            Self::From              => "from",
            Self::Fn                => "fn",
            Self::If                => "if",
            Self::Else              => "else",
            Self::Foreach           => "foreach",
            Self::While             => "while",
            Self::For               => "for",
            Self::Export            => "export",
            Self::Typeof            => "typeof",
            Self::In                => "in",
            Self::Return            => "return",
        }
    }

    /// Resolves a raw identifier string to a reserved keyword token kind.
    ///
    /// Returns `None` if the string is not a keyword, meaning it should be
    /// treated as a plain [`Identifier`](TokenKind::Identifier).
    ///
    /// # Example
    ///
    /// ```
    /// use relix::lexer::TokenKind;
    ///
    /// assert_eq!(TokenKind::from_keyword("let"), Some(TokenKind::Let));
    /// assert_eq!(TokenKind::from_keyword("foo"), None);
    /// ```
    pub fn from_keyword(word: &str) -> Option<Self> {
        match word {
            "true"    => Some(Self::True),
            "false"   => Some(Self::False),
            "null"    => Some(Self::Null),
            "let"     => Some(Self::Let),
            "const"   => Some(Self::Const),
            "class"   => Some(Self::Class),
            "new"     => Some(Self::New),
            "import"  => Some(Self::Import),
            "from"    => Some(Self::From),
            "fn"      => Some(Self::Fn),
            "if"      => Some(Self::If),
            "else"    => Some(Self::Else),
            "foreach" => Some(Self::Foreach),
            "while"   => Some(Self::While),
            "for"     => Some(Self::For),
            "export"  => Some(Self::Export),
            "typeof"  => Some(Self::Typeof),
            "in"      => Some(Self::In),
            "return"  => Some(Self::Return),
            _         => None,
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single lexical token produced by the lexer.
///
/// Each token carries a [`TokenKind`] indicating what category of token it is,
/// and a [`String`](std::string::String) holding the original source text that
/// was matched.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    /// The category of this token.
    pub kind:  TokenKind,
    /// The raw source text that was matched for this token.
    pub value: String,
}

impl Token {
    /// Creates a new token with the given kind and value.
    pub fn new(kind: TokenKind, value: impl Into<String>) -> Self {
        Self { kind, value: value.into() }
    }

    /// Returns `true` if this token's kind matches any of the provided kinds.
    ///
    /// This is a convenience method for checking against multiple token kinds
    /// without writing a long chain of `||` comparisons.
    ///
    /// # Example
    ///
    /// ```
    /// use relix::lexer::{Token, TokenKind};
    ///
    /// let tok = Token::new(TokenKind::Plus, "+");
    /// assert!(tok.is_one_of_many(&[TokenKind::Plus, TokenKind::Dash]));
    /// ```
    pub fn is_one_of_many(&self, expected: &[TokenKind]) -> bool {
        expected.contains(&self.kind)
    }

    /// Prints a human-readable debug representation of the token to stdout.
    ///
    /// For tokens that carry meaningful values (identifiers, numbers, strings),
    /// the value is included in the output. For all other tokens, only the kind
    /// is printed.
    pub fn debug(&self) {
        match self.kind {
            TokenKind::Identifier | TokenKind::Number | TokenKind::String => {
                println!("{}({})", self.kind, self.value);
            }
            _ => {
                println!("{}()", self.kind);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_str_roundtrip() {
        assert_eq!(TokenKind::Eof.as_str(), "eof");
        assert_eq!(TokenKind::NullishAssignment.as_str(), "nullish_assignment");
        assert_eq!(TokenKind::Foreach.as_str(), "foreach");
    }

    #[test]
    fn test_keyword_lookup() {
        assert_eq!(TokenKind::from_keyword("let"),     Some(TokenKind::Let));
        assert_eq!(TokenKind::from_keyword("foreach"), Some(TokenKind::Foreach));
        assert_eq!(TokenKind::from_keyword("typeof"),  Some(TokenKind::Typeof));
        assert_eq!(TokenKind::from_keyword("unknown"), None);
    }

    #[test]
    fn test_is_one_of_many() {
        let tok = Token::new(TokenKind::Plus, "+");
        assert!(tok.is_one_of_many(&[TokenKind::Plus, TokenKind::Dash]));
        assert!(!tok.is_one_of_many(&[TokenKind::Star, TokenKind::Slash]));
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", TokenKind::OpenBracket), "open_bracket");
        assert_eq!(format!("{}", TokenKind::In), "in");
    }

    #[test]
    fn test_token_new() {
        let tok = Token::new(TokenKind::Identifier, "foo");
        assert_eq!(tok.kind, TokenKind::Identifier);
        assert_eq!(tok.value, "foo");
    }
}
