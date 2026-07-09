use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use content_loader::{load_content_node, load_content_nodes_from_dir, load_ruleset, MonsterData};
use engine_core::{compute_attributes, Entity};
use persistence_sqlite::{
    Campaign, CombatEncounter, CombatParticipant, MapScene, MapToken, SqliteStore,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Manager};

const DEFAULT_CAMPAIGN_ID: &str = "campaign-local";
const DEFAULT_CAMPAIGN_NAME: &str = "Mesa Local";
const DEFAULT_CHARACTER_ID: &str = "hero-human-wizard-1";
const DEFAULT_CHARACTER_NAME: &str = "Arannis";
const DEFAULT_COMBAT_ID: &str = "combat-local";
const DEFAULT_MAP_SCENE_ID: &str = "scene-local";
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
    map: Option<MapSceneSummary>,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonsterSummary {
    id: String,
    name: String,
    description: String,
    size: String,
    creature_type: String,
    armor_class: i64,
    hit_points: i64,
    speed: String,
    challenge_rating: String,
    actions: Vec<MonsterActionSummary>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonsterActionSummary {
    name: String,
    attack_bonus: Option<i64>,
    damage_formula: Option<String>,
    damage_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomebrewMonsterDraft {
    name: String,
    description: String,
    size: String,
    creature_type: String,
    armor_class: i64,
    hit_points: i64,
    speed: i64,
    challenge_rating: String,
    str_score: i64,
    dex_score: i64,
    con_score: i64,
    int_score: i64,
    wis_score: i64,
    cha_score: i64,
    action_name: String,
    attack_bonus: Option<i64>,
    damage_formula: String,
    damage_type: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapSceneSummary {
    id: String,
    name: String,
    width: i64,
    height: i64,
    tokens: Vec<MapTokenSummary>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapTokenSummary {
    id: String,
    participant_id: Option<String>,
    entity_id: Option<String>,
    name: String,
    x: i64,
    y: i64,
}

#[tauri::command]
pub fn load_character_sheet(app: AppHandle) -> Result<CampaignWorkspace, String> {
    let db_path = app_database_path(&app).map_err(|error| error.to_string())?;
    let mut store = SqliteStore::open(db_path).map_err(|error| error.to_string())?;
    build_campaign_workspace(&mut store).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn load_bestiary(app: AppHandle) -> Result<Vec<MonsterSummary>, String> {
    let homebrew_dir = homebrew_monsters_dir(&app).map_err(|error| error.to_string())?;
    build_bestiary(Some(homebrew_dir)).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_homebrew_monster(
    app: AppHandle,
    draft: HomebrewMonsterDraft,
) -> Result<Vec<MonsterSummary>, String> {
    let homebrew_dir = homebrew_monsters_dir(&app).map_err(|error| error.to_string())?;
    save_homebrew_monster(&homebrew_dir, draft).map_err(|error| error.to_string())?;
    build_bestiary(Some(homebrew_dir)).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn load_basic_map(app: AppHandle) -> Result<CampaignWorkspace, String> {
    let db_path = app_database_path(&app).map_err(|error| error.to_string())?;
    let mut store = SqliteStore::open(db_path).map_err(|error| error.to_string())?;
    ensure_workspace_seed(&mut store).map_err(|error| error.to_string())?;
    ensure_default_map_scene(&mut store).map_err(|error| error.to_string())?;
    build_campaign_workspace(&mut store).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn move_map_token(
    app: AppHandle,
    token_id: String,
    x: i64,
    y: i64,
) -> Result<CampaignWorkspace, String> {
    let db_path = app_database_path(&app).map_err(|error| error.to_string())?;
    let mut store = SqliteStore::open(db_path).map_err(|error| error.to_string())?;
    ensure_workspace_seed(&mut store).map_err(|error| error.to_string())?;
    ensure_default_map_scene(&mut store).map_err(|error| error.to_string())?;

    let mut scene = store
        .load_map_scene(DEFAULT_MAP_SCENE_ID)
        .map_err(|error| error.to_string())?
        .ok_or("default map scene was not saved")?;
    if !scene.move_token(&token_id, x, y) {
        return Err(format!("map token not found: {token_id}"));
    }
    store
        .save_map_scene(&scene)
        .map_err(|error| error.to_string())?;

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
    // Validate the displayed content nodes early; mechanics are still applied
    // from the saved canonical Entity state below.
    race.race_data()?;
    let trait_node =
        load_content_node(pack_dir.join("features/human_ability_score_increase.json"))?;

    let class = load_content_node(pack_dir.join("classes/wizard.json"))?;
    class.class_data()?;

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
    let map = store
        .load_campaign_map_scene(&campaign.id)?
        .map(map_scene_summary);

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
        map,
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
    let goblin = load_default_goblin_participant()?;
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
            goblin,
        ],
    );
    combat
        .participants
        .sort_by_key(|participant| Reverse(participant.initiative));
    store.save_combat_encounter(&combat)?;
    Ok(())
}

fn load_default_goblin_participant() -> Result<CombatParticipant, Box<dyn std::error::Error>> {
    let node = load_content_node(pack_dir().join("monsters/goblin.json"))?;
    let data = node.monster_data()?;
    let initiative = passive_initiative(&data);

    Ok(CombatParticipant::new(
        "training-goblin-1",
        None,
        node.presentation.name,
        initiative,
    ))
}

fn passive_initiative(monster: &MonsterData) -> i64 {
    10 + ability_modifier(monster.ability_scores.dex_score)
}

fn ability_modifier(score: i64) -> i64 {
    (score - 10).div_euclid(2)
}

fn ensure_default_map_scene(store: &mut SqliteStore) -> Result<(), Box<dyn std::error::Error>> {
    let campaign = ensure_default_campaign(store)?;
    if store.load_map_scene(DEFAULT_MAP_SCENE_ID)?.is_some() {
        return Ok(());
    }
    if store.load_combat_encounter(DEFAULT_COMBAT_ID)?.is_none() {
        start_default_combat(store)?;
    }
    let combat = store
        .load_combat_encounter(DEFAULT_COMBAT_ID)?
        .ok_or("default combat was not saved")?;

    let tokens = combat
        .participants
        .into_iter()
        .enumerate()
        .map(|(index, participant)| {
            let (x, y) = match index {
                0 => (1, 3),
                1 => (8, 3),
                _ => (index as i64, 1),
            };
            MapToken::new(
                format!("{}-token", participant.id),
                Some(participant.id),
                participant.entity_id,
                participant.name,
                x,
                y,
            )
        })
        .collect();

    let scene = MapScene::new(
        DEFAULT_MAP_SCENE_ID,
        campaign.id,
        "Encontro inicial",
        10,
        6,
        tokens,
    );
    store.save_map_scene(&scene)?;
    Ok(())
}

fn pack_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../content-packs/dnd5e-core")
}

fn homebrew_monsters_dir(app: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(app.path().app_data_dir()?.join("homebrew/monsters"))
}

fn build_bestiary(
    homebrew_dir: Option<PathBuf>,
) -> Result<Vec<MonsterSummary>, Box<dyn std::error::Error>> {
    let mut nodes = load_monsters_from_dir(pack_dir().join("monsters"))?;
    if let Some(homebrew_dir) = homebrew_dir {
        nodes.extend(load_monsters_from_dir(homebrew_dir)?);
    }

    nodes
        .into_iter()
        .filter(|node| node.node_type == "monster")
        .map(|node| {
            let data = node.monster_data()?;
            Ok(MonsterSummary {
                id: node.id,
                name: node.presentation.name,
                description: node.presentation.description,
                size: data.size,
                creature_type: data.creature_type,
                armor_class: data.armor_class,
                hit_points: data.hit_points,
                speed: format!("{} ft.", data.speed),
                challenge_rating: data.challenge_rating,
                actions: data
                    .actions
                    .into_iter()
                    .map(|action| MonsterActionSummary {
                        name: action.name,
                        attack_bonus: action.attack_bonus,
                        damage_formula: action.damage_formula,
                        damage_type: action.damage_type,
                    })
                    .collect(),
            })
        })
        .collect()
}

fn load_monsters_from_dir(
    dir: PathBuf,
) -> Result<Vec<content_loader::ContentNode>, Box<dyn std::error::Error>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    Ok(load_content_nodes_from_dir(dir)?)
}

fn save_homebrew_monster(
    dir: &PathBuf,
    draft: HomebrewMonsterDraft,
) -> Result<(), Box<dyn std::error::Error>> {
    validate_homebrew_monster(&draft)?;
    std::fs::create_dir_all(dir)?;

    let timestamp = current_unix_timestamp()?;
    let slug = slugify(&draft.name);
    let node_id = format!("homebrew-monster-{slug}-{timestamp}");
    let path = dir.join(format!("{slug}-{timestamp}.json"));
    let node = json!({
        "id": node_id,
        "type": "monster",
        "metadata": {
            "pack_id": "local-homebrew",
            "version": "0.1.0",
            "author": "local",
            "created_at": timestamp.to_string(),
            "updated_at": timestamp.to_string(),
            "dependencies": []
        },
        "presentation": {
            "name": draft.name.trim(),
            "slug": slug,
            "description": draft.description.trim(),
            "tags": ["homebrew"],
            "icon_asset": null
        },
        "mechanics": {
            "data": {
                "size": draft.size.trim(),
                "creature_type": draft.creature_type.trim(),
                "alignment": "unaligned",
                "armor_class": draft.armor_class,
                "hit_points": draft.hit_points,
                "hit_dice": "1d8",
                "speed": draft.speed,
                "challenge_rating": draft.challenge_rating.trim(),
                "ability_scores": {
                    "STR": draft.str_score,
                    "DEX": draft.dex_score,
                    "CON": draft.con_score,
                    "INT": draft.int_score,
                    "WIS": draft.wis_score,
                    "CHA": draft.cha_score
                },
                "senses": [],
                "languages": [],
                "actions": [
                    {
                        "name": draft.action_name.trim(),
                        "description": "Acao homebrew simples.",
                        "attack_bonus": draft.attack_bonus,
                        "damage_formula": draft.damage_formula.trim(),
                        "damage_type": draft.damage_type.trim()
                    }
                ]
            },
            "effects": []
        }
    });

    let raw = serde_json::to_string_pretty(&node)?;
    std::fs::write(&path, raw)?;

    let saved = load_content_node(&path)?;
    saved.monster_data()?;
    Ok(())
}

fn validate_homebrew_monster(
    draft: &HomebrewMonsterDraft,
) -> Result<(), Box<dyn std::error::Error>> {
    let required = [
        ("nome", draft.name.trim()),
        ("tamanho", draft.size.trim()),
        ("tipo de criatura", draft.creature_type.trim()),
        ("ND", draft.challenge_rating.trim()),
        ("acao", draft.action_name.trim()),
        ("formula de dano", draft.damage_formula.trim()),
        ("tipo de dano", draft.damage_type.trim()),
    ];
    if let Some((field, _)) = required.iter().find(|(_, value)| value.is_empty()) {
        return Err(format!("campo obrigatorio vazio: {field}").into());
    }
    if draft.armor_class <= 0 || draft.hit_points <= 0 || draft.speed < 0 {
        return Err("CA/PV devem ser positivos e deslocamento nao pode ser negativo".into());
    }
    Ok(())
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_dash = false;
        } else if !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        "monster".to_string()
    } else {
        slug
    }
}

fn current_unix_timestamp() -> Result<u64, Box<dyn std::error::Error>> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
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

fn map_scene_summary(scene: MapScene) -> MapSceneSummary {
    MapSceneSummary {
        id: scene.id,
        name: scene.name,
        width: scene.width,
        height: scene.height,
        tokens: scene
            .tokens
            .into_iter()
            .map(|token| MapTokenSummary {
                id: token.id,
                participant_id: token.participant_id,
                entity_id: token.entity_id,
                name: token.name,
                x: token.x,
                y: token.y,
            })
            .collect(),
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
        assert!(workspace.map.is_none());
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
        assert_eq!(combat.participants[1].name, "Goblin");
        assert_eq!(combat.participants[1].initiative, 12);
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

    #[test]
    fn loads_bestiary_from_content_pack_monsters() {
        let monsters = build_bestiary(None).unwrap();

        assert_eq!(monsters.len(), 1);
        assert_eq!(monsters[0].name, "Goblin");
        assert_eq!(monsters[0].armor_class, 15);
        assert_eq!(monsters[0].hit_points, 7);
        assert_eq!(monsters[0].speed, "30 ft.");
        assert_eq!(monsters[0].actions.len(), 2);
    }

    #[test]
    fn saves_homebrew_monster_as_loadable_content_node() {
        let dir = std::env::temp_dir().join(format!(
            "rpg-vtt-homebrew-test-{}",
            current_unix_timestamp().unwrap()
        ));
        let draft = HomebrewMonsterDraft {
            name: "Rato Arcano".to_string(),
            description: "Um roedor imbuido de magia instavel.".to_string(),
            size: "Tiny".to_string(),
            creature_type: "beast".to_string(),
            armor_class: 12,
            hit_points: 3,
            speed: 30,
            challenge_rating: "0".to_string(),
            str_score: 2,
            dex_score: 14,
            con_score: 8,
            int_score: 3,
            wis_score: 10,
            cha_score: 4,
            action_name: "Mordida".to_string(),
            attack_bonus: Some(4),
            damage_formula: "1d4+2".to_string(),
            damage_type: "piercing".to_string(),
        };

        save_homebrew_monster(&dir, draft).unwrap();
        let monsters = build_bestiary(Some(dir.clone())).unwrap();
        let homebrew = monsters
            .iter()
            .find(|monster| monster.name == "Rato Arcano")
            .expect("homebrew monster should be listed");

        assert_eq!(homebrew.armor_class, 12);
        assert_eq!(homebrew.hit_points, 3);
        assert_eq!(homebrew.actions[0].name, "Mordida");

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn creates_and_moves_basic_map_tokens() {
        let mut store = SqliteStore::in_memory().unwrap();
        ensure_workspace_seed(&mut store).unwrap();
        ensure_default_map_scene(&mut store).unwrap();

        let workspace = build_campaign_workspace(&mut store).unwrap();
        let map = workspace.map.expect("map should exist");
        assert_eq!(map.id, DEFAULT_MAP_SCENE_ID);
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 6);
        assert_eq!(map.tokens.len(), 2);
        assert_eq!(map.tokens[0].name, DEFAULT_CHARACTER_NAME);

        let mut scene = store.load_map_scene(DEFAULT_MAP_SCENE_ID).unwrap().unwrap();
        assert!(scene.move_token(&map.tokens[0].id, 4, 2));
        store.save_map_scene(&scene).unwrap();

        let workspace = build_campaign_workspace(&mut store).unwrap();
        let moved = workspace
            .map
            .unwrap()
            .tokens
            .into_iter()
            .find(|token| token.name == DEFAULT_CHARACTER_NAME)
            .unwrap();
        assert_eq!(moved.x, 4);
        assert_eq!(moved.y, 2);
    }
}
