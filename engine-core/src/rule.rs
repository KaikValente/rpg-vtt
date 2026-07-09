//! Uma `DerivedRule` CALCULA um valor a partir de outros (ex: bônus de
//! proficiência a partir do nível). Isso é diferente de `Effect`, que
//! MODIFICA um valor existente e só se aplica quando "ativo" numa
//! Entity — ver a distinção completa em `effect.rs`.
//!
//! A fórmula é uma string avaliada pelo `dice-engine` (o mesmo crate que
//! resolve `1d20+STR+PROF` também resolve `2 + (level-1)/4` — é só uma
//! expressão sem dados). O core não interpreta a fórmula, só a delega.

#[derive(Debug, Clone, PartialEq)]
pub struct DerivedRule {
    pub id: String,
    pub formula: String,
}

impl DerivedRule {
    pub fn new(id: impl Into<String>, formula: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            formula: formula.into(),
        }
    }
}
