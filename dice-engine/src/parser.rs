//! Parser: recursive descent, não Pratt parser.
//!
//! Decisão consciente: a gramática do MVP tem só dois níveis de
//! precedência (`+`/`-` e `*`/`/`), sem operadores unários/pós-fixos
//! complexos. Um recursive descent parser resolve isso com a mesma
//! corretude que um Pratt parser, com menos código e mais fácil de ler
//! para quem está aprendendo Rust. Se a gramática crescer de verdade no
//! futuro (o que o critério de mudança da arquitetura deixa explícito que
//! não deve acontecer por antecipação), migrar para Pratt parser é um
//! refactor localizado nesse arquivo, não um redesenho do crate.
//!
//! Gramática implementada:
//! ```text
//! expression := term (('+' | '-') term)*
//! term       := factor (('*' | '/') factor)*
//! factor     := dice | number | variable | '(' expression ')' | '-' factor
//! dice       := [number] 'd' number (('kh' | 'kl') number)?
//! ```

use crate::ast::{BinaryOp, DiceExpr, DiceModifier, Expr};
use crate::error::DiceError;
use crate::lexer::{Lexer, Token};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Ponto de entrada público: string da fórmula -> AST.
    pub fn parse(input: &str) -> Result<Expr, DiceError> {
        let tokens = Lexer::tokenize(input)?;
        let mut parser = Parser { tokens, pos: 0 };
        let expr = parser.parse_expression()?;
        parser.expect(&Token::Eof)?;
        Ok(expr)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), DiceError> {
        if self.peek() == expected {
            self.advance();
            Ok(())
        } else {
            Err(DiceError::UnexpectedToken(format!("{:?}", self.peek())))
        }
    }

    /// Consome um Token::Number e devolve o i64 interno, ou erro.
    fn expect_number(&mut self) -> Result<i64, DiceError> {
        match self.advance() {
            Token::Number(n) => Ok(n),
            other => Err(DiceError::UnexpectedToken(format!("{other:?}"))),
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, DiceError> {
        let mut left = self.parse_term()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_term()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, DiceError> {
        let mut left = self.parse_factor()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr, DiceError> {
        match self.peek().clone() {
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            // Unário: "-2" ou "-1d4". Baixo custo, evita surpresa quando
            // um Effect gera uma fórmula com valor negativo (ex: "-2").
            Token::Minus => {
                self.advance();
                let inner = self.parse_factor()?;
                Ok(Expr::Binary {
                    left: Box::new(Expr::Number(0)),
                    op: BinaryOp::Sub,
                    right: Box::new(inner),
                })
            }
            Token::Dice => {
                self.advance();
                self.parse_dice_tail(1)
            }
            Token::Number(n) => {
                self.advance();
                if self.peek() == &Token::Dice {
                    self.advance();
                    let count = u32::try_from(n)
                        .map_err(|_| DiceError::InvalidDiceSpec { count: 0, sides: 0 })?;
                    self.parse_dice_tail(count)
                } else {
                    Ok(Expr::Number(n))
                }
            }
            Token::Identifier(name) => {
                self.advance();
                Ok(Expr::Variable(name))
            }
            other => Err(DiceError::UnexpectedToken(format!("{other:?}"))),
        }
    }

    /// Já consumiu o `Token::Dice` e sabe a quantidade (`count`, 1 se
    /// implícito via `d20`). Falta ler os lados e o modificador opcional.
    fn parse_dice_tail(&mut self, count: u32) -> Result<Expr, DiceError> {
        let sides_raw = self.expect_number()?;
        let sides =
            u32::try_from(sides_raw).map_err(|_| DiceError::InvalidDiceSpec { count, sides: 0 })?;

        if count == 0 || sides == 0 {
            return Err(DiceError::InvalidDiceSpec { count, sides });
        }

        let modifier = match self.peek() {
            Token::KeepHighest => {
                self.advance();
                let n = self.expect_number()?;
                DiceModifier::KeepHighest(u32::try_from(n).unwrap_or(0))
            }
            Token::KeepLowest => {
                self.advance();
                let n = self.expect_number()?;
                DiceModifier::KeepLowest(u32::try_from(n).unwrap_or(0))
            }
            _ => DiceModifier::None,
        };

        Ok(Expr::Dice(DiceExpr {
            count,
            sides,
            modifier,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_number() {
        assert_eq!(Parser::parse("42").unwrap(), Expr::Number(42));
    }

    #[test]
    fn parses_implicit_single_dice() {
        assert_eq!(
            Parser::parse("d20").unwrap(),
            Expr::Dice(DiceExpr {
                count: 1,
                sides: 20,
                modifier: DiceModifier::None
            })
        );
    }

    #[test]
    fn parses_explicit_dice_with_variables() {
        let expr = Parser::parse("1d20+STR+PROF").unwrap();
        // 1d20 + STR + PROF é associativo à esquerda:
        // (1d20 + STR) + PROF
        let expected = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Dice(DiceExpr {
                    count: 1,
                    sides: 20,
                    modifier: DiceModifier::None,
                })),
                op: BinaryOp::Add,
                right: Box::new(Expr::Variable("STR".to_string())),
            }),
            op: BinaryOp::Add,
            right: Box::new(Expr::Variable("PROF".to_string())),
        };
        assert_eq!(expr, expected);
    }

    #[test]
    fn parses_keep_highest_stat_roll() {
        assert_eq!(
            Parser::parse("4d6kh3").unwrap(),
            Expr::Dice(DiceExpr {
                count: 4,
                sides: 6,
                modifier: DiceModifier::KeepHighest(3),
            })
        );
    }

    #[test]
    fn respects_precedence_and_parens() {
        // (2d6 + 1) * 2  != 2d6 + 1 * 2
        let with_parens = Parser::parse("(1+1)*2").unwrap();
        assert_eq!(
            with_parens,
            Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Number(1)),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::Number(1)),
                }),
                op: BinaryOp::Mul,
                right: Box::new(Expr::Number(2)),
            }
        );
    }

    #[test]
    fn parses_unary_minus() {
        assert_eq!(
            Parser::parse("-2").unwrap(),
            Expr::Binary {
                left: Box::new(Expr::Number(0)),
                op: BinaryOp::Sub,
                right: Box::new(Expr::Number(2)),
            }
        );
    }

    #[test]
    fn rejects_dangling_operator() {
        assert!(Parser::parse("1+").is_err());
    }

    #[test]
    fn rejects_zero_sided_dice() {
        assert!(Parser::parse("1d0").is_err());
    }
}
