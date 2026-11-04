import type { Draft } from "./steps";

export function DoneStep({ draft }: { draft: Draft }) {
  if (draft.role === "client") {
    return (
      <>
        <h2>Ready to connect</h2>
        <p>
          {draft.deviceName} will pair with your host and mount its shelves as a drive. Nothing gets downloaded until you
          open it.
        </p>
      </>
    );
  }
  const where = draft.useDomain ? draft.domain : `port ${draft.port}`;
  return (
    <>
      <h2>Your host is ready</h2>
      <dl className="summary">
        <dt>Name</dt>
        <dd>{draft.deviceName}</dd>
        <dt>Library</dt>
        <dd>{draft.library}</dd>
        <dt>Reachable on</dt>
        <dd>{where}</dd>
      </dl>
      <p>Next up: make an invite for your first client from the Devices page.</p>
    </>
  );
}
