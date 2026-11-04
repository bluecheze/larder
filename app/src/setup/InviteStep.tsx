import { Field } from "../ui/Field";

type Props = { invite: string; onInvite: (invite: string) => void };

export function InviteStep({ invite, onInvite }: Props) {
  return (
    <>
      <h2>Paste your invite</h2>
      <p>On the host, open Devices and make an invite. It's good for one computer and runs out after 15 minutes.</p>
      <Field label="Invite" value={invite} onChange={onInvite} autoFocus placeholder="larder://..." />
    </>
  );
}
