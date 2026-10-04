import type { ReactNode } from "react";

// Le cadre commun à toutes les pages du site : il reçoit la page courante dans `children`.
export default function RootLayout({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <html lang="fr">
      <body>{children}</body>
    </html>
  );
}
