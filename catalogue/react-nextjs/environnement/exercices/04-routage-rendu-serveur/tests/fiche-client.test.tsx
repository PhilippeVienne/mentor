import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import PageProduit from "../app/[boutique]/produit/[id]/page";

describe("fiche produit interactive", () => {
  it("affiche la quantité et la fait monter au clic", async () => {
    render(await PageProduit({ params: Promise.resolve({ boutique: "bde", id: "12" }) }));
    const quantite = screen.getByRole("status");
    expect(quantite).toHaveTextContent("1");
    await userEvent.click(screen.getByRole("button", { name: "Une de plus" }));
    await userEvent.click(screen.getByRole("button", { name: "Une de plus" }));
    expect(quantite).toHaveTextContent("3");
  });
});
