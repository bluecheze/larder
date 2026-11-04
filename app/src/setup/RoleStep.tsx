import crate from "../assets/icon-library.png";
import folder from "../assets/icon-drive.png";
import type { Role } from "../lib/api";
import { Choice } from "../ui/Choice";

type Props = { role: Role; onRole: (role: Role) => void };

export function RoleStep({ role, onRole }: Props) {
  return (
    <>
      <h2>What's this computer for?</h2>
      <div className="choices" role="group" aria-label="Role">
        <Choice
          selected={role === "host"}
          onSelect={() => onRole("host")}
          title="The host"
          art={<img src={crate} alt="" />}
        >
          Keeps the library. Your other computers pair with it.
        </Choice>
        <Choice
          selected={role === "client"}
          onSelect={() => onRole("client")}
          title="A client"
          art={<img src={folder} alt="" />}
        >
          Shows the host's files as a drive and only downloads what you open.
        </Choice>
      </div>
      <p className="note">First time? Start with the host, a client needs an invite from it. You can switch later in Settings.</p>
    </>
  );
}
