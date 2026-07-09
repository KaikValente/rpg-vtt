//! # dice-engine
//!
//! Motor de rolagem de dados independente e agnóstico de sistema de RPG.
//!
//! Pipeline: `String -> Lexer -> Parser -> Expr (AST) -> Evaluator -> RollResult`.
//!
//! Ponto central de design: o AST (`Expr`) nunca conhece vantagem,
//! desvantagem ou crítico — essas regras vivem em `RollPolicy` e só são
//! aplicadas pelo `evaluator`, no momento de percorrer a árvore. Isso
//! mantém o parser 100% reutilizável por qualquer sistema de RPG futuro.
//!
//! ## Exemplo
//! ```
//! use dice_engine::{roll, RollContext, RollPolicy};
//!
//! let ctx = RollContext::new().with("STR", 3).with("PROF", 2);
//! let result = roll("1d20+STR+PROF", &ctx, &RollPolicy::with_advantage()).unwrap();
//! println!("{}", result.describe());
//! ```

mod ast;
mod context;
mod error;
mod evaluator;
mod lexer;
mod parser;
mod policy;
mod result;
mod rng;

pub use ast::{BinaryOp, DiceExpr, DiceModifier, Expr};
pub use context::RollContext;
pub use error::DiceError;
pub use policy::{Advantage, RollPolicy};
pub use result::{EvalNode, RollResult};
pub use rng::{RandRoller, Roller};

use parser::Parser;

/// API principal do crate: recebe a string de uma fórmula, resolve
/// variáveis via `RollContext`, aplica `RollPolicy` e devolve um
/// `RollResult` estruturado (total + árvore auditável).
///
/// Usa `RandRoller` (aleatoriedade real) internamente. Para testes
/// determinísticos, monte o pipeline manualmente com `parse_formula` +
/// `evaluate_with_roller` e um `Roller` próprio.
pub fn roll(formula: &str, ctx: &RollContext, policy: &RollPolicy) -> Result<RollResult, DiceError> {
    let expr = Parser::parse(formula)?;
    let mut roller = RandRoller::new();
    evaluator::evaluate(&expr, ctx, policy, &mut roller)
}

/// Só faz o parse, sem avaliar. Útil pra validar uma fórmula (ex: ao
/// carregar um content-pack, checar se o `formula` de um Effect é
/// sintaticamente válido) sem gastar nenhuma rolagem de dado.
pub fn parse_formula(formula: &str) -> Result<Expr, DiceError> {
    Parser::parse(formula)
}

/// Avalia um `Expr` já parseado, usando um `Roller` fornecido por quem
/// chama — é o que os testes usam para injetar `FixedRoller` e o que
/// permitiria, no futuro, trocar a fonte de aleatoriedade sem tocar
/// no parser.
pub fn evaluate_with_roller(
    expr: &Expr,
    ctx: &RollContext,
    policy: &RollPolicy,
    roller: &mut impl Roller,
) -> Result<RollResult, DiceError> {
    evaluator::evaluate(expr, ctx, policy, roller)
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn end_to_end_roll_without_dice_uses_real_rng_but_stays_in_range() {
        // Sem mockar RNG aqui: só garante que o pipeline público inteiro
        // (roll -> parse -> evaluate com RandRoller de verdade) não
        // quebra e devolve um total dentro do intervalo esperado.
        let ctx = RollContext::new().with("STR", 3);
        let result = roll("1d20+STR", &ctx, &RollPolicy::normal()).unwrap();
        assert!(result.total >= 1 + 3 && result.total <= 20 + 3);
    }

    #[test]
    fn parse_formula_validates_without_rolling() {
        assert!(parse_formula("1d20+STR+PROF").is_ok());
        assert!(parse_formula("1d20 & STR").is_err());
    }
}
