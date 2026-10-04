import { describe, expect, it } from "vitest";
import { creerFausseDb } from "./faux-db";
import { down } from "./migrations/015_seuil_alerte";

describe("migration 015 : down", () => {
  it("supprime la colonne seuil_alerte de product", async () => {
    const { db, requetes } = creerFausseDb();
    await down(db);
    expect(requetes).toHaveLength(1);
    expect(requetes[0].sql).toBe('alter table "product" drop column "seuil_alerte"');
  });
});
