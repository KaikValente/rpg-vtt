# RPG Engine Desktop

Aplicacao desktop da Fase 6, feita com Tauri + React.

## Papel na arquitetura

Este app e a camada de interface. Ele nao calcula regras por conta propria:

```text
React
-> Tauri commands
-> content-loader
-> engine-core
-> dice-engine
```

O frontend chama comandos Tauri. O backend Tauri orquestra os crates existentes e devolve dados prontos para renderizacao.

## O que existe agora

- App React/Vite.
- Backend Tauri v2.
- Comando `load_character_sheet`.
- Campanha local padrao criada/carregada via `persistence-sqlite`.
- Personagem inicial Humano Mago nivel 1 salvo como estado canonico e depois recalculado usando o content pack `dnd5e-core`.
- Comandos `start_basic_combat` e `advance_combat_turn`.
- Painel de combate com participantes, iniciativa e turno atual.
- Comando `load_bestiary`.
- Painel de bestiario carregando monstros do content-pack.
- Combate inicial cria o participante Goblin a partir do content-pack, mantendo CA/PV/acoes no bestiario ate existir um modelo completo de NPC em combate.
- Comandos `load_basic_map` e `move_map_token`.
- Painel de mapa basico com grid e tokens posicionaveis.
- Testes Rust garantindo que a ficha e montada a partir do estado salvo, que edicoes canonicas persistidas sao preservadas, que o combate basico avanca turno, que o bestiario vem dos JSONs do content-pack e que o mapa basico cria/move tokens.

## Rodando

Instale as dependencias de frontend:

```bash
cd apps/desktop
npm install
```

Modo desktop:

```bash
npm run tauri dev
```

Build do frontend:

```bash
npm run build
```

Da raiz do workspace, os testes Rust continuam rodando com:

```bash
cargo test
```

## Limites atuais

- Ainda nao ha edicao da ficha pela UI.
- Ainda nao ha selecao de campanha/personagem salvo; existe apenas uma campanha local padrao.
- O bestiario ainda e uma listagem simples; o combate inicial usa o Goblin do content-pack, mas ainda nao transforma qualquer monstro escolhido em NPC completo automaticamente.
- O mapa ainda e minimo: nao ha assets, paredes, iluminacao, fog of war, medida de distancia ou multiplayer.
- O combate ainda e minimo: nao ha ataques, dano, condicoes, expiracao de efeitos por rodada ou bestiario completo.
