import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ChoixQuantite } from "./ChoixQuantite";

describe("ChoixQuantite : ajout au panier", () => {
  it("prévient le parent avec la quantité choisie", async () => {
    const onAjouter = vi.fn();
    render(<ChoixQuantite stock={5} onAjouter={onAjouter} />);
    await userEvent.click(screen.getByRole("button", { name: "+" }));
    await userEvent.click(screen.getByRole("button", { name: "+" }));
    await userEvent.click(screen.getByRole("button", { name: "Ajouter au panier" }));
    expect(onAjouter).toHaveBeenCalledTimes(1);
    expect(onAjouter).toHaveBeenCalledWith(3);
  });

  it("revient à 1 après l'ajout", async () => {
    render(<ChoixQuantite stock={5} onAjouter={vi.fn()} />);
    await userEvent.click(screen.getByRole("button", { name: "+" }));
    await userEvent.click(screen.getByRole("button", { name: "Ajouter au panier" }));
    expect(screen.getByText("1")).toBeInTheDocument();
    expect(screen.queryByText("2")).not.toBeInTheDocument();
  });
});
