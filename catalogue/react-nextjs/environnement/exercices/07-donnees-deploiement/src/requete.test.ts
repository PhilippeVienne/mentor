import { describe, expect, it } from "vitest";
import { creerFausseDb } from "./faux-db";
import { stocksActifs } from "./stocks";

describe("stocksActifs", () => {
  it("joint les versions aux produits et ne garde que les produits actifs", async () => {
    const { db, requetes } = creerFausseDb();
    await stocksActifs(db);
    expect(requetes).toHaveLength(1);
    const { sql, parameters } = requetes[0];
    expect(sql).toMatch(
      /inner join "(product|product_version)" on ("product"\."id" = "product_version"\."product_id"|"product_version"\."product_id" = "product"\."id")/,
    );
    expect(sql).toMatch(/where "product"\."enabled" = \$1/);
    expect(parameters).toEqual([true]);
  });

  it("choisit seulement le nom et le stock, triés par identifiant de produit", async () => {
    const { db, requetes } = creerFausseDb();
    await stocksActifs(db);
    const { sql } = requetes[0];
    expect(sql).toMatch(/^select "product"\."name", "product_version"\."stock" from /);
    expect(sql).toMatch(/order by "product"\."id" asc$/);
    expect(sql).not.toContain("*");
  });
});
