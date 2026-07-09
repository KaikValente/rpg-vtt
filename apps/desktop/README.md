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
- Ficha inicial do Humano Mago nivel 1 usando o content pack `dnd5e-core`.
- Teste Rust garantindo que a ficha e montada a partir do content pack.

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
- Ainda nao ha selecao de campanha/personagem salvo.
- Ainda nao ha integracao da UI com `persistence-sqlite`.
- Ainda nao ha combate, mapas ou bestiario.
