//! Teste de integração da fatia vertical da Fase 4: monta um Humano Mago
//! nível 1 inteiro a partir de arquivos de conteúdo reais — raça,
//! feature, classe, 4 magias, 2 itens — e confirma que o resultado final
//! da ficha bate com o cálculo esperado do D&D 5e.
//!
//! É um teste de integração (não `#[cfg(test)]` dentro do crate) de
//! propósito: usa só a API pública do `content-loader`, do jeito que
//! qualquer código de fora (a futura Fase 6, UI de ficha) vai usar.

use content_loader::{load_content_node, load_ruleset};
use engine_core::{compute_attributes, Entity};
use std::path::PathBuf;

fn pack_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../content-packs/dnd5e-core")
}

#[test]
fn builds_and_computes_a_level_1_human_wizard() {
    let ruleset = load_ruleset(pack_dir().join("ruleset.json")).unwrap();

    let race = load_content_node(pack_dir().join("races/human.json")).unwrap();
    let race_data = race.race_data().unwrap();
    assert_eq!(race_data.speed, 30);
    assert_eq!(race_data.traits.len(), 1);

    // Resolução manual do trait por id (Content Registry ainda não
    // existe) — confirma que a referência do race bate com o arquivo
    // real que carregamos.
    let trait_node =
        load_content_node(pack_dir().join("features/human_ability_score_increase.json")).unwrap();
    assert_eq!(trait_node.id, race_data.traits[0]);
    let _feature_data = trait_node.feature_data().unwrap();

    let class = load_content_node(pack_dir().join("classes/wizard.json")).unwrap();
    let class_data = class.class_data().unwrap();
    assert_eq!(class_data.hit_dice, "d6");
    assert_eq!(class_data.primary_attribute, "INT");

    // As 4 magias — nesta fase só validamos que carregam e que o
    // `spell_data()` tipado bate com o que cada uma deveria ser.
    // Nenhuma mecânica de conjuração é exercitada ainda (isso é Fase 7).
    let fire_bolt = load_content_node(pack_dir().join("spells/fire_bolt.json")).unwrap();
    let fire_bolt_data = fire_bolt.spell_data().unwrap();
    assert_eq!(fire_bolt_data.level, 0);
    assert_eq!(fire_bolt_data.damage_formula.as_deref(), Some("1d10"));

    let mage_hand = load_content_node(pack_dir().join("spells/mage_hand.json")).unwrap();
    assert_eq!(mage_hand.spell_data().unwrap().level, 0);

    let magic_missile = load_content_node(pack_dir().join("spells/magic_missile.json")).unwrap();
    let magic_missile_data = magic_missile.spell_data().unwrap();
    assert_eq!(magic_missile_data.level, 1);
    assert_eq!(magic_missile_data.damage_formula.as_deref(), Some("3d4+3"));

    let shield = load_content_node(pack_dir().join("spells/shield.json")).unwrap();
    assert_eq!(shield.spell_data().unwrap().level, 1);

    // Os 2 itens — idem, só validando que carregam com os dados certos.
    let dagger = load_content_node(pack_dir().join("items/dagger.json")).unwrap();
    let dagger_data = dagger.item_data().unwrap();
    assert_eq!(dagger_data.item_type, "weapon");
    let weapon = dagger_data.weapon.expect("adaga deveria ter dados de arma");
    assert_eq!(weapon.damage_formula, "1d4");

    let component_pouch = load_content_node(pack_dir().join("items/component_pouch.json")).unwrap();
    assert!(component_pouch.item_data().unwrap().weapon.is_none());

    // Chamar `item_data()` num node que não é item deve dar erro de tipo,
    // não pânico silencioso — confirma que `check_type` funciona.
    assert!(fire_bolt.item_data().is_err());

    // --- Monta a ficha de verdade ---
    // Array de atributos clássico de Mago: INT alto, CON média, STR baixa.
    let mut hero = Entity::new("hero-human-wizard-1", "dnd5e");
    hero.set_base("level", 1);
    hero.set_base("STR", 8);
    hero.set_base("DEX", 14);
    hero.set_base("CON", 14);
    hero.set_base("INT", 15);
    hero.set_base("WIS", 12);
    hero.set_base("CHA", 10);

    // ORDEM IMPORTA: os Effects da raça (que sobem os atributos base)
    // precisam ser adicionados ANTES do Effect de PV da classe, porque
    // o Effect de PV lê CON diretamente do contexto no momento em que é
    // avaliado (Effects rodam antes das DerivedRules — ver nota no
    // README do engine-core). Se a ordem fosse invertida, o PV usaria
    // CON=14 em vez de CON=15.
    for effect in trait_node.effects().unwrap() {
        hero.add_effect(effect);
    }
    for effect in class.effects().unwrap() {
        hero.add_effect(effect);
    }

    let computed = compute_attributes(&hero, &ruleset).unwrap();

    // +1 de Humano em tudo:
    assert_eq!(computed["STR"], 9);
    assert_eq!(computed["DEX"], 15);
    assert_eq!(computed["CON"], 15);
    assert_eq!(computed["INT"], 16);
    assert_eq!(computed["WIS"], 13);
    assert_eq!(computed["CHA"], 11);

    // DerivedRules calculadas em cima dos atributos já modificados:
    assert_eq!(computed["int_mod"], 3); // (16-10)/2
    assert_eq!(computed["con_mod"], 2); // (15-10)/2
    assert_eq!(computed["prof_bonus"], 2); // 2 + (1-1)/4

    // PV: 6 (dado máximo do d6 no nível 1) + 0 (nível-1 vezes a média)
    // + 1*2 (modificador de CON já com o bônus racial) = 8.
    // Bate com a regra real do SRD pra um Mago humano nível 1 com CON 15.
    assert_eq!(computed["hp_max"], 8);
}
