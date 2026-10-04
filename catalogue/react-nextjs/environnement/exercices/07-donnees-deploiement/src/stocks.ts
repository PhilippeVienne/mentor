import type { Kysely } from "kysely";
import type { DB } from "./types";

// À corriger (étape 1) : les noms et les stocks des produits ACTIFS, triés par identifiant de produit.
// Il faut joindre `product_version` à `product`, filtrer sur `product.enabled`, et ne choisir que deux colonnes.
export async function stocksActifs(db: Kysely<DB>) {
  return db.selectFrom("product").selectAll().execute();
}
