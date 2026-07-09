import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type DiceRollSummary = {
  formula: string;
  total: number;
  breakdown: string;
};

const quickDice = [4, 6, 8, 10, 12, 20, 100];

function formulaDieSides(value: string) {
  const match = value.trim().match(/^1d(4|6|8|10|12|20|100)$/i);
  return match ? Number(match[1]) : null;
}

function d6Pips(total: number) {
  if (total < 1 || total > 6) {
    return null;
  }

  const layouts: Record<number, number[]> = {
    1: [5],
    2: [1, 9],
    3: [1, 5, 9],
    4: [1, 3, 7, 9],
    5: [1, 3, 5, 7, 9],
    6: [1, 3, 4, 6, 7, 9],
  };

  return (
    <div className="d6-pips" aria-hidden="true">
      {Array.from({ length: 9 }).map((_, index) => (
        <span
          className={layouts[total].includes(index + 1) ? "pip active" : "pip"}
          key={index}
        />
      ))}
    </div>
  );
}

function DiceRoller() {
  const [formula, setFormula] = useState("2d6+3");
  const [advantage, setAdvantage] = useState(false);
  const [disadvantage, setDisadvantage] = useState(false);
  const [result, setResult] = useState<DiceRollSummary | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [visualSides, setVisualSides] = useState<number | null>(20);
  const [rollKey, setRollKey] = useState(0);

  function roll(nextFormula: string) {
    const cleanFormula = nextFormula.trim();
    if (!cleanFormula) {
      setError("Informe uma formula para rolar.");
      return;
    }

    setBusy(true);
    invoke<DiceRollSummary>("roll_formula", {
      formula: cleanFormula,
      advantage,
      disadvantage,
    })
      .then((summary) => {
        setResult(summary);
        setVisualSides(formulaDieSides(cleanFormula));
        setRollKey((current) => current + 1);
        setError(null);
      })
      .catch((err) => {
        setError(String(err));
      })
      .finally(() => {
        setBusy(false);
      });
  }

  return (
    <section className="panel dice-panel">
      <div className="panel-heading">
        <h2>Dados</h2>
      </div>
      <div className="dice-stage" aria-live="polite">
        <div
          className={`visual-die ${rollKey > 0 ? "rolling" : ""}`}
          key={rollKey}
          data-sides={visualSides ?? "formula"}
        >
          <span className="die-kind">
            {visualSides ? `d${visualSides}` : "total"}
          </span>
          {visualSides === 6 && result ? (
            d6Pips(result.total)
          ) : (
            <strong>{result ? result.total : "?"}</strong>
          )}
        </div>
        <div className="quick-dice" aria-label="Dados rapidos">
          {quickDice.map((sides) => (
            <button
              className={
                visualSides === sides ? "die-button selected" : "die-button"
              }
              disabled={busy}
              key={sides}
              type="button"
              onClick={() => roll(`1d${sides}`)}
            >
              d{sides}
            </button>
          ))}
        </div>
      </div>
      <form
        className="dice-form"
        onSubmit={(event) => {
          event.preventDefault();
          roll(formula);
        }}
      >
        <label>
          <span>Formula</span>
          <input
            placeholder="2d6+3"
            value={formula}
            onChange={(event) => setFormula(event.target.value)}
          />
        </label>
        <button className="text-button" disabled={busy} type="submit">
          Rolar
        </button>
      </form>
      <div className="roll-mode" aria-label="Modo de rolagem">
        <label>
          <input
            checked={advantage}
            type="checkbox"
            onChange={(event) => {
              setAdvantage(event.target.checked);
              if (event.target.checked) {
                setDisadvantage(false);
              }
            }}
          />
          <span>Vantagem</span>
        </label>
        <label>
          <input
            checked={disadvantage}
            type="checkbox"
            onChange={(event) => {
              setDisadvantage(event.target.checked);
              if (event.target.checked) {
                setAdvantage(false);
              }
            }}
          />
          <span>Desvantagem</span>
        </label>
      </div>
      {result ? (
        <div className="roll-result">
          <span>{result.formula}</span>
          <strong>{result.total}</strong>
          <p>{result.breakdown}</p>
        </div>
      ) : (
        <p className="empty-state">Nenhuma rolagem ainda.</p>
      )}
      {error ? <p className="error-state">{error}</p> : null}
    </section>
  );
}

export default DiceRoller;
