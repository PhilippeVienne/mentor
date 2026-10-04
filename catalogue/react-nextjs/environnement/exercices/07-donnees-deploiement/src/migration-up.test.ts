import { describe, expect, it } from "vitest";
import { creerFausseDb } from "./faux-db";
import { up } from "./migrations/015_seuil_alerte";

describe("migration 015 : up", () => {
  it("ajoute à product une colonne entière seuil_alerte, obligatoire, à 0 par défaut", async () => {
    const { db, requetes } = creerFausseDb();
    await up(db);
    expect(requetes).toHaveLength(1);
    const { sql } = requetes[0];
    expect(sql).toMatch(/^alter table "product" add column "seuil_alerte" integer/);
    expect(sql).toMatch(/default 0/);
    expect(sql).toMatch(/not null/);
  });
});
