import { describe, expect, it } from "vitest";
import { retirer, totalArticles, totalCents, type Ligne } from "./panier";

const panier: Ligne[] = Object.freeze([
  Object.freeze({ id: 1, nom: "Gourde Éco", prixCents: 1250, stock: 5, quantite: 2 }),
  Object.freeze({ id: 2, nom: "Tote bag", prixCents: 600, stock: 5, quantite: 1 }),
]) as Ligne[];

describe("retirer", () => {
  it("renvoie un nouveau tableau sans la ligne demandée", () => {
    const resultat = retirer(panier, 1);
    expect(resultat.map((l) => l.id)).toEqual([2]);
    expect(resultat).not.toBe(panier);
    expect(panier).toHaveLength(2);
  });

  it("ne change rien si l'identifiant est inconnu", () => {
    expect(retirer(panier, 99).map((l) => l.id)).toEqual([1, 2]);
  });
});

describe("totaux", () => {
  it("compte les articles", () => {
    expect(totalArticles(panier)).toBe(3);
    expect(totalArticles([])).toBe(0);
  });

  it("additionne les prix en centimes", () => {
    expect(totalCents(panier)).toBe(2 * 1250 + 600);
    expect(totalCents([])).toBe(0);
  });
});
