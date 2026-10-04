import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import PageProduit from "../app/[boutique]/produit/[id]/page";

const parametres = (id: string) => Promise.resolve({ boutique: "bde", id });

describe("page produit", () => {
  it("affiche le produit demandé", async () => {
    render(await PageProduit({ params: parametres("12") }));
    expect(screen.getByRole("heading", { level: 1, name: "Gourde Éco" })).toBeInTheDocument();
  });

  it("affiche un autre produit pour un autre identifiant", async () => {
    render(await PageProduit({ params: parametres("13") }));
    expect(screen.getByRole("heading", { level: 1, name: "Tote bag" })).toBeInTheDocument();
  });
});
