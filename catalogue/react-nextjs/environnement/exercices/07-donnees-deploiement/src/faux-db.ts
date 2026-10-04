// Une « fausse » connexion pour l'exercice : Kysely écrit le SQL comme d'habitude, mais rien n'est envoyé à PostgreSQL.
// Chaque requête est simplement notée dans `requetes`, pour que les tests puissent la lire.
import { DummyDriver, Kysely, PostgresAdapter, PostgresIntrospector, PostgresQueryCompiler } from "kysely";
import type { DB } from "./types";

export type Requete = { sql: string; parameters: readonly unknown[] };

export function creerFausseDb() {
  const requetes: Requete[] = [];
  const db = new Kysely<DB>({
    dialect: {
      createAdapter: () => new PostgresAdapter(),
      createDriver: () => new DummyDriver(),
      createIntrospector: (base) => new PostgresIntrospector(base),
      createQueryCompiler: () => new PostgresQueryCompiler(),
    },
    log: (evenement) => {
      requetes.push({ sql: evenement.query.sql, parameters: evenement.query.parameters });
    },
  });
  return { db, requetes };
}
