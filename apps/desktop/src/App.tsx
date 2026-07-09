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

function modifierText(value: number) {
  return value >= 0 ? `+${value}` : `${value}`;
}

function App() {
  const [sheet, setSheet] = useState<CharacterSheet | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<CharacterSheet>("load_character_sheet")
      .then((loaded) => {
        setSheet(loaded);
        setError(null);
      })
      .catch((err) => {
        setError(String(err));
      })
      .finally(() => {
        setLoading(false);
      });
  }, []);

  const spellGroups = useMemo(() => {
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
  }, [sheet]);

  if (loading) {
    return <main className="app-shell loading">Carregando ficha...</main>;
  }

  if (error || !sheet) {
    return (
      <main className="app-shell loading">
        <h1>RPG Engine</h1>
        <p>{error ?? "Nao foi possivel carregar a ficha."}</p>
      </main>
    );
  }

  return (
    <main className="app-shell">
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
      </section>
    </main>
  );
}

export default App;
