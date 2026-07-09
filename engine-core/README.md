# engine-core

Domain Layer do RPG Engine — Fase 2. Consome o `dice-engine` (crate irmão) pra calcular atributos de qualquer `Entity`, sem saber o que é D&D, Pathfinder ou qualquer outro sistema.

## Status

Código escrito e revisado à mão, **não compilado neste ambiente** (mesma limitação do `dice-engine`: sandbox sem toolchain Rust). Antes de considerar pronto:

```bash
cd rpg-engine
cargo test
```

(Roda os testes de **todos** os crates do workspace — `dice-engine` e `engine-core` — de uma vez, porque agora tudo está sob um `Cargo.toml` de workspace na raiz.)

## O que foi feito

| Arquivo | Responsabilidade |
|---|---|
| `attribute.rs` | `AttributeDefinition`: um atributo declarado por um Ruleset (id, label, valor padrão). O core nunca sabe o que "STR" significa. |
| `rule.rs` | `DerivedRule`: uma fórmula que **calcula** um valor a partir de outros (ex: bônus de proficiência a partir do nível). |
| `effect.rs` | `Effect`: uma modificação declarativa que **altera** um valor existente quando está "ativo" numa Entity (item equipado, magia ativa). Inclui `source`, `duration`, `stacking` conforme a especificação da arquitetura. |
| `entity.rs` | `Entity`: qualquer "coisa" com atributos (personagem, NPC, monstro). Guarda só valores base + effects ativos — não calcula nada sozinha. Expõe leitura dos atributos base para a camada de persistência salvar estado canônico. |
| `engine.rs` | `compute_attributes()`: o motor de recálculo. Resolve base → effects → derived rules (em ordem topológica), usando o `dice-engine` pra avaliar cada fórmula. |
| `error.rs` | `EngineError`: dependência circular, atributo desconhecido, erro de fórmula (propagado do `dice-engine`). |

## Como funciona

### 1. `DerivedRule` calcula, `Effect` modifica

Essa distinção (definida no documento de arquitetura) é o que evita reinventar a mesma lógica de duas formas diferentes:

- **`DerivedRule`** sempre roda, não depende de nada estar "ativo". Ex: `prof_bonus = "2 + (level-1)/4"` — enquanto existir um `level`, o bônus de proficiência existe.
- **`Effect`** só tem efeito enquanto sua fonte estiver ativa na Entity (item equipado, magia com concentração). Ex: uma luva mágica que soma +2 na Força só conta enquanto o item estiver na lista de effects da Entity.

### 2. Ordem de resolução (fixa, documentada em `engine.rs`)

```
1. Contexto inicial = default_value de cada AttributeDefinition,
   sobrescrito por override explícito da Entity (Entity::set_base)
2. Effects aplicados, na ordem em que foram adicionados à Entity
3. DerivedRules resolvidas em ordem topológica
```

O passo 3 usa um grafo de dependências: se `attack_bonus` referencia `str_mod` e `prof_bonus` na fórmula, o motor detecta essa dependência automaticamente (analisando quais variáveis a fórmula usa) e garante que `str_mod`/`prof_bonus` rodem primeiro. Se duas rules dependerem uma da outra (ciclo), o motor detecta e retorna `EngineError::CircularDependency` em vez de travar/estourar recursão.

```rust
use engine_core::{AttributeDefinition, DerivedRule, Entity, Ruleset, compute_attributes};

let ruleset = Ruleset {
    attributes: vec![
        AttributeDefinition::new("level", "Nível", 1),
        AttributeDefinition::new("STR", "Força", 10),
    ],
    derived_rules: vec![
        DerivedRule::new("str_mod", "(STR-10)/2"),
        DerivedRule::new("prof_bonus", "2 + (level-1)/4"),
        DerivedRule::new("attack_bonus", "str_mod + prof_bonus"), // depende das duas acima
    ],
};

let mut hero = Entity::new("hero-1", "dnd5e");
hero.set_base("level", 5);
hero.set_base("STR", 16);

let computed = compute_attributes(&hero, &ruleset).unwrap();
assert_eq!(computed["attack_bonus"], 6); // str_mod(3) + prof_bonus(3)
```

### 3. Reaproveitando o dice-engine pra fórmulas sem dado

`(STR-10)/2` não tem nenhum `d20` nem `d6` — mas é avaliado pelo **mesmo** parser/evaluator do `dice-engine`, porque a gramática dele já cobre expressões aritméticas puras (números, variáveis, `+ - * /`, parênteses). Não foi criado um segundo parser só pra isso — reuso direto do crate da Fase 1.

## Limitação conhecida — resolvida

Era: "divisão inteira trunca em direção a zero, não faz `floor()`", o que dava resultado errado pra modificador de atributo com valor ímpar abaixo de 10 (ex: `STR=9` dava `0` em vez de `-1`). **Corrigido no `dice-engine`**: o operador `/` agora sempre arredonda pra baixo (`floor`), que é a convenção universal do D&D 5e pra qualquer divisão. Ver `dice-engine/src/evaluator.rs::floor_div` e o teste `division_rounds_down_not_toward_zero`. O teste de regressão aqui (`negative_odd_modifier_now_floors_correctly`) confirma que a correção se propaga corretamente até o `engine-core`.

## O que ficou de fora (de propósito)

- **Stacking de Effects não totalmente implementado.** Hoje os effects são aplicados na ordem em que estão na Entity, sem agrupar por `source`/`stacking`. A regra real ("bônus com mesmo nome não acumula, tipos diferentes acumulam") só vai ficar clara modelando magias/itens reais do SRD — especular ela agora seria inventar sem dado na frente, o que já rejeitamos antes nesse projeto.
- **`Duration` não é consultado ainda.** O campo existe no `Effect`, mas nada no `engine-core` expira um effect automaticamente — isso é claramente Fase 7 (Runtime de combate), não Domain Core.

## Próximo passo

Fase 3: `content-loader` — ler `ContentNode`/manifest de um pacote JSON e produzir `AttributeDefinition`/`DerivedRule`/`Effect` (os tipos deste crate) automaticamente, em vez de escrever tudo à mão como nos testes acima.
