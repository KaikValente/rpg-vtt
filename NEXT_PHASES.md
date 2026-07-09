# Proximas Fases

Resumo pratico para continuar o RPG Engine depois da Fase 6.

## Estado atual

- Fase 1: `dice-engine` pronto para formulas e rolagens auditaveis.
- Fase 2: `engine-core` calcula atributos a partir de `Entity`, `Effect` e `DerivedRule`.
- Fases 3-4: `content-loader` carrega o content pack `dnd5e-core`.
- Fase 5: `persistence-sqlite` salva estado canonico de campanhas e entidades.
- Fase 6: `apps/desktop` exibe uma ficha inicial via Tauri + React.

## Regras que continuam valendo

- `dice-engine` nunca conhece D&D.
- `engine-core` nunca conhece D&D.
- `content-loader` continua sendo a fronteira com JSON/content packs.
- `apps/desktop` nao calcula regra de dominio; chama comandos Tauri.
- Persistencia salva estado canonico, nao valores derivados.
- Content packs sao dados, nao codigo.

## Proxima fase: Campanhas e combate basico

Objetivo recomendado da Fase 7:

- permitir criar/carregar uma campanha;
- persistir pelo menos um personagem real via `persistence-sqlite`;
- conectar a UI com a persistencia;
- iniciar um fluxo minimo de combate com participantes, iniciativa e turno atual;
- recalcular ficha a partir do estado salvo, sem salvar atributos derivados.

O primeiro fluxo util deve ser pequeno:

```text
abrir app
-> carregar/criar campanha
-> carregar personagem salvo
-> ver ficha calculada
-> iniciar combate simples
-> registrar iniciativa/turno
```

## Pendencias importantes

- Integrar `apps/desktop` com `persistence-sqlite`.
- Tornar a ficha editavel sem mover regra para React.
- Criar algum indice/resolucao de ContentNode por id quando carregar referencias ficar incomodo.
- Decidir onde vive o estado runtime de combate.
- Modelar expiracao de `Duration::Rounds` apenas quando houver turno/rodada real.
- Implementar stacking completo de `Effect`s somente com casos concretos.

## Evitar por enquanto

- Marketplace, plugins, multiplayer e IA.
- Suporte real a varios sistemas.
- Conteudo SRD em massa antes de melhorar infraestrutura.
- Refactors grandes junto com feature nova.
- Cache/persistencia de atributos derivados.
- Regras de combate completas antes de uma iniciativa simples funcionar.

## Checklist antes de cada fase

- Ler `AGENTS.md`.
- Ler README da raiz e README dos crates tocados.
- Explicar a mudanca antes de alterar arquitetura.
- Manter testes pequenos cobrindo o fluxo real.
- Rodar `cargo fmt`, `cargo clippy` e `cargo test`.
- Atualizar READMEs quando a responsabilidade de uma camada mudar.
