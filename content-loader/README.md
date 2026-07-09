# content-loader

Content Layer do RPG Engine. Le arquivos JSON de um pacote de conteudo e converte para tipos usados pelas camadas acima, mantendo o `engine-core` livre de serde, filesystem e detalhes de D&D.

## Status

Implementado e validado pelo workspace. Para verificar:

```bash
cargo test
```

## O que foi feito

| Arquivo | Responsabilidade |
|---|---|
| `manifest.rs` | DTO de `manifest.json`. |
| `ruleset_file.rs` | DTO de `ruleset.json` + conversao para `engine_core::Ruleset`. |
| `content_node.rs` | DTO generico de `ContentNode` e conversao de `mechanics.effects` para `engine_core::Effect`. |
| `data_types.rs` | `mechanics.data` tipado por `type`: `race`, `feature`, `class`, `spell`, `item` e `monster`. |
| `loader.rs` | Entradas de leitura: `load_manifest`, `load_ruleset`, `load_content_node` e `load_content_nodes_from_dir`. |
| `error.rs` | `LoaderError`, incluindo `TypeMismatch`. |

## Como funciona

`ContentNode.mechanics.data` continua como `serde_json::Value` generico. Cada tipo ganha um metodo de conversao sob demanda: `race_data()`, `spell_data()`, `item_data()`, `monster_data()`, etc.

Cada metodo confere se `node_type` bate com o esperado antes de deserializar. Chamar `monster_data()` em um node que nao e `type: "monster"` retorna `LoaderError::TypeMismatch`.

Esse desenho evita deserializacao polimorfica complexa e mantem cada novo tipo de conteudo isolado: uma struct de dados e um metodo no `ContentNode`.

## Bestiario

A Fase 8 adiciona infraestrutura de bestiario baseada em content-packs:

- monstros sao `ContentNode`s com `type: "monster"`;
- os dados ficam em JSON dentro do content-pack;
- `MonsterData` descreve o resumo necessario para listar/ver um monstro;
- `speed` de monstros usa numero em pes, no mesmo formato canonico de racas;
- `load_content_nodes_from_dir` permite carregar todos os JSONs de uma pasta como `content-packs/dnd5e-core/monsters`;
- nenhum monstro e cadastrado diretamente no codigo.

Isso deixa aberto o caminho para ferramentas futuras de homebrew ou importacao gerarem JSONs compativeis e aparecerem no mesmo bestiario. A importacao de PDF em si nao faz parte desta fase.

Na Fase 10, o app desktop comeca a gravar monstros homebrew locais usando o mesmo envelope `ContentNode` e valida o JSON gerado carregando-o novamente por este crate. O `content-loader` continua sem saber se o arquivo veio do pack oficial ou de uma ferramenta de homebrew.

## Conteudo atual do dnd5e-core

O pack `content-packs/dnd5e-core/` contem a fatia vertical inicial:

- `races/human.json`;
- `features/human_ability_score_increase.json`;
- `classes/wizard.json`;
- quatro magias;
- tres itens;
- `monsters/goblin.json`.

## Limites atuais

- Nao ha Content Registry nem resolucao automatica de referencias por id.
- Nao ha importacao de PDF.
- Nao ha bestiario massivo do SRD.
- Nao ha transformacao de monstros em `Entity` de combate/NPC ainda.
- Stacking completo de effects e expiracao de `Duration::Rounds` seguem para fluxos futuros.

## Usado por

O app desktop usa este crate para montar a ficha inicial, carregar conteudo exibido na UI e listar o bestiario a partir dos JSONs do content-pack.
