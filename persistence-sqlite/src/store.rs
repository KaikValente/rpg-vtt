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
}
