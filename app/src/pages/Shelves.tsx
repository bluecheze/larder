import { useCallback, useEffect, useState, type FormEvent } from "react";
import { api } from "../lib/api";
import { Field } from "../ui/Field";
import { Icon } from "../ui/Icon";
import "./pages.css";

export function Shelves() {
  const [shelves, setShelves] = useState<string[]>([]);
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const refresh = useCallback(() => api.hostShelves().then(setShelves), []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  async function create(e: FormEvent) {
    e.preventDefault();
    setError(null);
    try {
      await api.createShelf(name.trim());
      setName("");
      refresh();
    } catch (err) {
      setError(String(err));
    }
  }

  return (
    <section className="page">
      <header className="page__head">
        <h1>Shelves</h1>
        <p>Top level folders in your library. Clients see each one as a folder on their Larder drive.</p>
      </header>
      <form className="card panel" onSubmit={create}>
        <h2>New shelf</h2>
        <div className="inline">
          <Field label="Name" value={name} onChange={setName} placeholder="Documents" />
          <button type="submit" className="btn btn--primary" disabled={!name.trim()}>
            <Icon name="plus" size={18} /> Add shelf
          </button>
        </div>
        {error && <p className="field__error">{error}</p>}
      </form>
      <div className="card panel">
        <h2>In the library</h2>
        {shelves.length === 0 ? (
          <p className="empty">No shelves yet. Folders you drop straight into the library show up here too.</p>
        ) : (
          <ul className="list">
            {shelves.map((shelf) => (
              <li key={shelf}>
                <Icon name="folder" size={18} />
                <span className="list__main">
                  <strong>{shelf}</strong>
                </span>
              </li>
            ))}
          </ul>
        )}
      </div>
    </section>
  );
}
