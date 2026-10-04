import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { FormulaireContact } from "./FormulaireContact";

describe("FormulaireContact : validation", () => {
  it("n'affiche pas d'erreur tant que le champ est vide", () => {
    render(<FormulaireContact onEnvoyer={vi.fn()} />);
    expect(screen.queryByText("Adresse invalide")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Envoyer" })).toBeDisabled();
  });

  it("signale une adresse invalide et bloque l'envoi", async () => {
    render(<FormulaireContact onEnvoyer={vi.fn()} />);
    await userEvent.type(screen.getByLabelText("Ton e-mail"), "pas-un-mail");
    expect(screen.getByText("Adresse invalide")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Envoyer" })).toBeDisabled();
  });

  it("accepte une adresse valide", async () => {
    render(<FormulaireContact onEnvoyer={vi.fn()} />);
    await userEvent.type(screen.getByLabelText("Ton e-mail"), "claire@example.org");
    expect(screen.queryByText("Adresse invalide")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Envoyer" })).toBeEnabled();
  });

  it("garde en mémoire ce qui est tapé (champ contrôlé)", async () => {
    render(<FormulaireContact onEnvoyer={vi.fn()} />);
    await userEvent.type(screen.getByLabelText("Ton message"), "Bonjour");
    expect(screen.getByLabelText("Ton message")).toHaveValue("Bonjour");
  });
});
