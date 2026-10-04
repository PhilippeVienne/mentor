import { describe, expect, it } from "vitest";
import { ajouter, type Ligne } from "./panier";

// Gèle un panier : toute modification en place lèverait une erreur.
const geler = (panier: Ligne[]): Ligne[] => Object.freeze(panier.map((l) => Object.freeze({ ...l }))) as Ligne[];

const gourde = { id: 1, nom: "Gourde Éco", prixCents: 1250, stock: 2 };

describe("ajouter", () => {
  it("ajoute une nouvelle ligne de quantité 1", () => {
    const panier = ajouter(geler([]), gourde);
    expect(panier).toEqual([{ ...gourde, quantite: 1 }]);
  });

  it("augmente la quantité d'un article déjà présent", () => {
    const p1 = ajouter(geler([]), gourde);
    const p2 = ajouter(geler(p1), gourde);
    expect(p2[0].quantite).toBe(2);
    expect(p1[0].quantite).toBe(1);
    expect(p2).not.toBe(p1);
  });

  it("ne dépasse jamais le stock", () => {
    let panier: Ligne[] = [];
    for (let i = 0; i < 5; i++) panier = ajouter(geler(panier), gourde);
    expect(panier).toHaveLength(1);
    expect(panier[0].quantite).toBe(2);
  });

  it("ignore un article épuisé", () => {
    const vide = geler([]);
    expect(ajouter(vide, { ...gourde, id: 2, stock: 0 })).toEqual([]);
  });
});
