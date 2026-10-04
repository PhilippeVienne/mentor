import { describe, expect, it } from "vitest";
import { generateMetadata } from "../app/[boutique]/produit/[id]/page";

const parametres = (id: string) => Promise.resolve({ boutique: "bde", id });

describe("generateMetadata", () => {
  it("prend le titre et la description du produit", async () => {
    const meta = await generateMetadata({ params: parametres("12") });
    expect(meta.title).toBe("Gourde Éco");
    expect(meta.description).toBe("Inox, 50 cl.");
  });

  it("annonce un produit introuvable sans planter", async () => {
    const meta = await generateMetadata({ params: parametres("999") });
    expect(meta.title).toBe("Produit introuvable");
    expect(meta.description).toBeUndefined();
  });
});
