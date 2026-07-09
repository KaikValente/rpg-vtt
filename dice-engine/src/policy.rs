//! Política de rolagem: tudo que NÃO pertence à expressão matemática em
//! si, mas afeta como ela é avaliada. É aqui, e não no parser/AST, que
//! mora a semântica de "vantagem" — que é uma regra específica de D&D 5e
//! (rolar duas vezes, manter maior), não um conceito universal de dados.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Advantage {
    #[default]
    None,
    Advantage,
    Disadvantage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RollPolicy {
    pub advantage: Advantage,
    /// Crítico dobra a quantidade de dados rolados (regra de dano de D&D
    /// 5e), nunca o modificador. Aplicado pelo evaluator em todo nó
    /// `Dice` da árvore — se a fórmula for só de ataque (`1d20+STR`), não
    /// há dado de dano para dobrar, então este campo simplesmente não
    /// tem efeito nessa avaliação.
    pub critical: bool,
}

impl RollPolicy {
    pub fn normal() -> Self {
        Self::default()
    }

    pub fn with_advantage() -> Self {
        Self {
            advantage: Advantage::Advantage,
            critical: false,
        }
    }

    pub fn with_disadvantage() -> Self {
        Self {
            advantage: Advantage::Disadvantage,
            critical: false,
        }
    }

    pub fn critical(mut self) -> Self {
        self.critical = true;
        self
    }
}
