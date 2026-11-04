import "./switch.css";

type Props = { on: boolean; onChange: (on: boolean) => void; label: string; disabled?: boolean };

export function Switch({ on, onChange, label, disabled }: Props) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-label={label}
      className="switch"
      disabled={disabled}
      onClick={() => onChange(!on)}
    >
      <span className="switch__knob" />
    </button>
  );
}
