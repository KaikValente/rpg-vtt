//! Erros do dice-engine. Um crate pequeno e isolado como este não precisa
//! de uma taxonomia elaborada — só o suficiente para a UI (ou quem chamar
//! o crate) mostrar uma mensagem útil sobre o que deu errado e onde.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum DiceError {
    UnexpectedChar(char),
    InvalidNumber(String),
    UnexpectedToken(String),
    UnexpectedEof,
    UnknownVariable(String),
    DivisionByZero,
    /// `count`/`sides` fora de faixa razoável (ex: 0 lados, quantidade
    /// negativa via overflow). Evita que uma fórmula mal-intencionada ou
    /// bugada trave o programa alocando um Vec gigante.
    InvalidDiceSpec { count: u32, sides: u32 },
}

impl fmt::Display for DiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiceError::UnexpectedChar(c) => write!(f, "caractere inesperado: '{c}'"),
            DiceError::InvalidNumber(s) => write!(f, "número inválido: '{s}'"),
            DiceError::UnexpectedToken(t) => write!(f, "token inesperado: {t}"),
            DiceError::UnexpectedEof => write!(f, "fórmula terminou de forma inesperada"),
            DiceError::UnknownVariable(v) => write!(f, "variável desconhecida no contexto: '{v}'"),
            DiceError::DivisionByZero => write!(f, "divisão por zero na fórmula"),
            DiceError::InvalidDiceSpec { count, sides } => {
                write!(f, "especificação de dado inválida: {count}d{sides}")
            }
        }
    }
}

impl std::error::Error for DiceError {}
