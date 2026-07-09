use std::collections::HashMap;
use std::path::PathBuf;

use content_loader::{load_content_node, load_ruleset};
use engine_core::{compute_attributes, Entity};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterSheet {
    id: String,
    name: String,
    ruleset_id: String,
    race: String,
    class_name: String,
    level: i64,
    hp_max: i64,
    proficiency_bonus: i64,
    ability_scores: Vec<AbilityScore>,
    spells: Vec<SpellSummary>,
    items: Vec<ItemSummary>,
}

#[derive(Debug, Serialize)]
struct AbilityScore {
    id: String,
    label: String,
    score: i64,
    modifier: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SpellSummary {
    name: String,
    level: i64,
    school: String,
    damage_formula: Option<String>,
    damage_type: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemSummary {
    name: String,
    item_type: String,
    damage_formula: Option<String>,
    damage_type: Option<String>,
}

#[tauri::command]
pub fn load_character_sheet() -> Result<CharacterSheet, String> {
    build_character_sheet().map_err(|error| error.to_string())
}

fn build_character_sheet() -> Result<CharacterSheet, Box<dyn std::error::Error>> {
    let pack_dir = pack_dir();
    let ruleset = load_ruleset(pack_dir.join("ruleset.json"))?;

    let race = load_content_node(pack_dir.join("races/human.json"))?;
    let _race_data = race.race_data()?;
    let trait_node =
        load_content_node(pack_dir.join("features/human_ability_score_increase.json"))?;

    let class = load_content_node(pack_dir.join("classes/wizard.json"))?;
    let _class_data = class.class_data()?;

    let mut hero = Entity::new("hero-human-wizard-1", "dnd5e");
    hero.set_base("level", 1);
    hero.set_base("STR", 8);
    hero.set_base("DEX", 14);
    hero.set_base("CON", 14);
    hero.set_base("INT", 15);
    hero.set_base("WIS", 12);
    hero.set_base("CHA", 10);

    for effect in trait_node.effects()? {
        hero.add_effect(effect);
    }
    for effect in class.effects()? {
        hero.add_effect(effect);
    }

    let computed = compute_attributes(&hero, &ruleset)?;
    let ability_scores = ability_scores(&computed);

    let spells = [
        "spells/fire_bolt.json",
        "spells/mage_hand.json",
        "spells/magic_missile.json",
        "spells/shield.json",
    ]
    .into_iter()
    .map(|relative_path| {
        let node = load_content_node(pack_dir.join(relative_path))?;
        let data = node.spell_data()?;
        Ok(SpellSummary {
            name: node.presentation.name,
            level: data.level,
            school: data.school,
            damage_formula: data.damage_formula,
            damage_type: data.damage_type,
        })
    })
    .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

    let items = ["items/dagger.json", "items/component_pouch.json"]
        .into_iter()
        .map(|relative_path| {
            let node = load_content_node(pack_dir.join(relative_path))?;
            let data = node.item_data()?;
            let (damage_formula, damage_type) = data
                .weapon
                .map(|weapon| (Some(weapon.damage_formula), Some(weapon.damage_type)))
                .unwrap_or((None, None));
            Ok(ItemSummary {
                name: node.presentation.name,
                item_type: data.item_type,
                damage_formula,
                damage_type,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

    Ok(CharacterSheet {
        id: hero.id,
        name: "Arannis".to_string(),
        ruleset_id: "dnd5e".to_string(),
        race: race.presentation.name,
        class_name: class.presentation.name,
        level: computed["level"],
        hp_max: computed["hp_max"],
        proficiency_bonus: computed["prof_bonus"],
        ability_scores,
        spells,
        items,
    })
}

fn pack_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../content-packs/dnd5e-core")
}

fn ability_scores(computed: &HashMap<String, i64>) -> Vec<AbilityScore> {
    [
        ("STR", "Forca", "str_mod"),
        ("DEX", "Destreza", "dex_mod"),
        ("CON", "Constituicao", "con_mod"),
        ("INT", "Inteligencia", "int_mod"),
        ("WIS", "Sabedoria", "wis_mod"),
        ("CHA", "Carisma", "cha_mod"),
    ]
    .into_iter()
    .map(|(id, label, modifier_id)| AbilityScore {
        id: id.to_string(),
        label: label.to_string(),
        score: computed[id],
        modifier: computed[modifier_id],
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_character_sheet_from_content_pack() {
        let sheet = build_character_sheet().unwrap();

        assert_eq!(sheet.id, "hero-human-wizard-1");
        assert_eq!(sheet.ruleset_id, "dnd5e");
        assert_eq!(sheet.race, "Humano");
        assert_eq!(sheet.class_name, "Mago");
        assert_eq!(sheet.level, 1);
        assert_eq!(sheet.hp_max, 8);
        assert_eq!(sheet.proficiency_bonus, 2);
        assert_eq!(sheet.ability_scores.len(), 6);
        assert_eq!(sheet.spells.len(), 4);
        assert_eq!(sheet.items.len(), 2);
    }
}
