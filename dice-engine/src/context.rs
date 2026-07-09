//! Contexto de variáveis usado para resolver `Expr::Variable` durante a
//! avaliação. É um wrapper fino sobre HashMap de propósito — se um dia
//! precisar otimizar (ex: acesso por índice em vez de string), a troca
//! fica isolada aqui, sem afetar o evaluator.

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct RollContext {
    variables: HashMap<String, i64>,
}

impl RollContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, name: impl Into<String>, value: i64) -> Self {
        self.variables.insert(name.into(), value);
        self
    }

    pub fn set(&mut self, name: impl Into<String>, value: i64) {
        self.variables.insert(name.into(), value);
    }

    pub fn get(&self, name: &str) -> Option<i64> {
        self.variables.get(name).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_and_get() {
        let ctx = RollContext::new().with("STR", 3).with("PROF", 2);
        assert_eq!(ctx.get("STR"), Some(3));
        assert_eq!(ctx.get("PROF"), Some(2));
        assert_eq!(ctx.get("DEX"), None);
    }
}
