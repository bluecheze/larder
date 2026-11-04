import { useState } from "react";
import { api, type AppConfig, type Defaults, type Role } from "../lib/api";
import { Icon } from "../ui/Icon";
import { DoneStep } from "./DoneStep";
import { InviteStep } from "./InviteStep";
import { LibraryStep } from "./LibraryStep";
import { NameStep } from "./NameStep";
import { NetworkStep } from "./NetworkStep";
import { RoleStep } from "./RoleStep";
import { SetupArt } from "./SetupArt";
import { initialDraft, problem, stepsFor, toConfig, type Draft, type StepId } from "./steps";
import "./setup.css";

type Props = { defaults: Defaults; onDone: (config: AppConfig) => void };

export function Setup({ defaults, onDone }: Props) {
  const [draft, setDraft] = useState<Draft>(() => initialDraft(defaults));
  const [index, setIndex] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const steps = stepsFor(draft.role);
  const step = steps[index];
  const last = index === steps.length - 1;
  const patch = (change: Partial<Draft>) => {
    setDraft((d) => ({ ...d, ...change }));
    setError(null);
  };

  async function next() {
    const issue = problem(step, draft);
    if (issue) return setError(issue);
    if (!last) return setIndex(index + 1);
    setSaving(true);
    try {
      const config = toConfig(draft);
      await api.saveConfig(config);
      onDone(config);
    } catch (e) {
      setError(String(e));
      setSaving(false);
    }
  }

  return (
    <main className="setup">
      <SetupArt steps={steps} current={index} />
      <div className="setup-main">
      <section className="setup-card card" aria-live="polite">
        <div className="setup-card__body" key={step}>
          {renderStep(step, draft, patch)}
        </div>
        {error && (
          <p className="field__error" role="alert">
            {error}
          </p>
        )}
        <footer className="setup-card__footer">
          {index > 0 && (
            <button type="button" className="btn" onClick={() => setIndex(index - 1)}>
              <Icon name="arrowLeft" size={18} /> Back
            </button>
          )}
          <button type="button" className="btn btn--primary" onClick={next} disabled={saving}>
            {last ? "Start Larder" : "Continue"} <Icon name={last ? "check" : "arrowRight"} size={18} />
          </button>
        </footer>
      </section>
      </div>
    </main>
  );
}

function renderStep(step: StepId, draft: Draft, patch: (change: Partial<Draft>) => void) {
  switch (step) {
    case "role":
      return <RoleStep role={draft.role} onRole={(role: Role) => patch({ role })} />;
    case "name":
      return <NameStep name={draft.deviceName} onName={(deviceName) => patch({ deviceName })} />;
    case "library":
      return <LibraryStep library={draft.library} onLibrary={(library) => patch({ library })} />;
    case "network":
      return <NetworkStep draft={draft} onChange={patch} />;
    case "invite":
      return <InviteStep invite={draft.invite} onInvite={(invite) => patch({ invite })} />;
    case "done":
      return <DoneStep draft={draft} />;
  }
}
