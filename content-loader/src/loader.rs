//! Funções de entrada: leem um arquivo do filesystem e devolvem o tipo
//! já parseado (ou convertido pro tipo de domínio, no caso do ruleset).
//! Nesta fase é só leitura direta de caminho — resolução de
//! dependências entre pacotes, hot-reload e indexação no Content
//! Registry (arquitetura, seção 5) ficam pra quando existir mais de um
//! pacote de verdade pra resolver (Fase 4 em diante).

use std::fs;
use std::path::Path;

use crate::content_node::ContentNode;
use crate::error::LoaderError;
use crate::manifest::Manifest;
use crate::ruleset_file::RulesetFile;
use engine_core::Ruleset;

pub fn load_manifest(path: impl AsRef<Path>) -> Result<Manifest, LoaderError> {
    let raw = fs::read_to_string(path)?;
    let manifest: Manifest = serde_json::from_str(&raw)?;
    Ok(manifest)
}

pub fn load_ruleset(path: impl AsRef<Path>) -> Result<Ruleset, LoaderError> {
    let raw = fs::read_to_string(path)?;
    let file: RulesetFile = serde_json::from_str(&raw)?;
    Ok(file.into_ruleset())
}

pub fn load_content_node(path: impl AsRef<Path>) -> Result<ContentNode, LoaderError> {
    let raw = fs::read_to_string(path)?;
    let node: ContentNode = serde_json::from_str(&raw)?;
    Ok(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::PackType;
    use engine_core::{compute_attributes, Entity};
    use std::path::PathBuf;

    /// Caminho absoluto até `content-packs/dnd5e-core/`, resolvido a
    /// partir da raiz deste crate — funciona independente de qual seja
    /// o diretório de trabalho de quem rodou `cargo test`.
    fn pack_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../content-packs/dnd5e-core")
    }

    #[test]
    fn loads_manifest() {
        let manifest = load_manifest(pack_dir().join("manifest.json")).unwrap();
        assert_eq!(manifest.slug, "dnd5e-core");
        assert_eq!(manifest.pack_type, PackType::CoreSystem);
        assert_eq!(manifest.ruleset, "dnd5e");
        assert!(manifest.dependencies.is_empty());
    }

    #[test]
    fn loads_ruleset_and_computes_same_values_as_hand_written_engine_core_test() {
        let ruleset = load_ruleset(pack_dir().join("ruleset.json")).unwrap();

        let mut hero = Entity::new("hero-1", "dnd5e");
        hero.set_base("level", 5);
        hero.set_base("STR", 16);

        let computed = compute_attributes(&hero, &ruleset).unwrap();

        // Mesmos valores já validados na Fase 2, agora vindo de um
        // arquivo JSON de verdade em vez de código Rust hardcoded.
        assert_eq!(computed["str_mod"], 3);
        assert_eq!(computed["prof_bonus"], 3);
    }

    #[test]
    fn loads_item_content_node_and_applies_its_effect_end_to_end() {
        let ruleset = load_ruleset(pack_dir().join("ruleset.json")).unwrap();
        let gauntlets =
            load_content_node(pack_dir().join("items/gauntlets_of_ogre_power.json")).unwrap();

        assert_eq!(gauntlets.presentation.name, "Luvas do Poder do Ogro");
        assert_eq!(gauntlets.node_type, "item");

        let mut hero = Entity::new("hero-1", "dnd5e");
        hero.set_base("level", 5);
        hero.set_base("STR", 8); // força fraca antes do item

        for effect in gauntlets.effects().unwrap() {
            hero.add_effect(effect);
        }

        let computed = compute_attributes(&hero, &ruleset).unwrap();

        // O Effect do item usa operation "set" com value "19" — o
        // item literalmente fixa a Força em 19, ignorando o valor base.
        assert_eq!(computed["STR"], 19);
        assert_eq!(computed["str_mod"], 4); // (19-10)/2 = 4
    }

    #[test]
    fn rejects_unknown_operation_string() {
        use crate::content_node::EffectDef;

        let bad = EffectDef {
            target: "STR".to_string(),
            operation: "double".to_string(), // não existe
            value: "1".to_string(),
            duration: None,
            stacking: None,
        };

        let err = bad.into_effect("test-source").unwrap_err();
        assert!(matches!(err, LoaderError::UnknownOperation(op) if op == "double"));
    }
}
