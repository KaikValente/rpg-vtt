import { type FormEvent, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type AbilityScore = {
  id: string;
  label: string;
  score: number;
  modifier: number;
};

type SpellSummary = {
  name: string;
  level: number;
  school: string;
  damageFormula?: string | null;
  damageType?: string | null;
};

type ItemSummary = {
  name: string;
  itemType: string;
  damageFormula?: string | null;
  damageType?: string | null;
};

type CharacterSheet = {
  id: string;
  name: string;
  rulesetId: string;
  race: string;
  className: string;
  level: number;
  hpMax: number;
  proficiencyBonus: number;
  abilityScores: AbilityScore[];
  spells: SpellSummary[];
  items: ItemSummary[];
};

type CampaignSummary = {
  id: string;
  name: string;
  rulesetId: string;
};

type CampaignWorkspace = {
  campaign: CampaignSummary;
  character: CharacterSheet;
  combat?: CombatSummary | null;
  map?: MapSceneSummary | null;
};

type CombatSummary = {
  id: string;
  currentTurnIndex: number;
  currentTurnParticipantId?: string | null;
  participants: CombatParticipantSummary[];
};

type CombatParticipantSummary = {
  id: string;
  entityId?: string | null;
  name: string;
  initiative: number;
  isCurrentTurn: boolean;
};

type MonsterSummary = {
  id: string;
  name: string;
  description: string;
  size: string;
  creatureType: string;
  armorClass: number;
  hitPoints: number;
  speed: string;
  challengeRating: string;
  actions: MonsterActionSummary[];
};

type MonsterActionSummary = {
  name: string;
  attackBonus?: number | null;
  damageFormula?: string | null;
  damageType?: string | null;
};

type HomebrewMonsterDraft = {
  name: string;
  description: string;
  size: string;
  creatureType: string;
  armorClass: number;
  hitPoints: number;
  speed: number;
  challengeRating: string;
  strScore: number;
  dexScore: number;
  conScore: number;
  intScore: number;
  wisScore: number;
  chaScore: number;
  actionName: string;
  attackBonus: number;
  damageFormula: string;
  damageType: string;
};

type MapSceneSummary = {
  id: string;
  name: string;
  width: number;
  height: number;
  tokens: MapTokenSummary[];
};

type MapTokenSummary = {
  id: string;
  participantId?: string | null;
  entityId?: string | null;
  name: string;
  x: number;
  y: number;
};

function modifierText(value: number) {
  return value >= 0 ? `+${value}` : `${value}`;
}

const initialHomebrewDraft: HomebrewMonsterDraft = {
  name: "",
  description: "",
  size: "Small",
  creatureType: "humanoid",
  armorClass: 12,
  hitPoints: 7,
  speed: 30,
  challengeRating: "1/8",
  strScore: 10,
  dexScore: 10,
  conScore: 10,
  intScore: 10,
  wisScore: 10,
  chaScore: 10,
  actionName: "Ataque",
  attackBonus: 2,
  damageFormula: "1d6",
  damageType: "bludgeoning",
};

function App() {
  const [workspace, setWorkspace] = useState<CampaignWorkspace | null>(null);
  const [bestiary, setBestiary] = useState<MonsterSummary[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [combatBusy, setCombatBusy] = useState(false);
  const [mapBusy, setMapBusy] = useState(false);
  const [homebrewBusy, setHomebrewBusy] = useState(false);
  const [selectedTokenId, setSelectedTokenId] = useState<string | null>(null);
  const [homebrewDraft, setHomebrewDraft] =
    useState<HomebrewMonsterDraft>(initialHomebrewDraft);

  useEffect(() => {
    Promise.all([
      invoke<CampaignWorkspace>("load_character_sheet"),
      invoke<MonsterSummary[]>("load_bestiary"),
    ])
      .then(([loadedWorkspace, loadedBestiary]) => {
        setWorkspace(loadedWorkspace);
        setBestiary(loadedBestiary);
        setError(null);
      })
      .catch((err) => {
        setError(String(err));
      })
      .finally(() => {
        setLoading(false);
      });
  }, []);

  function runCombatCommand(command: "start_basic_combat" | "advance_combat_turn") {
    setCombatBusy(true);
    invoke<CampaignWorkspace>(command)
      .then((loaded) => {
        setWorkspace(loaded);
        setError(null);
      })
      .catch((err) => {
        setError(String(err));
      })
      .finally(() => {
        setCombatBusy(false);
      });
  }

  function openMap() {
    setMapBusy(true);
    invoke<CampaignWorkspace>("load_basic_map")
      .then((loaded) => {
        setWorkspace(loaded);
        setSelectedTokenId(loaded.map?.tokens[0]?.id ?? null);
        setError(null);
      })
      .catch((err) => {
        setError(String(err));
      })
      .finally(() => {
        setMapBusy(false);
      });
  }

  function moveToken(tokenId: string, x: number, y: number) {
    setMapBusy(true);
    invoke<CampaignWorkspace>("move_map_token", { tokenId, x, y })
      .then((loaded) => {
        setWorkspace(loaded);
        setSelectedTokenId(tokenId);
        setError(null);
      })
      .catch((err) => {
        setError(String(err));
      })
      .finally(() => {
        setMapBusy(false);
      });
  }

  function updateHomebrew(key: keyof HomebrewMonsterDraft, value: string | number) {
    setHomebrewDraft((current) => ({ ...current, [key]: value }));
  }

  function createHomebrewMonster(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setHomebrewBusy(true);
    invoke<MonsterSummary[]>("create_homebrew_monster", { draft: homebrewDraft })
      .then((loadedBestiary) => {
        setBestiary(loadedBestiary);
        setHomebrewDraft(initialHomebrewDraft);
        setError(null);
      })
      .catch((err) => {
        setError(String(err));
      })
      .finally(() => {
        setHomebrewBusy(false);
      });
  }

  const spellGroups = useMemo(() => {
    const sheet = workspace?.character;
    if (!sheet) {
      return [];
    }
    return [
      {
        label: "Truques",
        spells: sheet.spells.filter((spell) => spell.level === 0),
      },
      {
        label: "1o circulo",
        spells: sheet.spells.filter((spell) => spell.level === 1),
      },
    ];
  }, [workspace]);

  const selectedToken = useMemo(() => {
    const map = workspace?.map;
    if (!map) {
      return null;
    }
    return map.tokens.find((token) => token.id === selectedTokenId) ?? map.tokens[0] ?? null;
  }, [selectedTokenId, workspace]);

  if (loading) {
    return <main className="app-shell loading">Carregando ficha...</main>;
  }

  if (error || !workspace) {
    return (
      <main className="app-shell loading">
        <h1>RPG Engine</h1>
        <p>{error ?? "Nao foi possivel carregar a ficha."}</p>
      </main>
    );
  }

  const { campaign, character: sheet } = workspace;

  return (
    <main className="app-shell">
      <section className="campaign-bar" aria-label="Campanha">
        <div>
          <span>Campanha</span>
          <strong>{campaign.name}</strong>
        </div>
        <div>
          <span>Estado</span>
          <strong>SQLite local</strong>
        </div>
      </section>

      <section className="sheet-header">
        <div>
          <p className="eyebrow">{sheet.rulesetId}</p>
          <h1>{sheet.name}</h1>
          <p className="subhead">
            {sheet.race} {sheet.className} nivel {sheet.level}
          </p>
        </div>
        <div className="stat-strip">
          <div>
            <span>PV max</span>
            <strong>{sheet.hpMax}</strong>
          </div>
          <div>
            <span>Prof</span>
            <strong>{modifierText(sheet.proficiencyBonus)}</strong>
          </div>
        </div>
      </section>

      <section className="ability-grid" aria-label="Atributos">
        {sheet.abilityScores.map((ability) => (
          <article className="ability-tile" key={ability.id}>
            <span>{ability.label}</span>
            <strong>{ability.score}</strong>
            <em>{modifierText(ability.modifier)}</em>
          </article>
        ))}
      </section>

      <section className="workspace-grid">
        <section className="panel">
          <div className="panel-heading">
            <h2>Magias</h2>
          </div>
          <div className="stack">
            {spellGroups.map((group) => (
              <div className="list-group" key={group.label}>
                <h3>{group.label}</h3>
                {group.spells.map((spell) => (
                  <article className="list-row" key={spell.name}>
                    <div>
                      <strong>{spell.name}</strong>
                      <span>{spell.school}</span>
                    </div>
                    <span>
                      {spell.damageFormula
                        ? `${spell.damageFormula} ${spell.damageType ?? ""}`
                        : "utilidade"}
                    </span>
                  </article>
                ))}
              </div>
            ))}
          </div>
        </section>

        <section className="panel">
          <div className="panel-heading">
            <h2>Equipamento</h2>
          </div>
          <div className="stack">
            {sheet.items.map((item) => (
              <article className="list-row" key={item.name}>
                <div>
                  <strong>{item.name}</strong>
                  <span>{item.itemType}</span>
                </div>
                <span>
                  {item.damageFormula
                    ? `${item.damageFormula} ${item.damageType ?? ""}`
                    : "sem dano"}
                </span>
              </article>
            ))}
          </div>
        </section>

        <section className="panel">
          <div className="panel-heading panel-heading-row">
            <h2>Combate</h2>
            {workspace.combat ? (
              <button
                className="text-button"
                disabled={combatBusy}
                type="button"
                onClick={() => runCombatCommand("advance_combat_turn")}
              >
                Proximo turno
              </button>
            ) : (
              <button
                className="text-button"
                disabled={combatBusy}
                type="button"
                onClick={() => runCombatCommand("start_basic_combat")}
              >
                Iniciar
              </button>
            )}
          </div>
          {workspace.combat ? (
            <div className="initiative-list">
              {workspace.combat.participants.map((participant) => (
                <article
                  className={
                    participant.isCurrentTurn
                      ? "initiative-row current"
                      : "initiative-row"
                  }
                  key={participant.id}
                >
                  <div>
                    <strong>{participant.name}</strong>
                    <span>
                      {participant.isCurrentTurn ? "turno atual" : "aguardando"}
                    </span>
                  </div>
                  <em>{participant.initiative}</em>
                </article>
              ))}
            </div>
          ) : (
            <p className="empty-state">Nenhum combate ativo.</p>
          )}
        </section>

        <section className="panel map-panel">
          <div className="panel-heading panel-heading-row">
            <h2>Mapa</h2>
            <button
              className="text-button"
              disabled={mapBusy}
              type="button"
              onClick={openMap}
            >
              {workspace.map ? "Recarregar" : "Abrir"}
            </button>
          </div>
          {workspace.map ? (
            <div className="map-stack">
              <div className="token-picker" aria-label="Tokens do mapa">
                {workspace.map.tokens.map((token) => (
                  <button
                    className={
                      selectedToken?.id === token.id
                        ? "token-chip selected"
                        : "token-chip"
                    }
                    disabled={mapBusy}
                    key={token.id}
                    type="button"
                    onClick={() => setSelectedTokenId(token.id)}
                  >
                    {token.name}
                  </button>
                ))}
              </div>
              <div
                className="map-grid"
                style={{
                  gridTemplateColumns: `repeat(${workspace.map.width}, minmax(0, 1fr))`,
                }}
              >
                {Array.from({ length: workspace.map.width * workspace.map.height }).map(
                  (_, index) => {
                    const x = index % workspace.map!.width;
                    const y = Math.floor(index / workspace.map!.width);
                    const token = workspace.map!.tokens.find(
                      (candidate) => candidate.x === x && candidate.y === y,
                    );
                    const isSelected = token && selectedToken?.id === token.id;
                    return (
                      <button
                        aria-label={`Celula ${x + 1}, ${y + 1}`}
                        className={
                          token
                            ? isSelected
                              ? "map-cell occupied selected"
                              : "map-cell occupied"
                            : "map-cell"
                        }
                        disabled={mapBusy}
                        key={`${x}-${y}`}
                        type="button"
                        onClick={() => {
                          if (token) {
                            setSelectedTokenId(token.id);
                            return;
                          }
                          if (selectedToken) {
                            moveToken(selectedToken.id, x, y);
                          }
                        }}
                      >
                        {token ? token.name.slice(0, 2).toUpperCase() : ""}
                      </button>
                    );
                  },
                )}
              </div>
            </div>
          ) : (
            <p className="empty-state">Nenhuma cena aberta.</p>
          )}
        </section>

        <section className="panel">
          <div className="panel-heading">
            <h2>Bestiario</h2>
          </div>
          <form className="homebrew-form" onSubmit={createHomebrewMonster}>
            <div className="form-grid">
              <label>
                <span>Nome</span>
                <input
                  required
                  value={homebrewDraft.name}
                  onChange={(event) => updateHomebrew("name", event.target.value)}
                />
              </label>
              <label>
                <span>Tipo</span>
                <input
                  required
                  value={homebrewDraft.creatureType}
                  onChange={(event) =>
                    updateHomebrew("creatureType", event.target.value)
                  }
                />
              </label>
              <label>
                <span>Tamanho</span>
                <select
                  value={homebrewDraft.size}
                  onChange={(event) => updateHomebrew("size", event.target.value)}
                >
                  <option>Tiny</option>
                  <option>Small</option>
                  <option>Medium</option>
                  <option>Large</option>
                  <option>Huge</option>
                  <option>Gargantuan</option>
                </select>
              </label>
              <label>
                <span>ND</span>
                <input
                  required
                  value={homebrewDraft.challengeRating}
                  onChange={(event) =>
                    updateHomebrew("challengeRating", event.target.value)
                  }
                />
              </label>
            </div>
            <label>
              <span>Descricao</span>
              <textarea
                rows={2}
                value={homebrewDraft.description}
                onChange={(event) =>
                  updateHomebrew("description", event.target.value)
                }
              />
            </label>
            <div className="number-grid">
              {[
                ["CA", "armorClass"],
                ["PV", "hitPoints"],
                ["Desloc.", "speed"],
                ["STR", "strScore"],
                ["DEX", "dexScore"],
                ["CON", "conScore"],
                ["INT", "intScore"],
                ["WIS", "wisScore"],
                ["CHA", "chaScore"],
              ].map(([label, key]) => (
                <label key={key}>
                  <span>{label}</span>
                  <input
                    min={key === "speed" ? 0 : 1}
                    required
                    type="number"
                    value={homebrewDraft[key as keyof HomebrewMonsterDraft] as number}
                    onChange={(event) =>
                      updateHomebrew(key as keyof HomebrewMonsterDraft, Number(event.target.value))
                    }
                  />
                </label>
              ))}
            </div>
            <div className="form-grid">
              <label>
                <span>Acao</span>
                <input
                  required
                  value={homebrewDraft.actionName}
                  onChange={(event) =>
                    updateHomebrew("actionName", event.target.value)
                  }
                />
              </label>
              <label>
                <span>Ataque</span>
                <input
                  type="number"
                  value={homebrewDraft.attackBonus}
                  onChange={(event) =>
                    updateHomebrew("attackBonus", Number(event.target.value))
                  }
                />
              </label>
              <label>
                <span>Dano</span>
                <input
                  required
                  value={homebrewDraft.damageFormula}
                  onChange={(event) =>
                    updateHomebrew("damageFormula", event.target.value)
                  }
                />
              </label>
              <label>
                <span>Tipo dano</span>
                <input
                  required
                  value={homebrewDraft.damageType}
                  onChange={(event) =>
                    updateHomebrew("damageType", event.target.value)
                  }
                />
              </label>
            </div>
            <button className="text-button" disabled={homebrewBusy} type="submit">
              Criar monstro
            </button>
          </form>
          {bestiary.length > 0 ? (
            <div className="monster-list">
              {bestiary.map((monster) => (
                <article className="monster-row" key={monster.id}>
                  <div className="monster-title">
                    <div>
                      <strong>{monster.name}</strong>
                      <span>
                        {monster.size} {monster.creatureType} - ND{" "}
                        {monster.challengeRating}
                      </span>
                    </div>
                    <div className="monster-stats">
                      <span>CA {monster.armorClass}</span>
                      <span>PV {monster.hitPoints}</span>
                    </div>
                  </div>
                  <p>{monster.description}</p>
                  <div className="monster-actions">
                    {monster.actions.map((action) => (
                      <span key={action.name}>
                        {action.name}
                        {action.damageFormula
                          ? ` ${action.damageFormula} ${action.damageType ?? ""}`
                          : ""}
                      </span>
                    ))}
                  </div>
                </article>
              ))}
            </div>
          ) : (
            <p className="empty-state">Nenhum monstro no content-pack.</p>
          )}
        </section>
      </section>
    </main>
  );
}

export default App;
