import { render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ListeGoodies } from "./ListeGoodies";

afterEach(() => {
  vi.unstubAllGlobals();
});

// Une réponse que le test décide de livrer plus tard.
function reponseDifferee(donnees: unknown) {
  let livrer: () => void = () => {};
  const promesse = new Promise((resolve) => {
    livrer = () => resolve({ ok: true, status: 200, json: async () => donnees });
  });
  return { promesse, livrer };
}

describe("ListeGoodies : changement de boutique", () => {
  it("recharge quand la boutique change", async () => {
    const fetchFictif = vi.fn((url: string) =>
      Promise.resolve({
        ok: true,
        status: 200,
        json: async () => (url.includes("/bde/") ? [{ id: 1, nom: "Gourde BDE" }] : [{ id: 2, nom: "Pull BDA" }]),
      }),
    );
    vi.stubGlobal("fetch", fetchFictif);

    const { rerender } = render(<ListeGoodies boutique="bde" />);
    expect(await screen.findByText("Gourde BDE")).toBeInTheDocument();

    rerender(<ListeGoodies boutique="bda" />);
    expect(await screen.findByText("Pull BDA")).toBeInTheDocument();
    expect(screen.queryByText("Gourde BDE")).not.toBeInTheDocument();
    expect(fetchFictif).toHaveBeenCalledTimes(2);
  });

  it("ignore la réponse en retard de l'ancienne boutique", async () => {
    const lente = reponseDifferee([{ id: 1, nom: "Gourde BDE" }]);
    const rapide = reponseDifferee([{ id: 2, nom: "Pull BDA" }]);
    vi.stubGlobal(
      "fetch",
      vi.fn((url: string) => (url.includes("/bde/") ? lente.promesse : rapide.promesse)),
    );

    const { rerender } = render(<ListeGoodies boutique="bde" />);
    rerender(<ListeGoodies boutique="bda" />);
    rapide.livrer();
    expect(await screen.findByText("Pull BDA")).toBeInTheDocument();

    lente.livrer();
    await new Promise((resolve) => setTimeout(resolve, 20));
    expect(screen.getByText("Pull BDA")).toBeInTheDocument();
    expect(screen.queryByText("Gourde BDE")).not.toBeInTheDocument();
  });
});
