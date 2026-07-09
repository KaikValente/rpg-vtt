//! `mechanics.data` tipado, por `type` de `ContentNode`. Na Fase 3,
//! `data` era só `serde_json::Value` genérico (não interpretado). Aqui
//! interpretamos os tipos que a fatia vertical da Fase 4 realmente usa:
//! `race`, `feature`, `class`, `spell`, `item`.
//!
//! Padrão escolhido: "tipado sob demanda". Em vez de fazer o
//! `ContentNode` já vir com `data` tipado (o que exigiria deserialização
//! polimórfica baseada no campo `type`, mais complexa em serde), o
//! `ContentNode` continua guardando `data` como `serde_json::Value`
//! genérico, e cada tipo ganha um método de conversão sob demanda —
//! `ContentNode::race_data()`, `ContentNode::spell_data()`, etc. Cada um
//! confere que `node_type` bate com o esperado antes de tentar
//! deserializar, e retorna `LoaderError::TypeMismatch` se não bater.
//!
//! Isso evita reescrever a estrutura genérica do `ContentNode` (Fase 3)
//! e mantém a adição de um tipo novo (`monster`, mais pra frente)
//! isolada: só mais um struct + um método, sem tocar no resto.

use crate::content_node::ContentNode;
use crate::error::LoaderError;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RaceData {
    pub size: String,
    pub speed: i64,
    #[serde(default)]
    pub languages: Vec<String>,
    /// Ids de ContentNodes do tipo `feature` que este race concede.
    /// Resolução (carregar cada id de verdade) ainda é manual nesta
    /// fase — o Content Registry (arquitetura, seção 1) que faria isso
    /// automaticamente ainda não existe; só entra quando o volume de
    /// conteúdo justificar.
    #[serde(default)]
    pub traits: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeatureData {
    pub source: String,
    pub level_requirement: i64,
}

/// Simplificação da Fase 4 em relação à seção 11.3 da arquitetura: o
/// schema original previa `levels: [{level, features, spell_slots}]`
/// (progressão completa por nível). Para a fatia vertical (1 personagem
/// nível 1), isso é complexidade sem uso ainda — fica de fora até
/// existir um segundo nível de personagem pra validar contra.
#[derive(Debug, Clone, Deserialize)]
pub struct ClassData {
    pub hit_dice: String,
    pub primary_attribute: String,
    #[serde(default)]
    pub saving_throw_proficiencies: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpellData {
    pub level: i64,
    pub school: String,
    pub casting_time: String,
    pub range: String,
    #[serde(default)]
    pub components: Vec<String>,
    pub duration: String,
    #[serde(default)]
    pub damage_formula: Option<String>,
    #[serde(default)]
    pub damage_type: Option<String>,
    #[serde(default)]
    pub save: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeaponData {
    pub damage_formula: String,
    pub damage_type: String,
    #[serde(default)]
    pub properties: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ItemData {
    pub item_type: String,
    pub weight: f64,
    pub cost_gp: i64,
    #[serde(default)]
    pub weapon: Option<WeaponData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonsterAbilityScores {
    #[serde(rename = "STR")]
    pub str_score: i64,
    #[serde(rename = "DEX")]
    pub dex_score: i64,
    #[serde(rename = "CON")]
    pub con_score: i64,
    #[serde(rename = "INT")]
    pub int_score: i64,
    #[serde(rename = "WIS")]
    pub wis_score: i64,
    #[serde(rename = "CHA")]
    pub cha_score: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonsterActionData {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub attack_bonus: Option<i64>,
    #[serde(default)]
    pub damage_formula: Option<String>,
    #[serde(default)]
    pub damage_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonsterData {
    pub size: String,
    pub creature_type: String,
    pub alignment: String,
    pub armor_class: i64,
    pub hit_points: i64,
    pub hit_dice: String,
    pub speed: String,
    pub challenge_rating: String,
    pub ability_scores: MonsterAbilityScores,
    #[serde(default)]
    pub senses: Vec<String>,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub actions: Vec<MonsterActionData>,
}

impl ContentNode {
    fn check_type(&self, expected: &str) -> Result<(), LoaderError> {
        if self.node_type == expected {
            Ok(())
        } else {
            Err(LoaderError::TypeMismatch {
                expected: expected.to_string(),
                actual: self.node_type.clone(),
            })
        }
    }

    pub fn race_data(&self) -> Result<RaceData, LoaderError> {
        self.check_type("race")?;
        Ok(serde_json::from_value(self.mechanics.data.clone())?)
    }

    pub fn feature_data(&self) -> Result<FeatureData, LoaderError> {
        self.check_type("feature")?;
        Ok(serde_json::from_value(self.mechanics.data.clone())?)
    }

    pub fn class_data(&self) -> Result<ClassData, LoaderError> {
        self.check_type("class")?;
        Ok(serde_json::from_value(self.mechanics.data.clone())?)
    }

    pub fn spell_data(&self) -> Result<SpellData, LoaderError> {
        self.check_type("spell")?;
        Ok(serde_json::from_value(self.mechanics.data.clone())?)
    }

    pub fn item_data(&self) -> Result<ItemData, LoaderError> {
        self.check_type("item")?;
        Ok(serde_json::from_value(self.mechanics.data.clone())?)
    }

    pub fn monster_data(&self) -> Result<MonsterData, LoaderError> {
        self.check_type("monster")?;
        Ok(serde_json::from_value(self.mechanics.data.clone())?)
    }
}
