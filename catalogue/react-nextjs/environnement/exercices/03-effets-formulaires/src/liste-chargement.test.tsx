import { render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ListeGoodies } from "./ListeGoodies";

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("ListeGoodies : chargement", () => {
  it("affiche « Chargement… » puis la liste reçue du serveur", async () => {
    const fetchFictif = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => [
        { id: 1, nom: "Gourde Éco" },
        { id: 2, nom: "Tote bag" },
      ],
    });
    vi.stubGlobal("fetch", fetchFictif);

    render(<ListeGoodies boutique="bde" />);
    expect(screen.getByText("Chargement…")).toBeInTheDocument();

    expect(await screen.findByText("Gourde Éco")).toBeInTheDocument();
    expect(screen.getByText("Tote bag")).toBeInTheDocument();
    expect(screen.queryByText("Chargement…")).not.toBeInTheDocument();
    expect(fetchFictif).toHaveBeenCalledWith("/api/boutiques/bde/goodies");
  });

  it("n'appelle le serveur qu'une fois pour un même affichage", async () => {
    const fetchFictif = vi.fn().mockResolvedValue({ ok: true, status: 200, json: async () => [] });
    vi.stubGlobal("fetch", fetchFictif);
    render(<ListeGoodies boutique="bde" />);
    await vi.waitFor(() => expect(screen.queryByText("Chargement…")).not.toBeInTheDocument());
    expect(fetchFictif).toHaveBeenCalledTimes(1);
  });
});
