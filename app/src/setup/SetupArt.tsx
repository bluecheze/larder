import jar from "../assets/icon-shelf.png";
import scene from "../assets/scene.png";
import type { StepId } from "./steps";

const LABELS: Record<StepId, string> = {
  role: "This computer",
  name: "Name",
  library: "Library",
  network: "Connection",
  invite: "Invite",
  done: "Finish",
};

type Props = { steps: StepId[]; current: number };

export function SetupArt({ steps, current }: Props) {
  return (
    <aside className="setup-side">
      <div className="mark">
        <img src={jar} alt="" />
        larder
      </div>
      <ol className="steps" aria-label="Setup steps">
        {steps.map((step, i) => {
          const state = i < current ? "done" : i === current ? "now" : "later";
          return (
            <li key={step} data-state={state} aria-current={state === "now" ? "step" : undefined}>
              <span className="steps__dot">{state === "done" ? "✓" : i + 1}</span>
              {LABELS[step]}
            </li>
          );
        })}
      </ol>
      <img className="setup-side__scene" src={scene} alt="" />
    </aside>
  );
}
