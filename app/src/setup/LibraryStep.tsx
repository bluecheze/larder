import { open } from "@tauri-apps/plugin-dialog";
import { Field } from "../ui/Field";
import { Icon } from "../ui/Icon";

type Props = { library: string; onLibrary: (path: string) => void };

export function LibraryStep({ library, onLibrary }: Props) {
  async function choose() {
    const picked = await open({ directory: true, defaultPath: library || undefined });
    if (typeof picked === "string") onLibrary(picked);
  }

  return (
    <>
      <h2>Where should the library live?</h2>
      <p>Every shelf becomes a folder in here. An external drive works fine, just keep it plugged in.</p>
      <div className="row">
        <Field label="Library folder" value={library} onChange={onLibrary} />
        <button type="button" className="btn" onClick={choose}>
          <Icon name="folder" /> Choose
        </button>
      </div>
    </>
  );
}
