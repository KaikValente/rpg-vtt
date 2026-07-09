//! Abstração sobre a fonte de aleatoriedade.
//!
//! O evaluator depende da trait `Roller`, não de `rand` diretamente. Isso
//! permite testar o evaluator com uma sequência de rolagens fixa (útil pra
//! testar vantagem/desvantagem/keep-highest sem depender de sorte no
//! teste), e troca a implementação real por outra fonte de aleatoriedade
//! no futuro sem tocar no evaluator.

use rand::Rng;

pub trait Roller {
    /// Rola um dado de `sides` lados, retornando um valor em `1..=sides`.
    fn roll(&mut self, sides: u32) -> u32;
}

/// Implementação real, usada em produção.
pub struct RandRoller {
    rng: rand::rngs::ThreadRng,
}

impl RandRoller {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }
}

impl Default for RandRoller {
    fn default() -> Self {
        Self::new()
    }
}

impl Roller for RandRoller {
    fn roll(&mut self, sides: u32) -> u32 {
        self.rng.gen_range(1..=sides)
    }
}

/// Implementação determinística para testes: devolve os valores de uma
/// fila fixa, em ordem, ignorando o parâmetro `sides` (o teste já sabe
/// o que está pedindo).
#[cfg(test)]
pub struct FixedRoller {
    values: std::collections::VecDeque<u32>,
}

#[cfg(test)]
impl FixedRoller {
    pub fn new(values: Vec<u32>) -> Self {
        Self {
            values: values.into(),
        }
    }
}

#[cfg(test)]
impl Roller for FixedRoller {
    fn roll(&mut self, _sides: u32) -> u32 {
        self.values
            .pop_front()
            .expect("FixedRoller ficou sem valores — teste pediu mais rolagens do que forneceu")
    }
}
