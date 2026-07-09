//! Evaluator: a única parte do crate que sabe rolar dados de verdade e
//! aplicar a `RollPolicy`. O AST (`Expr`) chega aqui sem nenhuma noção de
//! vantagem/crítico — essas regras são aplicadas *durante* o percurso da
//! árvore, não antes.
//!
//! Regra de vantagem/desvantagem: aplica-se apenas ao primeiro nó
//! `1d20` puro (count == 1, sides == 20) encontrado no percurso em
//! pré-ordem da árvore. Isso cobre o caso real do SRD — vantagem se
//! aplica ao d20 de um teste/ataque, nunca aos dados de dano de uma
//! fórmula composta. Um `RollPolicy` com vantagem aplicado a uma fórmula
//! sem nenhum `1d20` simplesmente não tem efeito (não é erro).
//!
//! Regra de crítico: dobra a *quantidade* de dados de cada nó `Dice`
//! encontrado (regra de dano de D&D 5e — dobra os dados, não o
//! modificador fixo). Numa fórmula de ataque (`1d20+STR`) isso não tem
//! efeito, porque não há dado de dano ali; crítico deve ser aplicado só
//! na avaliação da fórmula de dano, não na de ataque.
//!
//! Regra de divisão: `/` sempre arredonda pra **baixo** (`floor`), nunca
//! trunca em direção a zero. Essa é a convenção universal de D&D 5e
//! ("round down" aparece em praticamente toda regra que envolve
//! divisão — modificador de atributo, metade do nível, etc.), então faz
//! mais sentido `/` já se comportar assim por padrão do que introduzir
//! um segundo operador. Pra valores positivos, floor e truncamento dão
//! o mesmo resultado; a diferença só aparece com resultado intermediário
//! negativo (ex: `(9-10)/2` deve dar `-1`, não `0` — ver `floor_div`).

use crate::ast::{BinaryOp, DiceExpr, DiceModifier, Expr};
use crate::context::RollContext;
use crate::error::DiceError;
use crate::policy::{Advantage, RollPolicy};
use crate::result::{EvalNode, RollResult};
use crate::rng::Roller;

pub fn evaluate(
    expr: &Expr,
    ctx: &RollContext,
    policy: &RollPolicy,
    roller: &mut impl Roller,
) -> Result<RollResult, DiceError> {
    let mut advantage_applied = false;
    let root = eval_node(expr, ctx, policy, roller, &mut advantage_applied)?;
    let total = root.total();
    Ok(RollResult { total, root })
}

fn eval_node(
    expr: &Expr,
    ctx: &RollContext,
    policy: &RollPolicy,
    roller: &mut impl Roller,
    advantage_applied: &mut bool,
) -> Result<EvalNode, DiceError> {
    match expr {
        Expr::Number(n) => Ok(EvalNode::Number(*n)),

        Expr::Variable(name) => {
            let value = ctx
                .get(name)
                .ok_or_else(|| DiceError::UnknownVariable(name.clone()))?;
            Ok(EvalNode::Variable {
                name: name.clone(),
                value,
            })
        }

        Expr::Dice(dice) => Ok(eval_dice(dice, policy, roller, advantage_applied)),

        Expr::Binary { left, op, right } => {
            // Pré-ordem: avalia a esquerda primeiro, então a direita.
            // Isso é o que define "primeiro 1d20 encontrado" de forma
            // determinística e previsível para quem lê a fórmula da
            // esquerda para a direita.
            let left_node = eval_node(left, ctx, policy, roller, advantage_applied)?;
            let right_node = eval_node(right, ctx, policy, roller, advantage_applied)?;

            let l = left_node.total();
            let r = right_node.total();

            let total = match op {
                BinaryOp::Add => l + r,
                BinaryOp::Sub => l - r,
                BinaryOp::Mul => l * r,
                BinaryOp::Div => {
                    if r == 0 {
                        return Err(DiceError::DivisionByZero);
                    }
                    floor_div(l, r)
                }
            };

            Ok(EvalNode::Binary {
                op: *op,
                left: Box::new(left_node),
                right: Box::new(right_node),
                total,
            })
        }
    }
}

/// Divisão inteira arredondada pra baixo (`floor`), diferente da
/// divisão nativa do Rust (`/`), que trunca em direção a zero.
///
/// Pra dividendo e divisor positivos, dá exatamente o mesmo resultado
/// que `/`. A diferença só aparece quando o resultado matemático seria
/// negativo com resto: `-1 / 2` trunca pra `0` (Rust nativo), mas
/// `floor(-0.5)` é `-1` — é isso que `floor_div` corrige.
///
/// Fórmula padrão de correção sobre divisão truncada: se sobrou resto
/// (`r != 0`) e o sinal do resto diverge do sinal do divisor, o
/// quociente truncado "arredondou pra cima" em vez de pra baixo —
/// então subtrai 1 pra compensar.
fn floor_div(a: i64, b: i64) -> i64 {
    let q = a / b;
    let r = a % b;
    if r != 0 && (r < 0) != (b < 0) {
        q - 1
    } else {
        q
    }
}

fn eval_dice(
    dice: &DiceExpr,
    policy: &RollPolicy,
    roller: &mut impl Roller,
    advantage_applied: &mut bool,
) -> EvalNode {
    let is_primary_d20 = dice.count == 1 && dice.sides == 20;

    if is_primary_d20 && !*advantage_applied && policy.advantage != Advantage::None {
        *advantage_applied = true;

        let a = roller.roll(20) as i64;
        let b = roller.roll(20) as i64;
        let kept_value = match policy.advantage {
            Advantage::Advantage => a.max(b),
            Advantage::Disadvantage => a.min(b),
            Advantage::None => unreachable!("checado acima"),
        };

        return EvalNode::Dice {
            count: 1,
            sides: 20,
            modifier: DiceModifier::None,
            rolls: vec![a, b],
            kept: vec![kept_value],
            total: kept_value,
        };
    }

    // Crítico dobra a quantidade de dados (não o modificador fixo da
    // fórmula, que sequer existe neste nó — modificadores fixos são
    // representados como Expr::Number/Variable em outro ramo da árvore).
    let effective_count = if policy.critical {
        dice.count.saturating_mul(2)
    } else {
        dice.count
    };

    let rolls: Vec<i64> = (0..effective_count)
        .map(|_| roller.roll(dice.sides) as i64)
        .collect();

    let kept = apply_modifier(&rolls, &dice.modifier);
    let total: i64 = kept.iter().sum();

    EvalNode::Dice {
        count: dice.count,
        sides: dice.sides,
        modifier: dice.modifier.clone(),
        rolls,
        kept,
        total,
    }
}

fn apply_modifier(rolls: &[i64], modifier: &DiceModifier) -> Vec<i64> {
    match modifier {
        DiceModifier::None => rolls.to_vec(),
        DiceModifier::KeepHighest(n) => {
            let mut sorted = rolls.to_vec();
            sorted.sort_unstable_by(|a, b| b.cmp(a)); // desc
            let n = (*n as usize).min(sorted.len());
            sorted.into_iter().take(n).collect()
        }
        DiceModifier::KeepLowest(n) => {
            let mut sorted = rolls.to_vec();
            sorted.sort_unstable(); // asc
            let n = (*n as usize).min(sorted.len());
            sorted.into_iter().take(n).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::rng::FixedRoller;

    fn eval_str(
        formula: &str,
        ctx: &RollContext,
        policy: &RollPolicy,
        rolls: Vec<u32>,
    ) -> RollResult {
        let expr = Parser::parse(formula).unwrap();
        let mut roller = FixedRoller::new(rolls);
        evaluate(&expr, ctx, policy, &mut roller).unwrap()
    }

    #[test]
    fn evaluates_attack_roll() {
        let ctx = RollContext::new().with("STR", 3).with("PROF", 2);
        let result = eval_str("1d20+STR+PROF", &ctx, &RollPolicy::normal(), vec![14]);
        assert_eq!(result.total, 19); // 14 + 3 + 2
    }

    #[test]
    fn advantage_keeps_higher_of_two_d20() {
        let ctx = RollContext::new();
        let result = eval_str(
            "1d20",
            &ctx,
            &RollPolicy::with_advantage(),
            vec![14, 9], // rola 14 e 9, vantagem mantém 14
        );
        assert_eq!(result.total, 14);
        if let EvalNode::Dice { rolls, kept, .. } = result.root {
            assert_eq!(rolls, vec![14, 9]);
            assert_eq!(kept, vec![14]);
        } else {
            panic!("esperava EvalNode::Dice");
        }
    }

    #[test]
    fn disadvantage_keeps_lower_of_two_d20() {
        let ctx = RollContext::new();
        let result = eval_str("1d20", &ctx, &RollPolicy::with_disadvantage(), vec![14, 9]);
        assert_eq!(result.total, 9);
    }

    #[test]
    fn advantage_only_applies_to_first_d20_in_formula() {
        // Fórmula fictícia com dois d20 — só o primeiro deve consumir a
        // política de vantagem; o segundo rola normalmente (1 valor).
        let ctx = RollContext::new();
        let result = eval_str(
            "1d20+1d20",
            &ctx,
            &RollPolicy::with_advantage(),
            vec![14, 9, 5], // primeiro d20 rola 2x (vantagem), segundo rola 1x
        );
        // primeiro d20 com vantagem: max(14,9) = 14; segundo d20 normal: 5
        assert_eq!(result.total, 19);
    }

    #[test]
    fn critical_doubles_damage_dice_count() {
        let ctx = RollContext::new();
        let result = eval_str(
            "2d6",
            &ctx,
            &RollPolicy::normal().critical(),
            vec![3, 4, 5, 6], // 4 dados rolados (2 dobrado pra 4)
        );
        assert_eq!(result.total, 18); // 3+4+5+6
    }

    #[test]
    fn keep_highest_stat_roll() {
        let ctx = RollContext::new();
        let result = eval_str("4d6kh3", &ctx, &RollPolicy::normal(), vec![6, 6, 6, 1]);
        assert_eq!(result.total, 18); // mantém os 3 maiores: 6+6+6, descarta o 1
    }

    #[test]
    fn division_by_zero_is_an_error() {
        let ctx = RollContext::new();
        let expr = Parser::parse("10/0").unwrap();
        let mut roller = FixedRoller::new(vec![]);
        let err = evaluate(&expr, &ctx, &RollPolicy::normal(), &mut roller).unwrap_err();
        assert_eq!(err, DiceError::DivisionByZero);
    }

    #[test]
    fn division_rounds_down_not_toward_zero() {
        let ctx = RollContext::new();
        let mut roller = FixedRoller::new(vec![]);

        // Caso real que motivou a correção: modificador de STR 9 no
        // D&D 5e é floor((9-10)/2) = floor(-0.5) = -1, não 0.
        let expr = Parser::parse("(9-10)/2").unwrap();
        let result = evaluate(&expr, &ctx, &RollPolicy::normal(), &mut roller).unwrap();
        assert_eq!(result.total, -1);

        // Casos positivos continuam iguais a antes (floor == truncamento
        // quando não há resultado negativo).
        let expr = Parser::parse("5/2").unwrap();
        let result = evaluate(&expr, &ctx, &RollPolicy::normal(), &mut roller).unwrap();
        assert_eq!(result.total, 2);

        // Divisão exata, positiva ou negativa, não deve ser afetada.
        let expr = Parser::parse("-10/2").unwrap();
        let result = evaluate(&expr, &ctx, &RollPolicy::normal(), &mut roller).unwrap();
        assert_eq!(result.total, -5);
    }

    #[test]
    fn unknown_variable_is_an_error() {
        let ctx = RollContext::new();
        let expr = Parser::parse("STR").unwrap();
        let mut roller = FixedRoller::new(vec![]);
        let err = evaluate(&expr, &ctx, &RollPolicy::normal(), &mut roller).unwrap_err();
        assert_eq!(err, DiceError::UnknownVariable("STR".to_string()));
    }
}
