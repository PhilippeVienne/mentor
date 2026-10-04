import { describe, expect, it } from "vitest";
import PageProduit from "../app/[boutique]/produit/[id]/page";

const parametres = (id: string) => Promise.resolve({ boutique: "bde", id });

// `notFound()` lève une erreur spéciale : Next.js la reconnaît à son `digest` et affiche la page 404.
const estUne404 = { digest: expect.stringContaining("404") };

describe("page produit : produit introuvable", () => {
  it("lance notFound() pour un identifiant inconnu", async () => {
    await expect(PageProduit({ params: parametres("999") })).rejects.toMatchObject(estUne404);
  });

  it("lance notFound() quand l'identifiant n'est pas un nombre", async () => {
    await expect(PageProduit({ params: parametres("abc") })).rejects.toMatchObject(estUne404);
  });

  it("n'a pas de page 404 pour un produit qui existe", async () => {
    await expect(PageProduit({ params: parametres("12") })).resolves.toBeDefined();
  });
});
