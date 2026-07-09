use std::path::Path;

use engine_core::{Duration, Effect, Entity, Operation, Stacking};
use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::PersistenceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Campaign {
    pub id: String,
    pub name: String,
    pub ruleset_id: String,
}

impl Campaign {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        ruleset_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            ruleset_id: ruleset_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatEncounter {
    pub id: String,
    pub campaign_id: String,
    pub current_turn_index: usize,
    pub participants: Vec<CombatParticipant>,
}

impl CombatEncounter {
    pub fn new(
        id: impl Into<String>,
        campaign_id: impl Into<String>,
        participants: Vec<CombatParticipant>,
    ) -> Self {
        Self {
            id: id.into(),
            campaign_id: campaign_id.into(),
            current_turn_index: 0,
            participants,
        }
    }

    pub fn advance_turn(&mut self) {
        if self.participants.is_empty() {
            self.current_turn_index = 0;
            return;
        }

        self.current_turn_index = (self.current_turn_index + 1) % self.participants.len();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatParticipant {
    pub id: String,
    pub entity_id: Option<String>,
    pub name: String,
    pub initiative: i64,
}

impl CombatParticipant {
    pub fn new(
        id: impl Into<String>,
        entity_id: Option<String>,
        name: impl Into<String>,
        initiative: i64,
    ) -> Self {
        Self {
            id: id.into(),
            entity_id,
            name: name.into(),
            initiative,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapScene {
    pub id: String,
    pub campaign_id: String,
    pub name: String,
    pub width: i64,
    pub height: i64,
    pub tokens: Vec<MapToken>,
}

impl MapScene {
    pub fn new(
        id: impl Into<String>,
        campaign_id: impl Into<String>,
        name: impl Into<String>,
        width: i64,
        height: i64,
        tokens: Vec<MapToken>,
    ) -> Self {
        Self {
            id: id.into(),
            campaign_id: campaign_id.into(),
            name: name.into(),
            width,
            height,
            tokens,
        }
    }

    pub fn move_token(&mut self, token_id: &str, x: i64, y: i64) -> bool {
        let Some(token) = self.tokens.iter_mut().find(|token| token.id == token_id) else {
            return false;
        };

        token.x = x.clamp(0, self.width.saturating_sub(1));
        token.y = y.clamp(0, self.height.saturating_sub(1));
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapToken {
    pub id: String,
    pub participant_id: Option<String>,
    pub entity_id: Option<String>,
    pub name: String,
    pub x: i64,
    pub y: i64,
}

impl MapToken {
    pub fn new(
        id: impl Into<String>,
        participant_id: Option<String>,
        entity_id: Option<String>,
        name: impl Into<String>,
        x: i64,
        y: i64,
    ) -> Self {
        Self {
            id: id.into(),
            participant_id,
            entity_id,
            name: name.into(),
            x,
            y,
        }
    }
}

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, PersistenceError> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    pub fn in_memory() -> Result<Self, PersistenceError> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    pub fn init_schema(&self) -> Result<(), PersistenceError> {
        self.conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS campaigns (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                ruleset_id TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS entities (
                id TEXT PRIMARY KEY,
                campaign_id TEXT NOT NULL,
                ruleset_id TEXT NOT NULL,
                FOREIGN KEY (campaign_id) REFERENCES campaigns(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS entity_base_attributes (
                entity_id TEXT NOT NULL,
                attribute_id TEXT NOT NULL,
                value INTEGER NOT NULL,
                PRIMARY KEY (entity_id, attribute_id),
                FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS entity_effects (
                entity_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                target TEXT NOT NULL,
                operation TEXT NOT NULL,
                value TEXT NOT NULL,
                source TEXT NOT NULL,
                duration_kind TEXT NOT NULL,
                duration_rounds INTEGER,
                stacking TEXT NOT NULL,
                PRIMARY KEY (entity_id, position),
                FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS combat_encounters (
                id TEXT PRIMARY KEY,
                campaign_id TEXT NOT NULL,
                current_turn_index INTEGER NOT NULL,
                FOREIGN KEY (campaign_id) REFERENCES campaigns(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS combat_participants (
                encounter_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                id TEXT NOT NULL,
                entity_id TEXT,
                name TEXT NOT NULL,
                initiative INTEGER NOT NULL,
                PRIMARY KEY (encounter_id, position),
                FOREIGN KEY (encounter_id) REFERENCES combat_encounters(id) ON DELETE CASCADE,
                FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS map_scenes (
                id TEXT PRIMARY KEY,
                campaign_id TEXT NOT NULL,
                name TEXT NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                FOREIGN KEY (campaign_id) REFERENCES campaigns(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS map_tokens (
                scene_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                id TEXT NOT NULL,
                participant_id TEXT,
                entity_id TEXT,
                name TEXT NOT NULL,
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                PRIMARY KEY (scene_id, position),
                FOREIGN KEY (scene_id) REFERENCES map_scenes(id) ON DELETE CASCADE,
                FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE SET NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn save_campaign(&mut self, campaign: &Campaign) -> Result<(), PersistenceError> {
        let tx = self.conn.transaction()?;
        tx.execute(
            r#"
            INSERT INTO campaigns (id, name, ruleset_id)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                ruleset_id = excluded.ruleset_id
            "#,
            params![campaign.id, campaign.name, campaign.ruleset_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn load_campaign(&self, id: &str) -> Result<Option<Campaign>, PersistenceError> {
        let campaign = self
            .conn
            .query_row(
                "SELECT id, name, ruleset_id FROM campaigns WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Campaign {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        ruleset_id: row.get(2)?,
                    })
                },
            )
            .optional()?;
        Ok(campaign)
    }

    pub fn save_entity(
        &mut self,
        campaign_id: &str,
        entity: &Entity,
    ) -> Result<(), PersistenceError> {
        let tx = self.conn.transaction()?;

        tx.execute(
            r#"
            INSERT INTO entities (id, campaign_id, ruleset_id)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO UPDATE SET
                campaign_id = excluded.campaign_id,
                ruleset_id = excluded.ruleset_id
            "#,
            params![entity.id, campaign_id, entity.ruleset_id],
        )?;

        tx.execute(
            "DELETE FROM entity_base_attributes WHERE entity_id = ?1",
            params![entity.id],
        )?;
        for (attribute_id, value) in entity.base_attributes() {
            tx.execute(
                r#"
                INSERT INTO entity_base_attributes (entity_id, attribute_id, value)
                VALUES (?1, ?2, ?3)
                "#,
                params![entity.id, attribute_id, value],
            )?;
        }

        tx.execute(
            "DELETE FROM entity_effects WHERE entity_id = ?1",
            params![entity.id],
        )?;
        for (position, effect) in entity.effects().iter().enumerate() {
            insert_effect(&tx, &entity.id, position as i64, effect)?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn load_entity(&self, id: &str) -> Result<Option<Entity>, PersistenceError> {
        let Some((entity_id, ruleset_id)) = self
            .conn
            .query_row(
                "SELECT id, ruleset_id FROM entities WHERE id = ?1",
                params![id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?
        else {
            return Ok(None);
        };

        let mut entity = Entity::new(entity_id.clone(), ruleset_id);

        let mut base_stmt = self.conn.prepare(
            r#"
            SELECT attribute_id, value
            FROM entity_base_attributes
            WHERE entity_id = ?1
            ORDER BY attribute_id
            "#,
        )?;
        let base_rows = base_stmt.query_map(params![entity_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        for row in base_rows {
            let (attribute_id, value) = row?;
            entity.set_base(attribute_id, value);
        }

        let mut effects_stmt = self.conn.prepare(
            r#"
            SELECT target, operation, value, source, duration_kind, duration_rounds, stacking
            FROM entity_effects
            WHERE entity_id = ?1
            ORDER BY position
            "#,
        )?;
        let effect_rows = effects_stmt.query_map(params![entity_id], |row| {
            Ok(StoredEffect {
                target: row.get(0)?,
                operation: row.get(1)?,
                value: row.get(2)?,
                source: row.get(3)?,
                duration_kind: row.get(4)?,
                duration_rounds: row.get(5)?,
                stacking: row.get(6)?,
            })
        })?;
        for row in effect_rows {
            entity.add_effect(row?.into_effect()?);
        }

        Ok(Some(entity))
    }

    pub fn save_combat_encounter(
        &mut self,
        encounter: &CombatEncounter,
    ) -> Result<(), PersistenceError> {
        let tx = self.conn.transaction()?;
        tx.execute(
            r#"
            INSERT INTO combat_encounters (id, campaign_id, current_turn_index)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO UPDATE SET
                campaign_id = excluded.campaign_id,
                current_turn_index = excluded.current_turn_index
            "#,
            params![
                encounter.id,
                encounter.campaign_id,
                encounter.current_turn_index as i64
            ],
        )?;

        tx.execute(
            "DELETE FROM combat_participants WHERE encounter_id = ?1",
            params![encounter.id],
        )?;
        for (position, participant) in encounter.participants.iter().enumerate() {
            tx.execute(
                r#"
                INSERT INTO combat_participants (
                    encounter_id,
                    position,
                    id,
                    entity_id,
                    name,
                    initiative
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
                params![
                    encounter.id,
                    position as i64,
                    participant.id,
                    participant.entity_id,
                    participant.name,
                    participant.initiative,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn load_combat_encounter(
        &self,
        id: &str,
    ) -> Result<Option<CombatEncounter>, PersistenceError> {
        let Some((encounter_id, campaign_id, current_turn_index)) = self
            .conn
            .query_row(
                r#"
                SELECT id, campaign_id, current_turn_index
                FROM combat_encounters
                WHERE id = ?1
                "#,
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()?
        else {
            return Ok(None);
        };

        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, entity_id, name, initiative
            FROM combat_participants
            WHERE encounter_id = ?1
            ORDER BY position
            "#,
        )?;
        let rows = stmt.query_map(params![encounter_id], |row| {
            Ok(CombatParticipant {
                id: row.get(0)?,
                entity_id: row.get(1)?,
                name: row.get(2)?,
                initiative: row.get(3)?,
            })
        })?;

        let mut participants = Vec::new();
        for row in rows {
            participants.push(row?);
        }

        Ok(Some(CombatEncounter {
            id: encounter_id,
            campaign_id,
            current_turn_index: current_turn_index.max(0) as usize,
            participants,
        }))
    }

    pub fn load_campaign_combat(
        &self,
        campaign_id: &str,
    ) -> Result<Option<CombatEncounter>, PersistenceError> {
        let Some(encounter_id) = self
            .conn
            .query_row(
                r#"
                SELECT id
                FROM combat_encounters
                WHERE campaign_id = ?1
                ORDER BY id
                LIMIT 1
                "#,
                params![campaign_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
        else {
            return Ok(None);
        };

        self.load_combat_encounter(&encounter_id)
    }

    pub fn save_map_scene(&mut self, scene: &MapScene) -> Result<(), PersistenceError> {
        let tx = self.conn.transaction()?;
        tx.execute(
            r#"
            INSERT INTO map_scenes (id, campaign_id, name, width, height)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
                campaign_id = excluded.campaign_id,
                name = excluded.name,
                width = excluded.width,
                height = excluded.height
            "#,
            params![
                scene.id,
                scene.campaign_id,
                scene.name,
                scene.width,
                scene.height
            ],
        )?;

        tx.execute(
            "DELETE FROM map_tokens WHERE scene_id = ?1",
            params![scene.id],
        )?;
        for (position, token) in scene.tokens.iter().enumerate() {
            tx.execute(
                r#"
                INSERT INTO map_tokens (
                    scene_id,
                    position,
                    id,
                    participant_id,
                    entity_id,
                    name,
                    x,
                    y
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
                params![
                    scene.id,
                    position as i64,
                    token.id,
                    token.participant_id,
                    token.entity_id,
                    token.name,
                    token.x,
                    token.y,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn load_map_scene(&self, id: &str) -> Result<Option<MapScene>, PersistenceError> {
        let Some((scene_id, campaign_id, name, width, height)) = self
            .conn
            .query_row(
                r#"
                SELECT id, campaign_id, name, width, height
                FROM map_scenes
                WHERE id = ?1
                "#,
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                },
            )
            .optional()?
        else {
            return Ok(None);
        };

        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, participant_id, entity_id, name, x, y
            FROM map_tokens
            WHERE scene_id = ?1
            ORDER BY position
            "#,
        )?;
        let rows = stmt.query_map(params![scene_id], |row| {
            Ok(MapToken {
                id: row.get(0)?,
                participant_id: row.get(1)?,
                entity_id: row.get(2)?,
                name: row.get(3)?,
                x: row.get(4)?,
                y: row.get(5)?,
            })
        })?;

        let mut tokens = Vec::new();
        for row in rows {
            tokens.push(row?);
        }

        Ok(Some(MapScene {
            id: scene_id,
            campaign_id,
            name,
            width,
            height,
            tokens,
        }))
    }

    pub fn load_campaign_map_scene(
        &self,
        campaign_id: &str,
    ) -> Result<Option<MapScene>, PersistenceError> {
        let Some(scene_id) = self
            .conn
            .query_row(
                r#"
                SELECT id
                FROM map_scenes
                WHERE campaign_id = ?1
                ORDER BY id
                LIMIT 1
                "#,
                params![campaign_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
        else {
            return Ok(None);
        };

        self.load_map_scene(&scene_id)
    }
}

fn insert_effect(
    tx: &Transaction<'_>,
    entity_id: &str,
    position: i64,
    effect: &Effect,
) -> Result<(), PersistenceError> {
    let (duration_kind, duration_rounds) = duration_to_storage(&effect.duration);
    tx.execute(
        r#"
        INSERT INTO entity_effects (
            entity_id,
            position,
            target,
            operation,
            value,
            source,
            duration_kind,
            duration_rounds,
            stacking
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
        params![
            entity_id,
            position,
            effect.target,
            operation_to_storage(effect.operation),
            effect.value,
            effect.source,
            duration_kind,
            duration_rounds,
            stacking_to_storage(effect.stacking),
        ],
    )?;
    Ok(())
}

struct StoredEffect {
    target: String,
    operation: String,
    value: String,
    source: String,
    duration_kind: String,
    duration_rounds: Option<i64>,
    stacking: String,
}

impl StoredEffect {
    fn into_effect(self) -> Result<Effect, PersistenceError> {
        Ok(Effect::new(
            self.target,
            operation_from_storage(&self.operation)?,
            self.value,
            self.source,
        )
        .with_duration(duration_from_storage(
            &self.duration_kind,
            self.duration_rounds,
        )?)
        .with_stacking(stacking_from_storage(&self.stacking)?))
    }
}

fn operation_to_storage(operation: Operation) -> &'static str {
    match operation {
        Operation::Add => "add",
        Operation::Multiply => "multiply",
        Operation::Set => "set",
    }
}

fn operation_from_storage(value: &str) -> Result<Operation, PersistenceError> {
    match value {
        "add" => Ok(Operation::Add),
        "multiply" => Ok(Operation::Multiply),
        "set" => Ok(Operation::Set),
        other => Err(PersistenceError::InvalidOperation(other.to_string())),
    }
}

fn duration_to_storage(duration: &Duration) -> (&'static str, Option<i64>) {
    match duration {
        Duration::Permanent => ("permanent", None),
        Duration::UntilUnequipped => ("until_unequipped", None),
        Duration::Rounds(rounds) => ("rounds", Some(i64::from(*rounds))),
        Duration::Concentration => ("concentration", None),
    }
}

fn duration_from_storage(kind: &str, rounds: Option<i64>) -> Result<Duration, PersistenceError> {
    match kind {
        "permanent" => Ok(Duration::Permanent),
        "until_unequipped" => Ok(Duration::UntilUnequipped),
        "rounds" => {
            let rounds = rounds.ok_or(PersistenceError::MissingDurationRounds)?;
            Ok(Duration::Rounds(rounds as u32))
        }
        "concentration" => Ok(Duration::Concentration),
        other => Err(PersistenceError::InvalidDuration(other.to_string())),
    }
}

fn stacking_to_storage(stacking: Stacking) -> &'static str {
    match stacking {
        Stacking::Stack => "stack",
        Stacking::NoStack => "no_stack",
        Stacking::HighestWins => "highest_wins",
    }
}

fn stacking_from_storage(value: &str) -> Result<Stacking, PersistenceError> {
    match value {
        "stack" => Ok(Stacking::Stack),
        "no_stack" => Ok(Stacking::NoStack),
        "highest_wins" => Ok(Stacking::HighestWins),
        other => Err(PersistenceError::InvalidStacking(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_loads_campaign() {
        let mut store = SqliteStore::in_memory().unwrap();
        let campaign = Campaign::new("campaign-1", "Mesa de Quinta", "dnd5e");

        store.save_campaign(&campaign).unwrap();
        let loaded = store.load_campaign("campaign-1").unwrap().unwrap();

        assert_eq!(loaded, campaign);
    }

    #[test]
    fn saves_and_loads_entity_canonical_state() {
        let mut store = SqliteStore::in_memory().unwrap();
        store
            .save_campaign(&Campaign::new("campaign-1", "Mesa de Quinta", "dnd5e"))
            .unwrap();

        let mut entity = Entity::new("hero-1", "dnd5e");
        entity.set_base("level", 1);
        entity.set_base("STR", 8);
        entity.add_effect(
            Effect::new("STR", Operation::Set, "19", "gauntlets")
                .with_duration(Duration::UntilUnequipped)
                .with_stacking(Stacking::HighestWins),
        );
        entity.add_effect(
            Effect::new("AC", Operation::Add, "5", "shield")
                .with_duration(Duration::Rounds(1))
                .with_stacking(Stacking::NoStack),
        );

        store.save_entity("campaign-1", &entity).unwrap();
        let loaded = store.load_entity("hero-1").unwrap().unwrap();

        assert_eq!(loaded.id, "hero-1");
        assert_eq!(loaded.ruleset_id, "dnd5e");
        assert_eq!(loaded.base("level"), Some(1));
        assert_eq!(loaded.base("STR"), Some(8));
        assert_eq!(loaded.effects(), entity.effects());
    }

    #[test]
    fn replaces_entity_base_attributes_and_effects_on_resave() {
        let mut store = SqliteStore::in_memory().unwrap();
        store
            .save_campaign(&Campaign::new("campaign-1", "Mesa de Quinta", "dnd5e"))
            .unwrap();

        let mut entity = Entity::new("hero-1", "dnd5e");
        entity.set_base("STR", 8);
        entity.add_effect(Effect::new("STR", Operation::Add, "1", "human"));
        store.save_entity("campaign-1", &entity).unwrap();

        let mut updated = Entity::new("hero-1", "dnd5e");
        updated.set_base("DEX", 14);
        store.save_entity("campaign-1", &updated).unwrap();

        let loaded = store.load_entity("hero-1").unwrap().unwrap();
        assert_eq!(loaded.base("STR"), None);
        assert_eq!(loaded.base("DEX"), Some(14));
        assert!(loaded.effects().is_empty());
    }

    #[test]
    fn saves_loads_and_advances_basic_combat() {
        let mut store = SqliteStore::in_memory().unwrap();
        store
            .save_campaign(&Campaign::new("campaign-1", "Mesa de Quinta", "dnd5e"))
            .unwrap();

        let mut entity = Entity::new("hero-1", "dnd5e");
        entity.set_base("DEX", 14);
        store.save_entity("campaign-1", &entity).unwrap();

        let mut encounter = CombatEncounter::new(
            "combat-1",
            "campaign-1",
            vec![
                CombatParticipant::new("hero-1-combat", Some("hero-1".to_string()), "Arannis", 15),
                CombatParticipant::new("goblin-1", None, "Goblin de treino", 12),
            ],
        );
        encounter.advance_turn();

        store.save_combat_encounter(&encounter).unwrap();
        let loaded = store
            .load_campaign_combat("campaign-1")
            .unwrap()
            .expect("combat should be saved");

        assert_eq!(loaded.id, "combat-1");
        assert_eq!(loaded.current_turn_index, 1);
        assert_eq!(loaded.participants, encounter.participants);
    }

    #[test]
    fn saves_loads_and_moves_basic_map_scene() {
        let mut store = SqliteStore::in_memory().unwrap();
        store
            .save_campaign(&Campaign::new("campaign-1", "Mesa de Quinta", "dnd5e"))
            .unwrap();

        let mut entity = Entity::new("hero-1", "dnd5e");
        entity.set_base("DEX", 14);
        store.save_entity("campaign-1", &entity).unwrap();

        let mut scene = MapScene::new(
            "scene-1",
            "campaign-1",
            "Sala inicial",
            8,
            6,
            vec![
                MapToken::new(
                    "hero-token",
                    Some("hero-combat".to_string()),
                    Some("hero-1".to_string()),
                    "Arannis",
                    1,
                    1,
                ),
                MapToken::new(
                    "goblin-token",
                    Some("goblin-1".to_string()),
                    None,
                    "Goblin",
                    6,
                    4,
                ),
            ],
        );

        assert!(scene.move_token("goblin-token", 20, -3));
        store.save_map_scene(&scene).unwrap();
        let loaded = store
            .load_campaign_map_scene("campaign-1")
            .unwrap()
            .expect("scene should be saved");

        assert_eq!(loaded.id, "scene-1");
        assert_eq!(loaded.width, 8);
        assert_eq!(loaded.height, 6);
        assert_eq!(loaded.tokens.len(), 2);
        assert_eq!(loaded.tokens[1].x, 7);
        assert_eq!(loaded.tokens[1].y, 0);
    }
}
