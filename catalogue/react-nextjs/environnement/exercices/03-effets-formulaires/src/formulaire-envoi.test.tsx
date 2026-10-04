import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { FormulaireContact } from "./FormulaireContact";

async function remplir() {
  await userEvent.type(screen.getByLabelText("Ton e-mail"), "claire@example.org");
  await userEvent.type(screen.getByLabelText("Ton message"), "Bonjour !");
}

describe("FormulaireContact : envoi", () => {
  it("envoie l'e-mail et le message, puis vide le message", async () => {
    const onEnvoyer = vi.fn().mockResolvedValue(undefined);
    render(<FormulaireContact onEnvoyer={onEnvoyer} />);
    await remplir();
    await userEvent.click(screen.getByRole("button", { name: "Envoyer" }));
    expect(onEnvoyer).toHaveBeenCalledTimes(1);
    expect(onEnvoyer).toHaveBeenCalledWith({ email: "claire@example.org", texte: "Bonjour !" });
    await vi.waitFor(() => expect(screen.getByLabelText("Ton message")).toHaveValue(""));
  });

  it("désactive le bouton pendant l'envoi", async () => {
    let terminer: () => void = () => {};
    const onEnvoyer = vi.fn(() => new Promise<void>((resolve) => (terminer = resolve)));
    render(<FormulaireContact onEnvoyer={onEnvoyer} />);
    await remplir();
    await userEvent.click(screen.getByRole("button", { name: "Envoyer" }));
    expect(screen.getByRole("button", { name: "Envoi…" })).toBeDisabled();
    terminer();
    expect(await screen.findByRole("button", { name: "Envoyer" })).toBeEnabled();
  });

  it("affiche une erreur si l'envoi échoue, et garde le message", async () => {
    const onEnvoyer = vi.fn().mockRejectedValue(new Error("panne"));
    render(<FormulaireContact onEnvoyer={onEnvoyer} />);
    await remplir();
    await userEvent.click(screen.getByRole("button", { name: "Envoyer" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Envoi impossible, réessaie dans un instant.");
    expect(screen.getByLabelText("Ton message")).toHaveValue("Bonjour !");
  });

  it("n'envoie rien si le message est vide", async () => {
    const onEnvoyer = vi.fn().mockResolvedValue(undefined);
    render(<FormulaireContact onEnvoyer={onEnvoyer} />);
    await userEvent.type(screen.getByLabelText("Ton e-mail"), "claire@example.org");
    await userEvent.click(screen.getByRole("button", { name: "Envoyer" }));
    expect(onEnvoyer).not.toHaveBeenCalled();
  });
});
