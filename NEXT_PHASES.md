# Proximas Fases

Resumo pratico para continuar o RPG Engine depois da Fase 8.

## Estado atual

- Fase 1: `dice-engine` pronto para formulas e rolagens auditaveis.
- Fase 2: `engine-core` calcula atributos a partir de `Entity`, `Effect` e `DerivedRule`.
- Fases 3-4: `content-loader` carrega o content pack `dnd5e-core`.
- Fase 5: `persistence-sqlite` salva estado canonico de campanhas, entidades e effects ativos.
- Fase 6: `apps/desktop` exibe a ficha inicial via Tauri + React.
- Fase 7: `apps/desktop` cria/carrega uma campanha local, persiste o personagem inicial, recalcula a ficha a partir do estado salvo e oferece combate minimo com participantes, iniciativa e turno atual.
- Fase 8: bestiario baseado em content-packs. `content-loader` interpreta `monster`, `dnd5e-core` contem um monstro inicial em JSON, e o app desktop lista monstros via comando Tauri.

## Regras que continuam valendo

- `dice-engine` nunca conhece D&D.
- `engine-core` nunca conhece D&D.
- `content-loader` continua sendo a fronteira com JSON/content packs.
- `apps/desktop` nao calcula regra de dominio; chama comandos Tauri.
- Persistencia salva estado canonico/operacional minimo, nao valores derivados.
- Content packs sao dados, nao codigo.
- Conteudo criado por homebrew tooling ou importado no futuro deve entrar como content-pack compativel.
- Monstros nao devem ser cadastrados diretamente no codigo.

## Proxima fase: Mapas basicos

Objetivo recomendado da Fase 9:

- criar uma infraestrutura minima de mapa/cena;
- representar grid simples e tokens de participantes;
- manter mapa como estado de campanha, nao regra de dominio no `engine-core`;
- permitir posicionar pelo menos o personagem e um monstro do bestiario em uma cena;
- nao implementar VTT avancado, fog of war, assets complexos ou multiplayer.

O primeiro fluxo util deve ser pequeno:

```text
abrir app
-> carregar campanha local
-> ver ficha e bestiario
-> iniciar combate simples
-> abrir mapa basico
-> ver/posicionar tokens em um grid
```

## Pendencias importantes

- Decidir o schema minimo de cena/mapa em `persistence-sqlite`.
- Conectar monstro do bestiario a um participante/NPC real de combate.
- Tornar a ficha editavel sem mover regra para React.
- Criar algum indice/resolucao de `ContentNode` por id quando carregar referencias ficar incomodo.
- Modelar expiracao de `Duration::Rounds` apenas quando houver turno/rodada real.
- Implementar stacking completo de `Effect`s somente com casos concretos.

## Evitar por enquanto

- Marketplace, plugins, multiplayer e IA.
- Suporte real a varios sistemas.
- Importacao de PDF.
- Monstros hardcoded em Rust ou React.
- Conteudo SRD em massa antes de melhorar infraestrutura.
- Refactors grandes junto com feature nova.
- Cache/persistencia de atributos derivados.
- Regras completas de combate, ataques, dano e condicoes antes do mapa minimo funcionar.
- Mapas avancados, iluminacao, fog of war ou upload de assets.

## Checklist antes de cada fase

- Ler `AGENTS.md`.
- Ler README da raiz e README dos crates tocados.
- Ler `Cargo.toml` da raiz e dos crates tocados.
- Explicar a mudanca antes de alterar arquitetura.
- Manter testes pequenos cobrindo o fluxo real.
- Rodar `cargo fmt`, `cargo clippy` e `cargo test`.
- Atualizar READMEs quando a responsabilidade de uma camada mudar.
