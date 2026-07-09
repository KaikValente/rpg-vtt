# persistence-sqlite

Persistence Layer do RPG Engine. Salva estado canonico de campanhas, entidades e combate basico em SQLite, sem misturar banco de dados com o `engine-core`.

## Status

Implementa a persistencia necessaria para a Fase 7:

- schema SQLite criado automaticamente;
- campanhas;
- entidades;
- atributos base explicitos da `Entity`;
- `Effect`s ativos, preservando ordem, `duration` e `stacking`;
- encontros de combate basicos com participantes, iniciativa e turno atual.

Validar da raiz do workspace:

```bash
cargo test
```

## O que foi feito

| Arquivo | Responsabilidade |
|---|---|
| `lib.rs` | API publica do crate. |
| `store.rs` | `SqliteStore`, schema e operacoes de salvar/carregar. |
| `error.rs` | `PersistenceError`, incluindo erros de conversao de valores armazenados. |

## Como funciona

O crate persiste apenas estado canonico/operacional minimo:

- `Campaign` (`id`, `name`, `ruleset_id`);
- `Entity` (`id`, `ruleset_id`);
- atributos base definidos explicitamente;
- efeitos ativos;
- `CombatEncounter` (`id`, `campaign_id`, `current_turn_index`);
- `CombatParticipant` (`id`, `entity_id` opcional, `name`, `initiative`).

Valores calculados por `engine_core::compute_attributes()` **nao** sao salvos. Eles devem ser recalculados quando a ficha for carregada, usando o `Ruleset` atual e os efeitos ativos.

O estado de combate salvo aqui e propositalmente pequeno: ordem de iniciativa e participante do turno atual. Ele nao calcula regra de combate, nao expira `Duration::Rounds` automaticamente e nao salva atributos derivados/cache de ficha.

Essa decisao mantem a regra da arquitetura: salvar estado canonico, nao valores derivados. Tambem mantem o `engine-core` agnostico de SQLite; o dominio so expoe os dados canonicos da `Entity`.

## Limites atuais

- Nao ha migrations versionadas ainda; o schema e criado com `CREATE TABLE IF NOT EXISTS`.
- Nao ha Content Registry nem resolucao automatica de ContentNode por id.
- Nao ha persistencia de valores derivados/cache de ficha.
- Nao ha expiracao automatica de `Duration::Rounds` ou regras completas de stacking.

Esses pontos ficam para fases futuras, quando houver fluxo real exigindo cada um.

## Relacao com o app desktop

O app em `apps/desktop` usa este crate para criar/carregar uma campanha local padrao, salvar o personagem inicial como estado canonico e manter um combate simples com iniciativa e turno atual.
