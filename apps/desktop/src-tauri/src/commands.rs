use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::PathBuf;

use content_loader::{load_content_node, load_ruleset};
use engine_core::{compute_attributes, Entity};
use persistence_sqlite::{Campaign, CombatEncounter, CombatParticipant, SqliteStore};
use serde::Serialize;
use tauri::{AppHandle, Manager};

const DEFAULT_CAMPAIGN_ID: &str = "campaign-local";
const DEFAULT_CAMPAIGN_NAME: &str = "Mesa Local";
const DEFAULT_CHARACTER_ID: &str = "hero-human-wizard-1";
const DEFAULT_CHARACTER_NAME: &str = "Arannis";
const DEFAULT_COMBAT_ID: &str = "combat-local";
const DEFAULT_RULESET_ID: &str = "dnd5e";

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
#[serde(rename_all = "camelCase")]
pub struct CampaignSummary {
    id: String,
    name: String,
    ruleset_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignWorkspace {
    campaign: CampaignSummary,
    character: CharacterSheet,
    combat: Option<CombatSummary>,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatSummary {
    id: String,
    current_turn_index: usize,
    current_turn_participant_id: Option<String>,
    participants: Vec<CombatParticipantSummary>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatParticipantSummary {
    id: String,
    entity_id: Option<String>,
    name: String,
    initiative: i64,
    is_current_turn: bool,
}

#[tauri::command]
pub fn load_character_sheet(app: AppHandle) -> Result<CampaignWorkspace, String> {
    let db_path = app_database_path(&app).map_err(|error| error.to_string())?;
    let mut store = SqliteStore::open(db_path).map_err(|error| error.to_string())?;
    build_campaign_workspace(&mut store).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn start_basic_combat(app: AppHandle) -> Result<CampaignWorkspace, String> {
    let db_path = app_database_path(&app).map_err(|error| error.to_string())?;
    let mut store = SqliteStore::open(db_path).map_err(|error| error.to_string())?;
    ensure_workspace_seed(&mut store).map_err(|error| error.to_string())?;
    start_default_combat(&mut store).map_err(|error| error.to_string())?;
    build_campaign_workspace(&mut store).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn advance_combat_turn(app: AppHandle) -> Result<CampaignWorkspace, String> {
    let db_path = app_database_path(&app).map_err(|error| error.to_string())?;
    let mut store = SqliteStore::open(db_path).map_err(|error| error.to_string())?;
    ensure_workspace_seed(&mut store).map_err(|error| error.to_string())?;

    if store
        .load_combat_encounter(DEFAULT_COMBAT_ID)
        .map_err(|error| error.to_string())?
        .is_none()
    {
        start_default_combat(&mut store).map_err(|error| error.to_string())?;
    }

    let mut combat = store
        .load_combat_encounter(DEFAULT_COMBAT_ID)
        .map_err(|error| error.to_string())?
        .ok_or("default combat was not saved")?;
    combat.advance_turn();
    store
        .save_combat_encounter(&combat)
        .map_err(|error| error.to_string())?;

    build_campaign_workspace(&mut store).map_err(|error| error.to_string())
}

fn app_database_path(app: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data_dir)?;
    Ok(app_data_dir.join("rpg-engine.sqlite3"))
}

fn ensure_workspace_seed(store: &mut SqliteStore) -> Result<(), Box<dyn std::error::Error>> {
    let pack_dir = pack_dir();
    let trait_node =
        load_content_node(pack_dir.join("features/human_ability_score_increase.json"))?;
    let class = load_content_node(pack_dir.join("classes/wizard.json"))?;
    let campaign = ensure_default_campaign(store)?;
    ensure_default_character(store, &campaign, &trait_node, &class)?;
    Ok(())
}

fn build_campaign_workspace(
    store: &mut SqliteStore,
) -> Result<CampaignWorkspace, Box<dyn std::error::Error>> {
    let pack_dir = pack_dir();
    let ruleset = load_ruleset(pack_dir.join("ruleset.json"))?;

    let race = load_content_node(pack_dir.join("races/human.json"))?;
    let _race_data = race.race_data()?;
    let trait_node =
        load_content_node(pack_dir.join("features/human_ability_score_increase.json"))?;

    let class = load_content_node(pack_dir.join("classes/wizard.json"))?;
    let _class_data = class.class_data()?;

    let campaign = ensure_default_campaign(store)?;
    ensure_default_character(store, &campaign, &trait_node, &class)?;
    let hero = store
        .load_entity(DEFAULT_CHARACTER_ID)?
        .ok_or("default character was not saved")?;

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

    let combat = store
        .load_campaign_combat(&campaign.id)?
        .map(combat_summary);

    Ok(CampaignWorkspace {
        campaign: CampaignSummary {
            id: campaign.id,
            name: campaign.name,
            ruleset_id: campaign.ruleset_id,
        },
        character: CharacterSheet {
            id: hero.id,
            name: DEFAULT_CHARACTER_NAME.to_string(),
            ruleset_id: DEFAULT_RULESET_ID.to_string(),
            race: race.presentation.name,
            class_name: class.presentation.name,
            level: computed["level"],
            hp_max: computed["hp_max"],
            proficiency_bonus: computed["prof_bonus"],
            ability_scores,
            spells,
            items,
        },
        combat,
    })
}

fn ensure_default_campaign(
    store: &mut SqliteStore,
) -> Result<Campaign, Box<dyn std::error::Error>> {
    if let Some(campaign) = store.load_campaign(DEFAULT_CAMPAIGN_ID)? {
        return Ok(campaign);
    }

    let campaign = Campaign::new(
        DEFAULT_CAMPAIGN_ID,
        DEFAULT_CAMPAIGN_NAME,
        DEFAULT_RULESET_ID,
    );
    store.save_campaign(&campaign)?;
    Ok(campaign)
}

fn ensure_default_character(
    store: &mut SqliteStore,
    campaign: &Campaign,
    trait_node: &content_loader::ContentNode,
    class: &content_loader::ContentNode,
) -> Result<(), Box<dyn std::error::Error>> {
    if store.load_entity(DEFAULT_CHARACTER_ID)?.is_some() {
        return Ok(());
    }

    let mut hero = Entity::new(DEFAULT_CHARACTER_ID, DEFAULT_RULESET_ID);
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

    store.save_entity(&campaign.id, &hero)?;
    Ok(())
}

fn start_default_combat(store: &mut SqliteStore) -> Result<(), Box<dyn std::error::Error>> {
    let campaign = ensure_default_campaign(store)?;
    let mut combat = CombatEncounter::new(
        DEFAULT_COMBAT_ID,
        campaign.id,
        vec![
            CombatParticipant::new(
                "hero-human-wizard-1-combat",
                Some(DEFAULT_CHARACTER_ID.to_string()),
                DEFAULT_CHARACTER_NAME,
                15,
            ),
            CombatParticipant::new("training-goblin-1", None, "Goblin de treino", 12),
        ],
    );
    combat
        .participants
        .sort_by_key(|participant| Reverse(participant.initiative));
    store.save_combat_encounter(&combat)?;
    Ok(())
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

fn combat_summary(combat: CombatEncounter) -> CombatSummary {
    let current_turn_participant_id = combat
        .participants
        .get(combat.current_turn_index)
        .map(|participant| participant.id.clone());
    let participants = combat
        .participants
        .into_iter()
        .enumerate()
        .map(|(index, participant)| CombatParticipantSummary {
            id: participant.id,
            entity_id: participant.entity_id,
            name: participant.name,
            initiative: participant.initiative,
            is_current_turn: index == combat.current_turn_index,
        })
        .collect();

    CombatSummary {
        id: combat.id,
        current_turn_index: combat.current_turn_index,
        current_turn_participant_id,
        participants,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_character_sheet_from_saved_campaign_state() {
        let mut store = SqliteStore::in_memory().unwrap();
        let workspace = build_campaign_workspace(&mut store).unwrap();
        let sheet = workspace.character;

        assert_eq!(workspace.campaign.id, DEFAULT_CAMPAIGN_ID);
        assert_eq!(workspace.campaign.ruleset_id, "dnd5e");
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
        assert!(workspace.combat.is_none());
    }

    #[test]
    fn keeps_existing_saved_character_base_values() {
        let mut store = SqliteStore::in_memory().unwrap();
        let pack_dir = pack_dir();
        let trait_node =
            load_content_node(pack_dir.join("features/human_ability_score_increase.json")).unwrap();
        let class = load_content_node(pack_dir.join("classes/wizard.json")).unwrap();
        let campaign = ensure_default_campaign(&mut store).unwrap();
        ensure_default_character(&mut store, &campaign, &trait_node, &class).unwrap();

        let mut edited = store.load_entity(DEFAULT_CHARACTER_ID).unwrap().unwrap();
        edited.set_base("INT", 17);
        store.save_entity(&campaign.id, &edited).unwrap();

        let workspace = build_campaign_workspace(&mut store).unwrap();
        let intelligence = workspace
            .character
            .ability_scores
            .iter()
            .find(|ability| ability.id == "INT")
            .unwrap();

        assert_eq!(intelligence.score, 18);
        assert_eq!(intelligence.modifier, 4);
    }

    #[test]
    fn starts_and_advances_basic_combat() {
        let mut store = SqliteStore::in_memory().unwrap();
        ensure_workspace_seed(&mut store).unwrap();

        start_default_combat(&mut store).unwrap();
        let workspace = build_campaign_workspace(&mut store).unwrap();
        let combat = workspace.combat.expect("combat should exist");

        assert_eq!(combat.id, DEFAULT_COMBAT_ID);
        assert_eq!(combat.current_turn_index, 0);
        assert_eq!(combat.participants.len(), 2);
        assert_eq!(combat.participants[0].name, DEFAULT_CHARACTER_NAME);
        assert!(combat.participants[0].is_current_turn);

        let mut saved = store
            .load_combat_encounter(DEFAULT_COMBAT_ID)
            .unwrap()
            .unwrap();
        saved.advance_turn();
        store.save_combat_encounter(&saved).unwrap();

        let workspace = build_campaign_workspace(&mut store).unwrap();
        let combat = workspace.combat.expect("combat should still exist");
        assert_eq!(combat.current_turn_index, 1);
        assert_eq!(
            combat.current_turn_participant_id.as_deref(),
            Some("training-goblin-1")
        );
    }
}
