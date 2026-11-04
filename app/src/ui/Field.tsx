import { useId } from "react";

type Props = {
  label: string;
  value: string;
  onChange: (value: string) => void;
  hint?: string;
  placeholder?: string;
  inputMode?: "text" | "numeric" | "email";
  autoFocus?: boolean;
};

export function Field({ label, value, onChange, hint, placeholder, inputMode = "text", autoFocus }: Props) {
  const id = useId();
  return (
    <div className="field">
      <label className="field__label" htmlFor={id}>
        {label}
      </label>
      <input
        id={id}
        className="field__input"
        value={value}
        placeholder={placeholder}
        inputMode={inputMode}
        autoFocus={autoFocus}
        spellCheck={false}
        onChange={(e) => onChange(e.target.value)}
      />
      {hint && <span className="field__hint">{hint}</span>}
    </div>
  );
}
