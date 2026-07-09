# AGENTS.md

## Objetivo do projeto

Este projeto é um RPG Engine/VTT em Rust, focado inicialmente em D&D 5e SRD.

O objetivo atual NÃO é criar marketplace, multiplayer, plugins complexos, IA, suporte real a vários sistemas ou interface avançada.

A prioridade é construir um fluxo funcional para uma mesa de D&D 5e.

## Regra principal

Antes de modificar qualquer arquivo:

1. Leia este arquivo.
2. Leia o README.md da raiz.
3. Leia os README.md dos crates relevantes.
4. Leia o Cargo.toml da raiz e dos crates relevantes.
5. Explique o que entendeu.
6. Não altere arquitetura sem justificar.

## Arquitetura atual

Workspace Rust com crates separados:

- dice-engine: parser, AST, evaluator e rolagens de dados.
- engine-core: domínio genérico, Entity, AttributeDefinition, DerivedRule e Effect.
- content-loader: lê content-packs JSON e converte para engine-core.
- content-packs: conteúdo declarativo, começando por D&D 5e SRD.

O fluxo é:

content-packs JSON
-> content-loader
-> engine-core
-> dice-engine

## Regras de arquitetura

- dice-engine nunca deve conhecer D&D.
- engine-core nunca deve conhecer D&D.
- content-loader é a fronteira com JSON/arquivos.
- Content packs são dados, não código.
- Salvar estado canônico, não valores derivados.
- Não criar crates/camadas novas sem necessidade real.
- Não adicionar suporte a outros sistemas antes da hora.
- Não expandir conteúdo massivamente antes da infraestrutura funcionar.

## Regras de desenvolvimento

- Sempre rodar `cargo test` antes de finalizar.
- Sempre atualizar README do crate alterado.
- Não usar `unwrap()` fora de testes sem motivo claro.
- Não mover arquivos sem necessidade.
- Não apagar código sem explicar.
- Não fazer grandes refactors junto com funcionalidade nova.

## Git

Não faça commit automaticamente sem autorização.

Ao terminar uma tarefa:
1. Mostre o resumo do que mudou.
2. Mostre os testes executados.
3. Espere autorização do usuário para commit.

## Fase atual

Fase atual: preparar Fase 5 — Persistência SQLite.

Antes de iniciar a Fase 5, remover sujeiras óbvias do projeto, como a pasta `src/` da raiz se ela não fizer parte do workspace.
