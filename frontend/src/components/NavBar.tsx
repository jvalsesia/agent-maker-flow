import { UserButton } from "@clerk/clerk-react";
import { NavLink } from "react-router-dom";

const linkStyle = ({ isActive }: { isActive: boolean }): React.CSSProperties => ({
  fontWeight: isActive ? 700 : 400,
  textDecoration: "none",
  marginRight: "1rem",
});

/** Primary navigation between the Agents, Flows, and Settings workspaces, plus
 * the Clerk session/sign-out control. */
export function NavBar() {
  return (
    <nav aria-label="Primary">
      <NavLink to="/agents" style={linkStyle}>
        Agents
      </NavLink>
      <NavLink to="/flows" style={linkStyle}>
        Flows
      </NavLink>
      <NavLink to="/settings" style={linkStyle}>
        Settings
      </NavLink>
      <UserButton afterSignOutUrl="/sign-in" />
    </nav>
  );
}
