//! AST: representa somente a expressão matemática da fórmula.
//!
//! Importante (decisão de arquitetura revisada): `Expr` NUNCA sabe o que é
//! vantagem, desvantagem ou crítico. `1d20+STR+PROF` gera exatamente a
//! mesma árvore seja qual for a política de rolagem — quem decide "como"
//! avaliar essa árvore é o `evaluator`, usando uma `RollPolicy` (ver
//! policy.rs). Isso mantém o parser 100% genérico e reutilizável por
//! qualquer sistema de RPG futuro, não só D&D 5e.

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    Variable(String),
    Dice(DiceExpr),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiceExpr {
    pub count: u32,
    pub sides: u32,
    pub modifier: DiceModifier,
}

/// Transformações sobre o conjunto de dados rolados. Modelado como enum
/// (não string) de propósito: cada variante nova (DropHighest, Explode,
/// Reroll...) vira um branch no evaluator, não um parser especial.
/// Nenhuma dessas variantes futuras entra agora — SRD de D&D 5e no MVP
/// só usa KeepHighest/KeepLowest (ex: `4d6kh3` para rolagem de atributos).
#[derive(Debug, Clone, PartialEq)]
pub enum DiceModifier {
    None,
    KeepHighest(u32),
    KeepLowest(u32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}
