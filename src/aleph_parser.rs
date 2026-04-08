//! ℵ-OS λ_ℵ surface syntax tokenizer and parser.
//!
//! Parses the subset of the ALEPH grammar needed for the kernel REPL:
//!   expr ::= letter_id
//!          | expr "⊗" expr          tensor
//!          | expr "∨" expr          join
//!          | expr "∧" expr          meet
//!          | expr "::>" name        vav-cast
//!          | "probe_Φ" "(" expr ")" criticality probe
//!          | "probe_Ω" "(" expr ")" protection probe
//!          | "tier" "(" expr ")"    ouroboricity tier
//!          | "d" "(" expr "," expr ")"  structural distance
//!          | "mediate" "(" expr "," expr "," expr ")"
//!          | "system" "()"          22-letter JOIN
//!          | "census" "()"          tier distribution
//!          | "(" expr ")"
//!          | "let" name "=" expr    binding

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::string::ToString;
use alloc::boxed::Box;
use alloc::format;
use crate::aleph;

// ── Token ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Letter,       // Hebrew glyph
    Name,         // ASCII identifier
    Int,          // Integer literal
    Tensor,       // ⊗
    Join,         // ∨
    Meet,         // ∧
    Cast,         // ::>
    LParen,       // (
    RParen,       // )
    LBrace,       // {
    RBrace,       // }
    Comma,        // ,
    Eq,           // =
    FatArrow,     // =>
    Match,        // match keyword
    Let,          // let keyword
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub val: String,
    pub pos: usize,
}

// ── Tokenizer ─────────────────────────────────────────────────────────────────

pub fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = src.chars().collect();

    while i < chars.len() {
        let c = chars[i];

        // Whitespace
        if c == ' ' || c == '\t' || c == '\r' || c == '\n' {
            i += 1;
            continue;
        }

        // Comment
        if c == '#' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // ::>  (vav-cast)
        if i + 2 < chars.len() && chars[i] == ':' && chars[i+1] == ':' && chars[i+2] == '>' {
            tokens.push(Token { kind: TokenKind::Cast, val: "::>".into(), pos: i });
            i += 3;
            continue;
        }

        // Unicode operators
        if c == '⊗' {
            tokens.push(Token { kind: TokenKind::Tensor, val: "⊗".into(), pos: i });
            i += 1;
            continue;
        }
        if c == '∨' {
            tokens.push(Token { kind: TokenKind::Join, val: "∨".into(), pos: i });
            i += 1;
            continue;
        }
        if c == '∧' {
            tokens.push(Token { kind: TokenKind::Meet, val: "∧".into(), pos: i });
            i += 1;
            continue;
        }

        // =>  (fat arrow for match arms)
        if i + 1 < chars.len() && chars[i] == '=' && chars[i+1] == '>' {
            tokens.push(Token { kind: TokenKind::FatArrow, val: "=>".into(), pos: i });
            i += 2;
            continue;
        }

        // Punctuation
        match c {
            '(' => { tokens.push(Token { kind: TokenKind::LParen, val: "(".into(), pos: i }); i += 1; continue; }
            ')' => { tokens.push(Token { kind: TokenKind::RParen, val: ")".into(), pos: i }); i += 1; continue; }
            '{' => { tokens.push(Token { kind: TokenKind::LBrace, val: "{".into(), pos: i }); i += 1; continue; }
            '}' => { tokens.push(Token { kind: TokenKind::RBrace, val: "}".into(), pos: i }); i += 1; continue; }
            ',' => { tokens.push(Token { kind: TokenKind::Comma, val: ",".into(), pos: i }); i += 1; continue; }
            '=' => { tokens.push(Token { kind: TokenKind::Eq, val: "=".into(), pos: i }); i += 1; continue; }
            // ASCII alias for meet (∧) — parser's is_ascii_op path handles it as Name "^"
            '^' => { tokens.push(Token { kind: TokenKind::Name, val: "^".into(), pos: i }); i += 1; continue; }
            _ => {}
        }

        // Hebrew glyph
        if aleph::is_hebrew(c) {
            tokens.push(Token { kind: TokenKind::Letter, val: c.to_string(), pos: i });
            i += 1;
            continue;
        }

        // Integer
        if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let val: String = chars[start..i].iter().collect();
            tokens.push(Token { kind: TokenKind::Int, val, pos: start });
            continue;
        }

        // Identifier
        if c.is_ascii_alphabetic() || c == '_' || c == 'Φ' || c == 'Ω' || c == 'Γ' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_'
                || chars[i] == 'Φ' || chars[i] == 'Ω' || chars[i] == 'Γ')
            {
                i += 1;
            }
            let val: String = chars[start..i].iter().collect();

            // Check for keyword
            let kind = match val.as_str() {
                "let" => TokenKind::Let,
                "match" => TokenKind::Match,
                _ => TokenKind::Name,
            };
            tokens.push(Token { kind, val, pos: start });
            continue;
        }

        return Err(format!("Unexpected character '{}' at position {}", c, i));
    }

    tokens.push(Token { kind: TokenKind::Eof, val: String::new(), pos: src.len() });
    Ok(tokens)
}

// ── AST ───────────────────────────────────────────────────────────────────────

/// Tier pattern for match arms.
#[derive(Debug, Clone)]
pub enum TierPattern {
    O0,       // O_0
    O1,       // O_1
    O2,       // O_2
    O2d,      // O_2d
    OInf,     // O_inf
    Wildcard, // _
}

/// A single match arm: tier pattern => expression
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: TierPattern,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Letter(String),           // Hebrew glyph or name
    Name(String),             // Variable name
    Tensor(Box<Expr>, Box<Expr>),
    Join(Box<Expr>, Box<Expr>),
    Meet(Box<Expr>, Box<Expr>),
    Cast(Box<Expr>, String),  // source ::> target
    ProbePhi(Box<Expr>),
    ProbeOmega(Box<Expr>),
    Tier(Box<Expr>),
    Distance(Box<Expr>, Box<Expr>),
    Mediate(Box<Expr>, Box<Expr>, Box<Expr>), // witness, a, b
    System,
    Census,
    Let(String, Box<Expr>),   // name = expr
    Palace(u8, Box<Expr>),    // palace(n) expr — Kabbalistic palace modifier
    Match(Box<Expr>, Vec<MatchArm>),  // match expr { arms }
}

// ── Parser ────────────────────────────────────────────────────────────────────

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn eat(&mut self, expected: Option<TokenKind>) -> Result<Token, String> {
        let t = &self.tokens[self.pos];
        if let Some(kind) = expected {
            if t.kind != kind {
                return Err(format!(
                    "Expected {:?}, got {:?}('{}') at pos {}",
                    kind, t.kind, t.val, t.pos
                ));
            }
        }
        let tok = t.clone();
        self.pos += 1;
        Ok(tok)
    }

    /// Top-level: left-associative binary operators.
    pub fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_primary()?;
        loop {
            let is_ascii_op = if self.peek().kind == TokenKind::Name {
                let val = &self.peek().val;
                val == "x" || val == "v" || val == "^"
            } else {
                false
            };
            if is_ascii_op {
                let op_name = self.peek().val.clone();
                self.eat(None)?;
                let right = self.parse_primary()?;
                match op_name.as_str() {
                    "x" => left = Expr::Tensor(Box::new(left), Box::new(right)),
                    "v" => left = Expr::Join(Box::new(left), Box::new(right)),
                    "^" => left = Expr::Meet(Box::new(left), Box::new(right)),
                    _ => unreachable!(),
                }
                continue;
            }
            match self.peek().kind {
                TokenKind::Tensor => {
                    self.eat(None)?;
                    let right = self.parse_primary()?;
                    left = Expr::Tensor(Box::new(left), Box::new(right));
                }
                TokenKind::Join => {
                    self.eat(None)?;
                    let right = self.parse_primary()?;
                    left = Expr::Join(Box::new(left), Box::new(right));
                }
                TokenKind::Meet => {
                    self.eat(None)?;
                    let right = self.parse_primary()?;
                    left = Expr::Meet(Box::new(left), Box::new(right));
                }
                TokenKind::Cast => {
                    self.eat(None)?;
                    let tgt = self.eat(Some(TokenKind::Name))?;
                    left = Expr::Cast(Box::new(left), tgt.val.clone());
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let t = self.peek();

        // let binding (TokenKind::Let from tokenizer)
        if t.kind == TokenKind::Let {
            self.eat(None)?; // consume 'let'
            let var_tok = self.eat(Some(TokenKind::Name))?;
            self.eat(Some(TokenKind::Eq))?;
            let expr = self.parse_expr()?;
            return Ok(Expr::Let(var_tok.val.clone(), Box::new(expr)));
        }

        // match expression
        if t.kind == TokenKind::Match {
            self.eat(None)?; // consume 'match'
            let scrutinee = self.parse_expr()?;
            self.eat(Some(TokenKind::LBrace))?;
            let mut arms = Vec::new();
            loop {
                // Check for closing brace (allow empty match body)
                if self.peek().kind == TokenKind::RBrace {
                    self.eat(None)?;
                    break;
                }
                // Parse tier pattern
                let pattern = self.parse_tier_pattern()?;
                self.eat(Some(TokenKind::FatArrow))?;
                let arm_expr = self.parse_expr()?;
                // Optional trailing comma
                if self.peek().kind == TokenKind::Comma {
                    self.eat(None)?;
                }
                arms.push(MatchArm {
                    pattern,
                    expr: Box::new(arm_expr),
                });
            }
            return Ok(Expr::Match(Box::new(scrutinee), arms));
        }

        // Hebrew glyph
        if t.kind == TokenKind::Letter {
            let tok = self.eat(None)?;
            return Ok(Expr::Letter(tok.val.clone()));
        }

        // Parenthesised expression
        if t.kind == TokenKind::LParen {
            self.eat(None)?;
            let inner = self.parse_expr()?;
            self.eat(Some(TokenKind::RParen))?;
            return Ok(inner);
        }

        // Named constructs
        if t.kind == TokenKind::Name {
            let name = t.val.clone();

            // probes
            if name == "probe_Φ" || name == "probe_Phi" || name == "probe_Ph" {
                self.eat(None)?;
                return Ok(Expr::ProbePhi(Box::new(self.paren_expr()?)));
            }
            if name == "probe_Ω" || name == "probe_Omega" || name == "probe_Om" {
                self.eat(None)?;
                return Ok(Expr::ProbeOmega(Box::new(self.paren_expr()?)));
            }

            // tier
            if name == "tier" {
                self.eat(None)?;
                return Ok(Expr::Tier(Box::new(self.paren_expr()?)));
            }

            // d(a, b)
            if name == "d" {
                self.eat(None)?;
                self.eat(Some(TokenKind::LParen))?;
                let a = self.parse_expr()?;
                self.eat(Some(TokenKind::Comma))?;
                let b = self.parse_expr()?;
                self.eat(Some(TokenKind::RParen))?;
                return Ok(Expr::Distance(Box::new(a), Box::new(b)));
            }

            // mediate(w, a, b)
            if name == "mediate" {
                self.eat(None)?;
                self.eat(Some(TokenKind::LParen))?;
                let w = self.parse_expr()?;
                self.eat(Some(TokenKind::Comma))?;
                let a = self.parse_expr()?;
                self.eat(Some(TokenKind::Comma))?;
                let b = self.parse_expr()?;
                self.eat(Some(TokenKind::RParen))?;
                return Ok(Expr::Mediate(Box::new(w), Box::new(a), Box::new(b)));
            }

            // system()
            if name == "system" {
                self.eat(None)?;
                self.maybe_unit()?;
                return Ok(Expr::System);
            }

            // census()
            if name == "census" {
                self.eat(None)?;
                self.maybe_unit()?;
                return Ok(Expr::Census);
            }

            // palace(n) expr — prefix modifier
            if name == "palace" {
                self.eat(None)?; // consume 'palace'
                self.eat(Some(TokenKind::LParen))?;
                let num_tok = self.eat(Some(TokenKind::Int))?;
                let palace_num: u8 = num_tok.val.parse()
                    .map_err(|_| format!("Invalid palace number: '{}'", num_tok.val))?;
                self.eat(Some(TokenKind::RParen))?;
                let inner = self.parse_expr()?;
                return Ok(Expr::Palace(palace_num, Box::new(inner)));
            }

            // Plain name (letter or variable)
            self.eat(None)?;
            return Ok(Expr::Name(name));
        }

        Err(format!("Unexpected token {:?}('{}') at pos {}", t.kind, t.val, t.pos))
    }

    fn paren_expr(&mut self) -> Result<Expr, String> {
        self.eat(Some(TokenKind::LParen))?;
        let inner = self.parse_expr()?;
        self.eat(Some(TokenKind::RParen))?;
        Ok(inner)
    }

    fn maybe_unit(&mut self) -> Result<(), String> {
        if self.peek().kind == TokenKind::LParen {
            self.eat(Some(TokenKind::LParen))?;
            self.eat(Some(TokenKind::RParen))?;
        }
        Ok(())
    }

    fn parse_tier_pattern(&mut self) -> Result<TierPattern, String> {
        let t = self.eat(None)?;
        match t.val.as_str() {
            "O_0" => Ok(TierPattern::O0),
            "O_1" => Ok(TierPattern::O1),
            "O_2" => Ok(TierPattern::O2),
            "O_2d" => Ok(TierPattern::O2d),
            "O_inf" => Ok(TierPattern::OInf),
            "_" => Ok(TierPattern::Wildcard),
            _ => Err(format!("Unknown tier pattern: '{}'. Expected O_0, O_1, O_2, O_2d, O_inf, or _", t.val)),
        }
    }
}

/// Parse a source string into an AST.
pub fn parse(src: &str) -> Result<Expr, String> {
    let tokens = tokenize(src)?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr()?;
    // Ensure we consumed everything (except EOF)
    if parser.peek().kind != TokenKind::Eof {
        return Err(format!(
            "Unexpected token after expression: {:?}('{}')",
            parser.peek().kind, parser.peek().val
        ));
    }
    Ok(expr)
}
