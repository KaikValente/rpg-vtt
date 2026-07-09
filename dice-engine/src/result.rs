//! Resultado estruturado de uma rolagem. O objetivo não é só devolver um
//! número — é devolver uma árvore auditável que a UI pode renderizar como
//! "1d20 + STR + PROF -> 14 + 3 + 2 -> 19" sem recalcular nada e sem
//! precisar entender a fórmula original.

use crate::ast::{BinaryOp, DiceModifier};

#[derive(Debug, Clone, PartialEq)]
pub enum EvalNode {
    Number(i64),
    Variable {
        name: String,
        value: i64,
    },
    Dice {
        count: u32,
        sides: u32,
        modifier: DiceModifier,
        /// Todos os valores rolados, na ordem em que saíram do Roller.
        /// Para vantagem/desvantagem, contém as DUAS rolagens do d20
        /// (ex: [14, 9]), não só a mantida — isso é o que permite a UI
        /// mostrar "rolou 14 e 9, manteve 14 (vantagem)".
        rolls: Vec<i64>,
        /// Subconjunto de `rolls` que efetivamente contou pro total
        /// (depois de aplicar modifier e/ou vantagem/desvantagem).
        kept: Vec<i64>,
        total: i64,
    },
    Binary {
        op: BinaryOp,
        left: Box<EvalNode>,
        right: Box<EvalNode>,
        total: i64,
    },
}

impl EvalNode {
    pub fn total(&self) -> i64 {
        match self {
            EvalNode::Number(n) => *n,
            EvalNode::Variable { value, .. } => *value,
            EvalNode::Dice { total, .. } => *total,
            EvalNode::Binary { total, .. } => *total,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollResult {
    pub total: i64,
    pub root: EvalNode,
}

impl RollResult {
    /// Representação textual simples do breakdown, útil pra debug/log.
    /// A UI de verdade (Fase 6) deve renderizar `root` com seu próprio
    /// componente visual, não parsear esta string.
    pub fn describe(&self) -> String {
        format!("{} = {}", describe_node(&self.root), self.total)
    }
}

fn describe_node(node: &EvalNode) -> String {
    match node {
        EvalNode::Number(n) => n.to_string(),
        EvalNode::Variable { name, value } => format!("{name}({value})"),
        EvalNode::Dice {
            count,
            sides,
            rolls,
            kept,
            ..
        } => {
            format!("{count}d{sides}{rolls:?}->kept{kept:?}")
        }
        EvalNode::Binary {
            op, left, right, ..
        } => {
            let op_str = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
            };
            format!("({} {} {})", describe_node(left), op_str, describe_node(right))
        }
    }
}
