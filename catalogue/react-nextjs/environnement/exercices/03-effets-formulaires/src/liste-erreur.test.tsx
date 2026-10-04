import { render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ListeGoodies } from "./ListeGoodies";

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("ListeGoodies : erreurs", () => {
  it("signale une réponse HTTP en erreur", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue({ ok: false, status: 500, json: async () => ({}) }));
    render(<ListeGoodies boutique="bde" />);
    expect(await screen.findByRole("alert")).toHaveTextContent("Erreur HTTP 500");
    expect(screen.queryByText("Chargement…")).not.toBeInTheDocument();
  });

  it("signale une panne réseau", async () => {
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new Error("Réseau coupé")));
    render(<ListeGoodies boutique="bde" />);
    expect(await screen.findByRole("alert")).toHaveTextContent("Réseau coupé");
    expect(screen.queryByRole("list")).not.toBeInTheDocument();
  });
});
