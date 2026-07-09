//! DTO de `ruleset.json` — declara os `AttributeDefinition`/`DerivedRule`
//! de um Ruleset inteiro (ex: os seis atributos de D&D 5e + seus
//! modificadores + bônus de proficiência). Convenção introduzida nesta
//! fase (não estava explícita no documento de arquitetura, que definia
//! `ContentNode` individual mas não onde vive a declaração do Ruleset
//! como um todo) — decisão pequena de formato de arquivo, não mudança
//! de arquitetura.

use engine_core::{AttributeDefinition, DerivedRule, Ruleset};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AttributeDef {
    pub id: String,
    pub label: String,
    pub default: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DerivedRuleDef {
    pub id: String,
    pub formula: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RulesetFile {
    pub id: String,
    pub attributes: Vec<AttributeDef>,
    pub derived_rules: Vec<DerivedRuleDef>,
}

impl RulesetFile {
    /// Converte o DTO de arquivo pro tipo de domínio
    /// (`engine_core::Ruleset`). Essa conversão explícita — em vez de
    /// derivar `Deserialize` direto nos tipos do `engine-core` — é o que
    /// mantém o Domain Layer sem depender de serde/formato de arquivo.
    /// Quem sabe ler JSON é o content-loader, não o core.
    pub fn into_ruleset(self) -> Ruleset {
        Ruleset {
            attributes: self
                .attributes
                .into_iter()
                .map(|a| AttributeDefinition::new(a.id, a.label, a.default))
                .collect(),
            derived_rules: self
                .derived_rules
                .into_iter()
                .map(|r| DerivedRule::new(r.id, r.formula))
                .collect(),
        }
    }
}
