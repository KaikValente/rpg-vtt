//! # persistence-sqlite
//!
//! SQLite persistence layer for RPG Engine.
//!
//! This crate stores canonical state only: campaigns, entities, explicit base
//! attributes and active effects. Computed attributes stay out of the database
//! and are recalculated by `engine-core`.

mod error;
mod store;

pub use error::PersistenceError;
pub use store::{Campaign, SqliteStore};
