//! Um atributo declarado por um Ruleset (ex: "STR", "hp", "level"). O
//! core nunca sabe o que "STR" significa — só sabe que existe um
//! atributo com esse id e um valor padrão.

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeDefinition {
    pub id: String,
    pub label: String,
    pub default_value: i64,
}

impl AttributeDefinition {
    pub fn new(id: impl Into<String>, label: impl Into<String>, default_value: i64) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            default_value,
        }
    }
}
