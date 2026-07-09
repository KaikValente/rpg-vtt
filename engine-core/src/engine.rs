//! O motor de recálculo: dado uma `Entity` e um `Ruleset`, produz o mapa
//! final de atributos computados.
//!
//! Ordem de resolução (fixa — importa pra quem for depurar um valor
//! errado na ficha):
//! 1. Contexto inicial = `default_value` de cada `AttributeDefinition`
//!    do Ruleset, sobrescrito por qualquer valor base explícito da
//!    Entity (`Entity::set_base`).
//! 2. `Effect`s ativos na Entity são aplicados, na ordem em que foram
//!    adicionados (ver limitação de stacking documentada em `effect.rs`).
//! 3. `DerivedRule`s são resolvidas em ordem topológica — uma rule que
//!    depende de outra (ex: `attack_bonus` depende de `str_mod` e de
//!    `prof_bonus`) roda só depois que suas dependências já estão no
//!    contexto. Ciclos são detectados e retornam erro em vez de
//!    travar/estourar recursão.

use crate::attribute::AttributeDefinition;
use crate::effect::Operation;
use crate::entity::Entity;
use crate::error::EngineError;
use crate::rule::DerivedRule;

use dice_engine::{parse_formula, Expr, RollContext, RollPolicy};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Default)]
pub struct Ruleset {
    pub attributes: Vec<AttributeDefinition>,
    pub derived_rules: Vec<DerivedRule>,
}

pub fn compute_attributes(
    entity: &Entity,
    ruleset: &Ruleset,
) -> Result<HashMap<String, i64>, EngineError> {
    let mut context: HashMap<String, i64> = HashMap::new();

    // 1. Base: default do Ruleset, sobrescrito por override da Entity.
    for def in &ruleset.attributes {
        let value = entity.base(&def.id).unwrap_or(def.default_value);
        context.insert(def.id.clone(), value);
    }

    // 2. Effects, na ordem em que estão na Entity.
    for effect in entity.effects() {
        let formula_value = eval_formula_i64(&effect.value, &context)?;
        let current = *context.get(&effect.target).unwrap_or(&0);
        let new_value = match effect.operation {
            Operation::Add => current + formula_value,
            Operation::Multiply => current * formula_value,
            Operation::Set => formula_value,
        };
        context.insert(effect.target.clone(), new_value);
    }

    // 3. DerivedRules em ordem topológica.
    let order = topological_order(&ruleset.derived_rules)?;
    let rules_by_id: HashMap<&str, &DerivedRule> = ruleset
        .derived_rules
        .iter()
        .map(|r| (r.id.as_str(), r))
        .collect();

    for rule_id in order {
        let rule = rules_by_id[rule_id.as_str()];
        let value = eval_formula_i64(&rule.formula, &context)?;
        context.insert(rule.id.clone(), value);
    }

    Ok(context)
}

fn eval_formula_i64(formula: &str, context: &HashMap<String, i64>) -> Result<i64, EngineError> {
    let mut ctx = RollContext::new();
    for (k, v) in context {
        ctx.set(k.clone(), *v);
    }
    // Nota: usa RollPolicy::normal(). Fórmulas de DerivedRule/Effect
    // não deveriam conter dados (ver aviso em effect.rs) — se
    // contiverem, isso rola de verdade a cada recálculo.
    let result = dice_engine::roll(formula, &ctx, &RollPolicy::normal())?;
    Ok(result.total)
}

/// Ordena as DerivedRules via Kahn's algorithm, considerando só arestas
/// entre rules (dependência numa AttributeDefinition base não entra no
/// grafo — já está resolvida no contexto antes deste passo).
fn topological_order(rules: &[DerivedRule]) -> Result<Vec<String>, EngineError> {
    let rule_ids: HashSet<&str> = rules.iter().map(|r| r.id.as_str()).collect();

    let mut depends_on: HashMap<String, HashSet<String>> = HashMap::new();
    for rule in rules {
        let expr = parse_formula(&rule.formula)?;
        let mut vars = HashSet::new();
        collect_variable_names(&expr, &mut vars);
        let deps: HashSet<String> = vars
            .into_iter()
            .filter(|v| rule_ids.contains(v.as_str()) && v != &rule.id)
            .collect();
        depends_on.insert(rule.id.clone(), deps);
    }

    let mut in_degree: HashMap<String, usize> = rules
        .iter()
        .map(|r| (r.id.clone(), depends_on[&r.id].len()))
        .collect();

    let mut dependents: HashMap<String, Vec<String>> =
        rules.iter().map(|r| (r.id.clone(), Vec::new())).collect();
    for rule in rules {
        for dep in &depends_on[&rule.id] {
            dependents.get_mut(dep).unwrap().push(rule.id.clone());
        }
    }

    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(id, _)| id.clone())
        .collect();

    let mut order = Vec::new();
    while let Some(id) = queue.pop_front() {
        order.push(id.clone());
        if let Some(deps_list) = dependents.get(&id) {
            for dependent in deps_list {
                let deg = in_degree.get_mut(dependent).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(dependent.clone());
                }
            }
        }
    }

    if order.len() != rules.len() {
        let remaining: Vec<String> = rules
            .iter()
            .map(|r| r.id.clone())
            .filter(|id| !order.contains(id))
            .collect();
        return Err(EngineError::CircularDependency(remaining));
    }

    Ok(order)
}

fn collect_variable_names(expr: &Expr, out: &mut HashSet<String>) {
    match expr {
        Expr::Number(_) => {}
        Expr::Variable(name) => {
            out.insert(name.clone());
        }
        Expr::Dice(_) => {}
        Expr::Binary { left, right, .. } => {
            collect_variable_names(left, out);
            collect_variable_names(right, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::Effect;

    fn dnd_like_ruleset() -> Ruleset {
        Ruleset {
            attributes: vec![
                AttributeDefinition::new("level", "Nível", 1),
                AttributeDefinition::new("STR", "Força", 10),
                AttributeDefinition::new("CON", "Constituição", 10),
            ],
            derived_rules: vec![
                DerivedRule::new("str_mod", "(STR-10)/2"),
                DerivedRule::new("con_mod", "(CON-10)/2"),
                DerivedRule::new("prof_bonus", "2 + (level-1)/4"),
                // depende de duas outras derived rules — testa a
                // ordenação topológica de verdade.
                DerivedRule::new("attack_bonus", "str_mod + prof_bonus"),
                DerivedRule::new("hp_max", "level * 6 + con_mod * level"),
            ],
        }
    }

    #[test]
    fn resolves_chained_derived_rules_with_real_dnd_formulas() {
        let mut entity = Entity::new("hero-1", "dnd5e");
        entity.set_base("level", 5);
        entity.set_base("STR", 16); // mod = 3
        entity.set_base("CON", 14); // mod = 2

        let result = compute_attributes(&entity, &dnd_like_ruleset()).unwrap();

        assert_eq!(result["str_mod"], 3);
        assert_eq!(result["con_mod"], 2);
        assert_eq!(result["prof_bonus"], 3); // 2 + (5-1)/4 = 2+1 = 3
        assert_eq!(result["attack_bonus"], 6); // str_mod(3) + prof_bonus(3)
        assert_eq!(result["hp_max"], 40); // 5*6 + 2*5 = 30+10
    }

    #[test]
    fn effect_modifies_base_before_derived_rules_run() {
        let mut entity = Entity::new("hero-1", "dnd5e");
        entity.set_base("level", 5);
        entity.set_base("STR", 14); // mod seria 2, sem o item
        entity.add_effect(Effect::new(
            "STR",
            Operation::Add,
            "2",
            "gauntlets_of_ogre_power",
        ));

        let result = compute_attributes(&entity, &dnd_like_ruleset()).unwrap();

        assert_eq!(result["STR"], 16); // 14 + 2 do effect
        assert_eq!(result["str_mod"], 3); // derived rule usa o STR já modificado
    }

    #[test]
    fn detects_circular_dependency_between_derived_rules() {
        let ruleset = Ruleset {
            attributes: vec![],
            derived_rules: vec![
                DerivedRule::new("a", "b + 1"),
                DerivedRule::new("b", "a + 1"),
            ],
        };
        let entity = Entity::new("x", "dnd5e");

        let err = compute_attributes(&entity, &ruleset).unwrap_err();
        assert!(matches!(err, EngineError::CircularDependency(_)));
    }

    #[test]
    fn unknown_variable_in_formula_is_an_error() {
        let ruleset = Ruleset {
            attributes: vec![],
            derived_rules: vec![DerivedRule::new("x", "totally_unknown_var + 1")],
        };
        let entity = Entity::new("x", "dnd5e");

        let err = compute_attributes(&entity, &ruleset).unwrap_err();
        assert!(matches!(err, EngineError::Formula(_)));
    }

    /// Antes da correção do dice-engine (divisão trunc em vez de
    /// floor), este teste documentava um resultado ERRADO (0) como
    /// comportamento conhecido. Depois que `/` passou a fazer floor
    /// division no dice-engine, o resultado aqui é o correto do D&D 5e:
    /// modificador de STR 9 é -1 (floor((9-10)/2) = floor(-0.5) = -1).
    /// Mantido como teste de regressão pra garantir que a correção do
    /// dice-engine realmente se propaga até aqui.
    #[test]
    fn negative_odd_modifier_now_floors_correctly() {
        let ruleset = Ruleset {
            attributes: vec![AttributeDefinition::new("STR", "Força", 10)],
            derived_rules: vec![DerivedRule::new("str_mod", "(STR-10)/2")],
        };
        let mut entity = Entity::new("x", "dnd5e");
        entity.set_base("STR", 9);

        let result = compute_attributes(&entity, &ruleset).unwrap();
        assert_eq!(result["str_mod"], -1); // corrigido — antes dava 0
    }
}
