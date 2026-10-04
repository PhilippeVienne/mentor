// Une fausse bibliothèque d'authentification, pour l'exercice. Dans MiniShop, `auth` vient de better-auth
// et `getSession` lit le cookie de session de la requête pour retrouver la personne connectée.
export type Role = "admin" | "caissier" | "user";
export type Session = { user: { id: string; role: Role } };

export const auth = {
  api: {
    async getSession(_options: { headers: Headers }): Promise<Session | null> {
      return null;
    },
  },
};
