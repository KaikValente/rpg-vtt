//! # engine-core
//!
//! Domain Layer do RPG Engine: `Entity`, `AttributeDefinition`,
//! `DerivedRule`, `Effect` — tudo agnóstico de sistema de RPG. O core
//! nunca sabe o que é "Força" ou "PV"; só sabe manipular atributos,
//! fórmulas e efeitos genéricos declarados por um `Ruleset`.
//!
//! Depende do `dice-engine` (crate irmão) pra avaliar toda fórmula —
//! seja de uma `DerivedRule` (calcula) ou de um `Effect` (modifica).
//!
//! ## Exemplo
//! ```
//! use engine_core::{AttributeDefinition, DerivedRule, Entity, Ruleset, compute_attributes};
//!
//! let ruleset = Ruleset {
//!     attributes: vec![
//!         AttributeDefinition::new("level", "Nível", 1),
//!         AttributeDefinition::new("STR", "Força", 10),
//!     ],
//!     derived_rules: vec![
//!         DerivedRule::new("str_mod", "(STR-10)/2"),
//!         DerivedRule::new("prof_bonus", "2 + (level-1)/4"),
//!     ],
//! };
//!
//! let mut hero = Entity::new("hero-1", "dnd5e");
//! hero.set_base("level", 5);
//! hero.set_base("STR", 16);
//!
//! let computed = compute_attributes(&hero, &ruleset).unwrap();
//! assert_eq!(computed["str_mod"], 3);
//! assert_eq!(computed["prof_bonus"], 3);
//! ```

mod attribute;
mod effect;
mod engine;
mod entity;
mod error;
mod rule;

pub use attribute::AttributeDefinition;
pub use effect::{Duration, Effect, Operation, Stacking};
pub use engine::{compute_attributes, Ruleset};
pub use entity::Entity;
pub use error::EngineError;
pub use rule::DerivedRule;
