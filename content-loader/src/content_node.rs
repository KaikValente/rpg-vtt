//! DTO de um `ContentNode` individual (raça, classe, magia, item,
//! monstro, feature, asset). Ver seção 11.2 do documento de arquitetura
//! pra estrutura completa. Nesta fase, só o suficiente pra ler o
//! envelope genérico (`metadata`/`presentation`/`mechanics`) e converter
//! `mechanics.effects` pro tipo `engine_core::Effect` — a interpretação
//! de `mechanics.data` por tipo (`race`, `class`, `spell`...) fica pra
//! Fase 4, quando o SRD de verdade for modelado.

use crate::error::LoaderError;
use engine_core::{Duration, Effect, Operation, Stacking};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ContentNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub metadata: NodeMetadata,
    pub presentation: NodePresentation,
    pub mechanics: NodeMechanics,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeMetadata {
    pub pack_id: String,
    pub version: String,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodePresentation {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub icon_asset: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeMechanics {
    /// Bloco específico do `type` (ex: `damage_formula` numa spell,
    /// `hit_dice` numa class). Não interpretado nesta fase — só
    /// preservado como JSON genérico. A Fase 4 é quem vai saber o que
    /// fazer com isso, tipo por tipo.
    #[serde(default)]
    pub data: serde_json::Value,
    #[serde(default)]
    pub effects: Vec<EffectDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EffectDef {
    pub target: String,
    pub operation: String,
    pub value: String,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub stacking: Option<String>,
}

impl EffectDef {
    pub fn into_effect(self, source: &str) -> Result<Effect, LoaderError> {
        let operation = match self.operation.as_str() {
            "add" => Operation::Add,
            "multiply" => Operation::Multiply,
            "set" => Operation::Set,
            other => return Err(LoaderError::UnknownOperation(other.to_string())),
        };

        let mut effect = Effect::new(self.target, operation, self.value, source);

        if let Some(duration) = self.duration {
            let duration = match duration.as_str() {
                "permanent" => Duration::Permanent,
                "until_unequipped" => Duration::UntilUnequipped,
                "concentration" => Duration::Concentration,
                // Nota: Duration::Rounds(u32) não tem formato de string
                // simples — não apareceu ainda em conteúdo real (só a
                // Fase 7/combate vai precisar disso de verdade). Quando
                // precisar, vira algo tipo {"duration": {"rounds": 3}}
                // no JSON, tratado à parte.
                other => return Err(LoaderError::UnknownDuration(other.to_string())),
            };
            effect = effect.with_duration(duration);
        }

        if let Some(stacking) = self.stacking {
            let stacking = match stacking.as_str() {
                "stack" => Stacking::Stack,
                "no_stack" => Stacking::NoStack,
                "highest_wins" => Stacking::HighestWins,
                other => return Err(LoaderError::UnknownStacking(other.to_string())),
            };
            effect = effect.with_stacking(stacking);
        }

        Ok(effect)
    }
}

impl ContentNode {
    /// Extrai os Effects deste node já convertidos pro tipo do
    /// engine-core, usando o próprio id do node como `source` — assim
    /// fica rastreável de onde cada modificação da ficha veio.
    pub fn effects(&self) -> Result<Vec<Effect>, LoaderError> {
        self.mechanics
            .effects
            .iter()
            .cloned()
            .map(|e| e.into_effect(&self.id))
            .collect()
    }
}
