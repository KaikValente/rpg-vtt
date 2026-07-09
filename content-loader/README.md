# content-loader

Content Layer do RPG Engine — Fases 3 e 4. Lê arquivos JSON de um pacote de conteúdo e converte pros tipos de domínio do `engine-core` (`AttributeDefinition`, `DerivedRule`, `Effect`), com interpretação tipada de `mechanics.data` por tipo de `ContentNode`.

## Status

Código escrito e revisado à mão, **não compilado neste ambiente**. Validar com:

```bash
cd rpg-engine
cargo test
```

## O que foi feito

| Arquivo | Responsabilidade |
|---|---|
| `manifest.rs` | DTO de `manifest.json`. |
| `ruleset_file.rs` | DTO de `ruleset.json` + conversão pra `engine_core::Ruleset`. |
| `content_node.rs` | DTO genérico de `ContentNode` (metadata/presentation/mechanics) + conversão de `mechanics.effects` pra `Vec<engine_core::Effect>`. |
| `data_types.rs` | **Novo na Fase 4.** `mechanics.data` tipado por `type`: `RaceData`, `FeatureData`, `ClassData`, `SpellData`, `ItemData`/`WeaponData`. Métodos `ContentNode::race_data()`, `::class_data()`, etc. |
| `loader.rs` | Funções de entrada: `load_manifest`, `load_ruleset`, `load_content_node`. |
| `error.rs` | `LoaderError`, incluindo `TypeMismatch` (novo — chamar `race_data()` num node que não é `race`). |

## Como funciona

### 1. `data` tipado sob demanda, não polimórfico

`ContentNode.mechanics.data` continua sendo `serde_json::Value` genérico (decisão da Fase 3, mantida). Cada tipo ganha um método de conversão sob demanda: `node.race_data()`, `node.spell_data()`, etc. — cada um confere que `node_type` bate com o esperado antes de deserializar, e devolve `LoaderError::TypeMismatch` se não bater. Evita deserialização polimórfica (mais complexa em serde) e mantém adicionar um tipo novo isolado — só mais uma struct + um método.

### 2. Fatia vertical: Humano Mago nível 1

`content-packs/dnd5e-core/` agora tem conteúdo suficiente pra montar um personagem inteiro:

```
races/human.json               — raça, referencia 1 feature por id
features/human_ability_score_increase.json  — +1 em todos os 6 atributos
classes/wizard.json            — d6 de vida, INT primário, Effect de PV
spells/fire_bolt.json          — truque, dano
spells/mage_hand.json          — truque, utilidade
spells/magic_missile.json      — 1º círculo, dano
spells/shield.json             — 1º círculo, defensiva
items/dagger.json              — arma simples
items/component_pouch.json     — equipamento sem mecânica de combate
```

O teste `content-loader/tests/character_slice.rs` carrega tudo isso, monta uma `Entity`, aplica os `Effect`s (raça primeiro, depois classe) e confirma que o resultado bate com a regra real do D&D 5e — inclusive **PV máximo = 8** pra um Mago humano nível 1 com Constituição 15 (14 base + 1 do traço racial).

### 3. Achado importante: ordem entre Effects e DerivedRules

O `Effect` de PV da classe usa a fórmula `"6 + (level-1)*4 + level*(CON-10)/2"` — repara que usa `CON` (atributo base), **não** `con_mod` (regra derivada). Isso não é estilo, é necessidade: `Effect`s rodam *antes* das `DerivedRule`s no `engine-core` (ver seção "Ordem de resolução" no README dele), então um Effect não pode depender de um valor derivado que ainda não foi calculado. Documentado aqui e no comentário do teste — é o tipo de restrição que só aparece implementando conteúdo real, exatamente como a arquitetura previu que aconteceria.

### 4. Resolução de referência ainda é manual

`race.race_data().traits` guarda o **id** da feature que a raça concede — mas carregar esse id de verdade (`load_content_node` com o caminho certo) ainda é manual no teste, porque o Content Registry (índice em memória por id, arquitetura seção 1) ainda não existe. Só vale a pena construir quando o volume de conteúdo justificar — carregar 8 arquivos à mão num teste ainda é gerenciável.

## O que ficou de fora (de propósito)

- **Progressão por nível** (`ClassData.levels`) — schema original da arquitetura previa isso; simplificado pra fora até existir um segundo nível de personagem pra validar contra.
- **Mecânica de conjuração** — as 4 magias carregam e têm `spell_data()` tipado, mas nada usa `damage_formula` pra rolar dano de verdade ainda. Isso é Fase 7 (combate).
- **Content Registry** — ver ponto 4 acima.
- **Stacking de Effects e Duration::Rounds** — mesmas limitações já documentadas na Fase 3, ainda válidas.

## Próximo passo

Fase 5: Persistência (SQLite) — salvar o resultado de `compute_attributes` (ou melhor, o `Entity` + quais ContentNodes estão aplicados) numa Campaign de verdade, em vez de reconstruir tudo em memória a cada `cargo test`.

