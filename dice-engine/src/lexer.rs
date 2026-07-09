//! Lexer: transforma a string da fórmula em uma sequência de tokens.
//!
//! Responsabilidade única: reconhecer símbolos. O lexer NÃO sabe o que é
//! "dado", "vantagem" ou qualquer semântica de RPG — isso é decisão do
//! parser/evaluator. O lexer só sabe: isto é um número, isto é uma
//! palavra, isto é um operador.
//!
//! Duas palavras têm tratamento especial na hora de tokenizar (mas ainda
//! são só símbolos, sem significado de regra):
//! - `d`/`D` isolado -> Token::Dice (separador de "quantidade" e "lados")
//! - `kh`/`kl` (case-insensitive) -> Token::KeepHighest / Token::KeepLowest
//!
//! Qualquer outra sequência de letras vira Token::Identifier (variável,
//! ex: STR, PROF, level).

use crate::error::DiceError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(i64),
    Identifier(String),
    Dice,
    KeepHighest,
    KeepLowest,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Eof,
}

pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    /// Tokeniza a string inteira de uma vez. Fórmulas de rolagem são
    /// curtas (poucas dezenas de caracteres), então não há necessidade
    /// de tokenizar sob demanda / streaming.
    pub fn tokenize(input: &'a str) -> Result<Vec<Token>, DiceError> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token()?;
            let is_eof = token == Token::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, DiceError> {
        self.skip_whitespace();

        let Some(&ch) = self.chars.peek() else {
            return Ok(Token::Eof);
        };

        match ch {
            '+' => {
                self.chars.next();
                Ok(Token::Plus)
            }
            '-' => {
                self.chars.next();
                Ok(Token::Minus)
            }
            '*' => {
                self.chars.next();
                Ok(Token::Star)
            }
            '/' => {
                self.chars.next();
                Ok(Token::Slash)
            }
            '(' => {
                self.chars.next();
                Ok(Token::LParen)
            }
            ')' => {
                self.chars.next();
                Ok(Token::RParen)
            }
            c if c.is_ascii_digit() => self.read_number(),
            c if c.is_ascii_alphabetic() || c == '_' => self.read_word(),
            other => Err(DiceError::UnexpectedChar(other)),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Result<Token, DiceError> {
        let mut buf = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() {
                buf.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        buf.parse::<i64>()
            .map(Token::Number)
            .map_err(|_| DiceError::InvalidNumber(buf))
    }

    fn read_word(&mut self) -> Result<Token, DiceError> {
        let mut buf = String::new();
        while let Some(&c) = self.chars.peek() {
            // Só letras e '_' — dígitos SEMPRE terminam uma palavra.
            // Isso é o que separa corretamente "d20" em Token::Dice +
            // Token::Number(20) em vez de ler tudo como um identificador
            // só ("d20"). Sem essa restrição, "4d6kh3" também quebraria
            // (leria "d6kh3" como uma palavra única).
            if c.is_ascii_alphabetic() || c == '_' {
                buf.push(c);
                self.chars.next();
            } else {
                break;
            }
        }

        let lower = buf.to_ascii_lowercase();
        match lower.as_str() {
            "d" => Ok(Token::Dice),
            "kh" => Ok(Token::KeepHighest),
            "kl" => Ok(Token::KeepLowest),
            // Preserva o texto original (case) para identificadores —
            // "STR" e "str" podem ser convenções diferentes dependendo
            // do content-pack, então quem decide normalizar é o
            // resolvedor de contexto (RollContext), não o lexer.
            _ => Ok(Token::Identifier(buf)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_simple_expression() {
        let tokens = Lexer::tokenize("1d20+STR+PROF").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number(1),
                Token::Dice,
                Token::Number(20),
                Token::Plus,
                Token::Identifier("STR".to_string()),
                Token::Plus,
                Token::Identifier("PROF".to_string()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenizes_implicit_one_dice() {
        let tokens = Lexer::tokenize("d20").unwrap();
        assert_eq!(tokens, vec![Token::Dice, Token::Number(20), Token::Eof]);
    }

    #[test]
    fn tokenizes_keep_highest() {
        let tokens = Lexer::tokenize("4d6kh3").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number(4),
                Token::Dice,
                Token::Number(6),
                Token::KeepHighest,
                Token::Number(3),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenizes_parens_and_all_operators() {
        let tokens = Lexer::tokenize("(2d6 + 1) * 2 - STR / 2").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::Number(2),
                Token::Dice,
                Token::Number(6),
                Token::Plus,
                Token::Number(1),
                Token::RParen,
                Token::Star,
                Token::Number(2),
                Token::Minus,
                Token::Identifier("STR".to_string()),
                Token::Slash,
                Token::Number(2),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn rejects_unknown_char() {
        let err = Lexer::tokenize("1d20 & STR").unwrap_err();
        assert!(matches!(err, DiceError::UnexpectedChar('&')));
    }
}
