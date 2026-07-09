# dice-engine

Motor de rolagem de dados independente e agnóstico de sistema de RPG — Fase 1 do RPG Engine.

Não sabe o que é D&D, Pathfinder ou qualquer outro sistema. Sabe só: ler uma fórmula de texto, resolver variáveis, rolar dados e devolver um resultado auditável. Quem decide o que "vantagem" ou "crítico" significam é quem chama o crate — ele mesmo só executa a política que recebe.

## Status

Implementado e validado pelo workspace. Para verificar:

```bash
cargo test
```

## O que foi feito

Todo o pipeline descrito na arquitetura (seção 9 do documento de arquitetura): `String → Lexer → Parser → AST → Evaluator → RollResult`, dividido em módulos de responsabilidade única:

| Arquivo | Responsabilidade |
|---|---|
| `lexer.rs` | Transforma a string da fórmula em tokens (`Number`, `Identifier`, `Dice`, `KeepHighest`, operadores, parênteses). Só reconhece símbolos, não sabe o que é uma regra de jogo. |
| `ast.rs` | Define a árvore de sintaxe (`Expr`). Representa **só a expressão matemática** — nunca sabe o que é vantagem, desvantagem ou crítico. |
| `parser.rs` | Recursive descent parser: tokens → `Expr`. Resolve precedência (`+`/`-` antes de `*`/`/`), parênteses, dados implícitos (`d20` = `1d20`) e `kh`/`kl`. |
| `policy.rs` | `RollPolicy` (vantagem/desvantagem/crítico) — deliberadamente **fora** do AST. É o que decide *como* avaliar a árvore, não o que a árvore representa. |
| `context.rs` | `RollContext`: mapa de variáveis (`STR`, `PROF`, `level`...) usado para resolver `Expr::Variable` na hora de avaliar. |
| `rng.rs` | Abstrai a fonte de aleatoriedade via trait `Roller`. `RandRoller` é a implementação real (usa a crate `rand`); `FixedRoller` (só em testes) devolve uma sequência fixa de valores, pra testar vantagem/crítico/keep-highest sem depender de sorte. |
| `evaluator.rs` | Percorre o AST aplicando `RollContext` e `RollPolicy`, produz a árvore `EvalNode` com o resultado de cada nó. É o único módulo que sabe rolar dados de verdade e aplicar vantagem/crítico. |
| `result.rs` | `RollResult`/`EvalNode`: resultado estruturado, não só um número — guarda cada rolagem individual, o que foi mantido/descartado, e o total de cada subexpressão. Serve pra UI (Fase 6) mostrar o "porquê" de um resultado sem recalcular nada. |
| `error.rs` | `DiceError`: erros de lexer, parser e avaliação (caractere inesperado, variável desconhecida, divisão por zero, dado com 0 lados, etc). |
| `lib.rs` | Ponto de entrada público: expõe `roll()`, `parse_formula()`, `evaluate_with_roller()` e reexporta os tipos públicos. |

## Como funciona

### 1. Gramática suportada

```text
expression := term (('+' | '-') term)*
term       := factor (('*' | '/') factor)*
factor     := dice | number | variable | '(' expression ')' | '-' factor
dice       := [number] 'd' number (('kh' | 'kl') number)?
```

Exemplos válidos: `1d20+STR+PROF`, `d20` (implícito = `1d20`), `4d6kh3` (rolagem de atributo, mantém os 3 maiores de 4), `(2d6+1)*2`, `-2`.

Fora do escopo (de propósito — ver critério de mudança da arquitetura): explode, reroll, drop, funções, comparadores.

### 2. A separação mais importante: AST vs. Política

`1d20+STR+PROF` gera **exatamente a mesma árvore** (`Expr`) seja qual for a política de rolagem. Vantagem, desvantagem e crítico não existem no parser nem no AST — eles são parâmetros passados ao `evaluator` via `RollPolicy`, aplicados *durante* o percurso da árvore.

```rust
use dice_engine::{roll, RollContext, RollPolicy};

let ctx = RollContext::new().with("STR", 3).with("PROF", 2);

// Rolagem normal
let normal = roll("1d20+STR+PROF", &ctx, &RollPolicy::normal()).unwrap();

// Mesma fórmula, com vantagem
let com_vantagem = roll("1d20+STR+PROF", &ctx, &RollPolicy::with_advantage()).unwrap();

println!("{}", normal.total);
println!("{}", com_vantagem.describe()); // ex: "((1d20[14,9]->kept[14] + STR(3)) + PROF(2)) = 19"
```

### 3. Regra de vantagem/desvantagem

Aplica-se **só ao primeiro nó `1d20` puro** (`count == 1`, `sides == 20`) encontrado ao percorrer a árvore da esquerda para a direita. Isso cobre o caso real do SRD: vantagem afeta o d20 de um teste/ataque, nunca os dados de uma fórmula de dano. Uma `RollPolicy` com vantagem aplicada a uma fórmula sem nenhum `1d20` simplesmente não tem efeito — não é erro.

### 4. Regra de crítico

Dobra a **quantidade** de dados de cada nó `Dice` avaliado (a regra de dano de D&D 5e: dobra os dados, não o modificador fixo). Por isso, `RollPolicy::critical()` deve ser usado só na avaliação da fórmula de **dano**, nunca na de ataque — uma fórmula de ataque não tem dado de dano pra dobrar.

### 5. Regra de divisão: sempre arredonda pra baixo (`floor`)

`/` nunca trunca em direção a zero — sempre faz `floor()` matemático, via a função `floor_div` em `evaluator.rs`. É a convenção universal de D&D 5e ("round down" aparece em quase toda regra com divisão: modificador de atributo, metade do nível, etc.). Pra valores positivos, dá o mesmo resultado de uma divisão truncada normal; a diferença só aparece com resultado intermediário negativo — ex: `(9-10)/2` dá `-1` (modificador correto de Força 9), não `0`.

### 6. `RollResult` é auditável

Não devolve só um `i64`. `RollResult.root` é uma árvore `EvalNode` guardando, em cada nó `Dice`, todos os valores rolados (`rolls`) e quais foram efetivamente mantidos (`kept`) — inclusive nas duas rolagens de uma vantagem/desvantagem. É isso que permite a UI (Fase 6) mostrar "rolou 14 e 9, manteve 14 (vantagem)" sem reimplementar a lógica de avaliação do lado da interface.

### 7. Testabilidade sem depender de sorte

O `evaluator` não usa `rand` diretamente — depende da trait `Roller`. Em produção, `RandRoller` (aleatoriedade real via `rand::thread_rng`). Nos testes, `FixedRoller` devolve uma fila de valores pré-definida, permitindo testar determinísticamente "vantagem mantém o maior dos dois" sem rodar a rolagem mil vezes esperando cobrir os casos.

## API pública (`lib.rs`)

```rust
// Rolagem completa, com RNG real
pub fn roll(formula: &str, ctx: &RollContext, policy: &RollPolicy) -> Result<RollResult, DiceError>;

// Só valida a sintaxe, sem rolar nada — útil pro Content Loader validar
// a fórmula de um Effect ao carregar um pacote, sem gastar rolagem
pub fn parse_formula(formula: &str) -> Result<Expr, DiceError>;

// Avalia um Expr já parseado com um Roller à sua escolha (usado em testes)
pub fn evaluate_with_roller(expr: &Expr, ctx: &RollContext, policy: &RollPolicy, roller: &mut impl Roller) -> Result<RollResult, DiceError>;
```

## O que ficou de fora (de propósito)

Consistente com o critério de mudança da arquitetura ("resolve o caso real de uma mesa de D&D 5e?"):

- Dice pools, step dice, explode, reroll — não usados pelo SRD 5e no MVP.
- Pratt parser — a gramática tem só 2 níveis de precedência; recursive descent é igualmente correto e mais simples de ler.
- Qualquer noção de sistema de RPG dentro do crate — isso é responsabilidade do content-pack (Fase 4), nunca do dice-engine.

## Usado por

O `engine-core` usa este crate para avaliar fórmulas de `DerivedRule` e `Effect`. A aplicação desktop da Fase 6 consome o resultado indiretamente via `engine-core`.
