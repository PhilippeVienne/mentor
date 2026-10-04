// La description des tables de la base, comme celle que `kysely-codegen` écrit dans `src/db/types.ts` de MiniShop.
import type { Generated } from "kysely";

export interface ProductTable {
  id: Generated<number>;
  name: string;
  enabled: boolean;
}

export interface ProductVersionTable {
  id: Generated<number>;
  product_id: number;
  stock: number;
}

export interface DB {
  product: ProductTable;
  product_version: ProductVersionTable;
}
