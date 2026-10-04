import type { ReactNode } from "react";

// Un bouton sans style. À compléter (étape 3) : couleur de fond, survol et état désactivé.
export function Bouton({ children, disabled }: Readonly<{ children: ReactNode; disabled?: boolean }>) {
  return (
    <button type="button" disabled={disabled}>
      {children}
    </button>
  );
}
