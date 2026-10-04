import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { GoodieCard } from "./GoodieCard";

describe("GoodieCard", () => {
  it("affiche le nom dans un titre de niveau 3", () => {
    render(<GoodieCard nom="Gourde Éco" prixCents={1250} stock={8} />);
    expect(screen.getByRole("heading", { level: 3, name: "Gourde Éco" })).toBeInTheDocument();
  });

  it("affiche le prix à la française", () => {
    render(<GoodieCard nom="Gourde Éco" prixCents={1250} stock={8} />);
    expect(screen.getByText(/12,50\s€/)).toBeInTheDocument();
  });

  it("affiche le stock restant", () => {
    render(<GoodieCard nom="Gourde Éco" prixCents={1250} stock={8} />);
    expect(screen.getByText("8 en stock")).toBeInTheDocument();
    expect(screen.queryByText("Épuisé")).not.toBeInTheDocument();
  });

  it("affiche « Épuisé » quand le stock est nul", () => {
    render(<GoodieCard nom="Tote bag" prixCents={600} stock={0} />);
    expect(screen.getByText("Épuisé")).toBeInTheDocument();
    expect(screen.queryByText(/en stock/)).not.toBeInTheDocument();
  });
});
