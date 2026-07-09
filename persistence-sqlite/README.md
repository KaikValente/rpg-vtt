# persistence-sqlite

Persistence Layer do RPG Engine — Fase 5. Salva estado canônico de campanhas e entidades em SQLite, sem misturar banco de dados com o `engine-core`.

## Status

Implementa a primeira fatia de persistência:

- schema SQLite criado automaticamente;
- campanhas;
- entidades;
- atributos base explícitos da `Entity`;
- `Effect`s ativos, preservando ordem, `duration` e `stacking`.

Validar da raiz do workspace:

```bash
cargo test
```

## O que foi feito

| Arquivo | Responsabilidade |
|---|---|
| `lib.rs` | API pública do crate. |
| `store.rs` | `SqliteStore`, schema e operações de salvar/carregar. |
| `error.rs` | `PersistenceError`, incluindo erros de conversão de valores armazenados. |

## Como funciona

O crate persiste apenas o estado canônico:

- `Campaign` (`id`, `name`, `ruleset_id`);
- `Entity` (`id`, `ruleset_id`);
- atributos base definidos explicitamente;
- efeitos ativos.

Valores calculados por `engine_core::compute_attributes()` **não** são salvos. Eles devem ser recalculados quando a ficha for carregada, usando o `Ruleset` atual e os efeitos ativos.

Essa decisão mantém a regra da arquitetura: salvar estado canônico, não valores derivados. Também mantém o `engine-core` agnóstico de SQLite; o domínio só expõe os dados canônicos da `Entity`.

## Limites atuais

- Não há migrations versionadas ainda; o schema é criado com `CREATE TABLE IF NOT EXISTS`.
- Não há Content Registry nem resolução automática de ContentNode por id.
- Não há persistência de valores derivados/cache de ficha.
- Não há runtime de combate, expiração automática de `Duration::Rounds` ou regras completas de stacking.

Esses pontos ficam para fases futuras, quando houver fluxo real exigindo cada um.

## Relação com a Fase 6

A UI desktop já existe em `apps/desktop`, mas ainda não salva/carrega personagens reais por este crate. A integração entre ficha editável e persistência fica para uma fatia futura de campanhas/personagens.
