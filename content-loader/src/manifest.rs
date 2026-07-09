//! DTOs (Data Transfer Objects) do `manifest.json` — a forma como um
//! pacote se descreve (id, versão, tipo, dependências). Ver seção 11.1
//! do documento de arquitetura.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackType {
    CoreSystem,
    Supplement,
    Homebrew,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Dependency {
    pub pack_id: String,
    pub min_version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub version: String,
    pub ruleset: String,
    #[serde(rename = "type")]
    pub pack_type: PackType,
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub content_files: Vec<String>,
}
