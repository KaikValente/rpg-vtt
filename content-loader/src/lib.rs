//! # content-loader
//!
//! Content Layer do RPG Engine: lê `manifest.json`, `ruleset.json` e
//! `ContentNode`s individuais (raça, classe, magia, item, monstro,
//! feature) de um pacote em disco, convertendo pros tipos de domínio
//! do `engine-core`.
//!
//! Fronteira importante: este crate é o único que sabe ler JSON/arquivo.
//! O `engine-core` nunca depende de serde — a conversão de DTO pra tipo
//! de domínio acontece só aqui (ver `ruleset_file.rs`/`content_node.rs`).

mod content_node;
mod data_types;
mod error;
mod loader;
mod manifest;
mod ruleset_file;

pub use content_node::{ContentNode, EffectDef, NodeMechanics, NodeMetadata, NodePresentation};
pub use data_types::{ClassData, FeatureData, ItemData, RaceData, SpellData, WeaponData};
pub use error::LoaderError;
pub use loader::{load_content_node, load_manifest, load_ruleset};
pub use manifest::{Dependency, Manifest, PackType};
pub use ruleset_file::{AttributeDef, DerivedRuleDef, RulesetFile};
