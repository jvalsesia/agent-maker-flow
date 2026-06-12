import { UserButton } from "@clerk/clerk-react";
import { NavLink } from "react-router-dom";

import { ThemeToggle } from "./ui/ThemeToggle";
import styles from "./NavBar.module.css";

const ITEMS = [
  { to: "/agents", label: "Agents" },
  { to: "/flows", label: "Flows" },
  { to: "/settings", label: "Settings" },
] as const;

const linkClass = ({ isActive }: { isActive: boolean }) =>
  isActive ? `${styles.link} ${styles.active}` : styles.link;

/** Primary navigation as a segmented control (active = accent-subtle pill +
 * aria-current via NavLink), plus the theme toggle and Clerk session control. */
export function NavBar() {
  return (
    <nav className={styles.nav} aria-label="Primary">
      <div className={styles.segments}>
        {ITEMS.map((item) => (
          <NavLink key={item.to} to={item.to} className={linkClass}>
            {item.label}
          </NavLink>
        ))}
      </div>
      <div className={styles.controls}>
        <ThemeToggle />
        <UserButton afterSignOutUrl="/sign-in" />
      </div>
    </nav>
  );
}
