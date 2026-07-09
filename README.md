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
└── content-packs/         # dados de conteúdo (JSON), não código
    └── dnd5e-core/         # ruleset + 1 raça + 1 feature + 1 classe + 4 magias + 3 itens
```

Cada crate tem seu próprio README detalhando o que foi feito e como funciona.

## Rodando os testes

Da raiz do workspace, roda os testes de **todos** os crates de uma vez:

```bash
cargo test
```

## Progresso

- [x] **Fase 1 — Dice Engine.** Lexer/Parser/Evaluator para fórmulas tipo `1d20+STR+PROF`, vantagem/desvantagem, crítico, keep-highest, divisão sempre arredondando pra baixo (`floor`). 27 testes + doctest, todos passando (validado localmente).
- [x] **Fase 2 — Domain Core.** `Entity`, `AttributeDefinition`, `DerivedRule`, `Effect`, motor de recálculo com resolução topológica de dependências. 5 testes + doctest, todos passando (validado localmente).
- [x] **Fase 3 — Content Loader.** Lê `manifest.json`/`ruleset.json`/`ContentNode` em JSON e converte pro `engine-core`. 4 testes, validado localmente.
- [x] **Fase 4 — Fatia vertical: Humano Mago nível 1.** `mechanics.data` tipado por tipo (`race`/`feature`/`class`/`spell`/`item`). Personagem completo montado a partir de 9 arquivos de conteúdo real, PV calculado bate com a regra do SRD. **Ainda não validado localmente.**
- [ ] Fase 5 — Persistência (SQLite)
- [ ] Fase 6 — UI: Ficha de Personagem
- [ ] Fase 7 — Campanhas e Combate básico
- [ ] Fase 8 — Bestiário e NPCs
- [ ] Fase 9 — Mapas (básico)
- [ ] Fase 10 — Homebrew tooling
