import type { Kysely } from "kysely";
import type { DB } from "../types";

// À écrire (étapes 2 et 3) : la migration qui ajoute à `product` une colonne entière `seuil_alerte`,
// obligatoire (NOT NULL) et valant 0 par défaut ; `down` doit la supprimer.
export async function up(db: Kysely<DB>): Promise<void> {}

export async function down(db: Kysely<DB>): Promise<void> {}
