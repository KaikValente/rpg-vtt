use std::fmt;

#[derive(Debug)]
pub enum LoaderError {
    Io(std::io::Error),
    Json(serde_json::Error),
    /// `operation` de um Effect no JSON que não é "add"/"multiply"/"set".
    UnknownOperation(String),
    /// `duration` de um Effect no JSON que não é um dos valores
    /// reconhecidos como string simples (ver nota em `content_node.rs`
    /// sobre `Duration::Rounds` ainda não ter formato de arquivo).
    UnknownDuration(String),
    /// `stacking` de um Effect no JSON que não é "stack"/"no_stack"/"highest_wins".
    UnknownStacking(String),
    /// Tentativa de interpretar `mechanics.data` de um ContentNode como
    /// um tipo que não bate com o `type` declarado (ex: chamar
    /// `race_data()` num node que é `type: "item"`).
    TypeMismatch {
        expected: String,
        actual: String,
    },
}

impl fmt::Display for LoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoaderError::Io(e) => write!(f, "erro de I/O ao ler pacote de conteúdo: {e}"),
            LoaderError::Json(e) => write!(f, "erro de JSON ao ler pacote de conteúdo: {e}"),
            LoaderError::UnknownOperation(op) => {
                write!(
                    f,
                    "operation desconhecida num Effect: '{op}' (esperado: add, multiply, set)"
                )
            }
            LoaderError::UnknownDuration(d) => {
                write!(f, "duration desconhecida num Effect: '{d}'")
            }
            LoaderError::UnknownStacking(s) => {
                write!(f, "stacking desconhecido num Effect: '{s}' (esperado: stack, no_stack, highest_wins)")
            }
            LoaderError::TypeMismatch { expected, actual } => {
                write!(
                    f,
                    "esperava ContentNode do tipo '{expected}', mas era '{actual}'"
                )
            }
        }
    }
}

impl std::error::Error for LoaderError {}

impl From<std::io::Error> for LoaderError {
    fn from(e: std::io::Error) -> Self {
        LoaderError::Io(e)
    }
}

impl From<serde_json::Error> for LoaderError {
    fn from(e: serde_json::Error) -> Self {
        LoaderError::Json(e)
    }
}
