//! `Entity`: qualquer "coisa" que tem atributos — personagem, NPC,
//! monstro, ou até um item mágico com "vida própria". Guarda só dois
//! tipos de dado: valores base (override explícito do padrão do
//! Ruleset) e a lista de Effects atualmente ativos. Não sabe calcular
//! nada sozinha — isso é responsabilidade do `engine::compute_attributes`.

use crate::effect::Effect;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: String,
    pub ruleset_id: String,
    base_attributes: HashMap<String, i64>,
    effects: Vec<Effect>,
}

impl Entity {
    pub fn new(id: impl Into<String>, ruleset_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ruleset_id: ruleset_id.into(),
            base_attributes: HashMap::new(),
            effects: Vec::new(),
        }
    }

    /// Define um valor base explícito, sobrescrevendo o `default_value`
    /// do `AttributeDefinition` correspondente no Ruleset.
    pub fn set_base(&mut self, attribute_id: impl Into<String>, value: i64) {
        self.base_attributes.insert(attribute_id.into(), value);
    }

    pub fn base(&self, attribute_id: &str) -> Option<i64> {
        self.base_attributes.get(attribute_id).copied()
    }

    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }
}
