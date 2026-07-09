# RPG Engine

Motor modular de RPG de mesa. Foco da V1: uma mesa jogando D&D 5e (SRD) com homebrews compatíveis — arquitetura preparada pra crescer, implementação guiada pelo caso real, não por especulação. Ver o documento de arquitetura completo pra contexto de decisões (`arquitetura-rpg-engine.md`, compartilhado fora deste repositório).

## Estrutura

Cargo workspace com um crate por camada/responsabilidade:

```
rpg-engine/
├── Cargo.toml            # workspace root
├── dice-engine/           # Fase 1 — motor de rolagem, independente
├── engine-core/           # Fase 2 — Domain Layer (Entity/AttributeDefinition/DerivedRule/Effect)
├── content-loader/        # Fases 3-4 — Content Layer (lê JSON, tipa por tipo, converte pro Domain Layer)
├── persistence-sqlite/    # Fase 5 — Persistence Layer (SQLite, estado canônico)
├── apps/
│   └── desktop/            # Fase 6 — app Tauri + React (Ficha de Personagem)
└── content-packs/         # dados de conteúdo (JSON), não código
    └── dnd5e-core/         # ruleset + raça, feature, classe, magias, itens e monstro inicial
```

Cada crate tem seu próprio README detalhando o que foi feito e como funciona.

## Rodando os testes

Da raiz do workspace, roda os testes de **todos** os crates de uma vez:

```bash
cargo test
```

## Rodando o app desktop

```bash
cd apps/desktop
npm install
npm run tauri dev
```

## Progresso

- [x] **Fase 1 — Dice Engine.** Lexer/Parser/Evaluator para fórmulas tipo `1d20+STR+PROF`, vantagem/desvantagem, crítico, keep-highest, divisão sempre arredondando pra baixo (`floor`). 27 testes + doctest, todos passando (validado localmente).
- [x] **Fase 2 — Domain Core.** `Entity`, `AttributeDefinition`, `DerivedRule`, `Effect`, motor de recálculo com resolução topológica de dependências. 5 testes + doctest, todos passando (validado localmente).
- [x] **Fase 3 — Content Loader.** Lê `manifest.json`/`ruleset.json`/`ContentNode` em JSON e converte pro `engine-core`. 4 testes, validado localmente.
- [x] **Fase 4 — Fatia vertical: Humano Mago nível 1.** `mechanics.data` tipado por tipo (`race`/`feature`/`class`/`spell`/`item`). Personagem completo montado a partir de 9 arquivos de conteúdo real, PV calculado bate com a regra do SRD. **Ainda não validado localmente.**
- [x] **Fase 5 — Persistência SQLite.** Novo crate `persistence-sqlite` salva campanhas, entidades, atributos base explícitos e effects ativos. Não persiste valores derivados de `compute_attributes`; eles continuam sendo recalculados pelo `engine-core`.
- [x] **Fase 6 — UI: Ficha de Personagem.** App desktop Tauri + React em `apps/desktop`, com comando Tauri que monta a ficha do Humano Mago nível 1 usando `content-loader` + `engine-core`.
- [x] **Fase 7 — Campanhas e Combate básico.** App desktop cria/carrega campanha local via `persistence-sqlite`, salva o personagem inicial como estado canônico, recalcula a ficha a partir do estado salvo e oferece combate mínimo com participantes, iniciativa e avanço de turno.
- [x] **Fase 8 — Bestiário baseado em content-packs.** `content-loader` interpreta `monster`, o content pack `dnd5e-core` inclui um monstro inicial em JSON, e o app desktop lista o bestiário via comando Tauri sem hardcode de monstros ou regra de D&D no `engine-core`.
- [ ] Fase 9 — Mapas (básico)
- [ ] Fase 10 — Homebrew tooling
