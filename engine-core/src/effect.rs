//! `Effect`: modificação declarativa (não código) aplicada a um atributo
//! quando o ContentNode de origem está "ativo" numa Entity (equipado,
//! aprendido, escolhido). Ver `rule.rs` para a distinção com
//! `DerivedRule`.
//!
//! Campos alinhados com a especificação da arquitetura (seção 3.3):
//! `source`, `duration`, `stacking` — mas a resolução de conflito de
//! stacking (dois bônus da mesma fonte não acumulam, fontes diferentes
//! acumulam) **não está totalmente implementada neste MVP**. O motor de
//! recálculo (`engine.rs`) aplica os efeitos na ordem em que aparecem na
//! Entity, sem agrupar por `stacking` ainda — essa regra fica pra
//! quando o SRD real (Fase 4) mostrar os casos concretos que precisam
//! dela, em vez de especular agora.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Add,
    Multiply,
    Set,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Duration {
    #[default]
    Permanent,
    UntilUnequipped,
    Rounds(u32),
    Concentration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Stacking {
    #[default]
    Stack,
    NoStack,
    HighestWins,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Effect {
    pub target: String,
    pub operation: Operation,
    /// Fórmula avaliada pelo dice-engine, ex: "level * con_mod" ou "2".
    /// Nota: evite fórmulas com dado (ex: "1d4") aqui — um Effect é
    /// recalculado toda vez que a ficha é recomputada, então um `1d4`
    /// dentro de um Effect gera um valor NOVO a cada recálculo, não um
    /// valor fixo. Isso não é bloqueado no código (o dice-engine aceita
    /// a fórmula normalmente), só documentado como cuidado de quem
    /// escreve o content-pack.
    pub value: String,
    pub source: String,
    pub duration: Duration,
    pub stacking: Stacking,
}

impl Effect {
    pub fn new(
        target: impl Into<String>,
        operation: Operation,
        value: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            target: target.into(),
            operation,
            value: value.into(),
            source: source.into(),
            duration: Duration::default(),
            stacking: Stacking::default(),
        }
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn with_stacking(mut self, stacking: Stacking) -> Self {
        self.stacking = stacking;
        self
    }
}
