import { Field } from "../ui/Field";

type Props = { name: string; onName: (name: string) => void };

export function NameStep({ name, onName }: Props) {
  return (
    <>
      <h2>Name this computer</h2>
      <Field
        label="Computer name"
        value={name}
        onChange={onName}
        autoFocus
        hint="Shows up in the device list and in conflict copies, like report (conflict, laptop).pdf"
      />
    </>
  );
}
