use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum EngineError {
    /// Uma DerivedRule referenciou um atributo que não existe nem como
    /// AttributeDefinition do Ruleset, nem como outra DerivedRule.
    UnknownAttribute(String),
    /// Ciclo detectado entre DerivedRules (ex: A depende de B, B depende
    /// de A). Lista os ids envolvidos no ciclo, na ordem em que foram
    /// encontrados — não necessariamente o ciclo completo mínimo, mas o
    /// suficiente pra você achar o problema no content-pack.
    CircularDependency(Vec<String>),
    /// Erro propagado do dice-engine ao parsear/avaliar a fórmula de uma
    /// DerivedRule ou de um Effect.
    Formula(dice_engine::DiceError),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::UnknownAttribute(id) => {
                write!(f, "atributo desconhecido referenciado numa fórmula: '{id}'")
            }
            EngineError::CircularDependency(ids) => {
                write!(f, "dependência circular entre DerivedRules: {}", ids.join(" -> "))
            }
            EngineError::Formula(e) => write!(f, "erro ao avaliar fórmula: {e}"),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<dice_engine::DiceError> for EngineError {
    fn from(e: dice_engine::DiceError) -> Self {
        EngineError::Formula(e)
    }
}
