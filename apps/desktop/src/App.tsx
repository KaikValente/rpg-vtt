import { useEffect, useMemo, useState } from "react";
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

function modifierText(value: number) {
  return value >= 0 ? `+${value}` : `${value}`;
}

function App() {
  const [workspace, setWorkspace] = useState<CampaignWorkspace | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [combatBusy, setCombatBusy] = useState(false);

  useEffect(() => {
    invoke<CampaignWorkspace>("load_character_sheet")
      .then((loaded) => {
        setWorkspace(loaded);
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
      </section>
    </main>
  );
}

export default App;
