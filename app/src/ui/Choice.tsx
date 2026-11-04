import type { ReactNode } from "react";
import "./choice.css";

type Props = {
  selected: boolean;
  onSelect: () => void;
  title: string;
  children: ReactNode;
  art: ReactNode;
};

// A pick-one card. aria-pressed makes screen readers announce it as a
// toggle, the group around it carries the label.
export function Choice({ selected, onSelect, title, children, art }: Props) {
  return (
    <button type="button" className="choice" aria-pressed={selected} onClick={onSelect}>
      <span className="choice__art">{art}</span>
      <span className="choice__title">{title}</span>
      <span className="choice__body">{children}</span>
    </button>
  );
}
