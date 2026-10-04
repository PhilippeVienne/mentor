import { render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { GoodieList } from "./GoodieList";
import { goodies } from "./goodies";

afterEach(() => {
  vi.restoreAllMocks();
});

describe("GoodieList", () => {
  it("affiche une carte par goodie", () => {
    render(<GoodieList goodies={goodies} />);
    expect(screen.getAllByRole("article")).toHaveLength(2);
    expect(screen.getByRole("heading", { name: "Tote bag" })).toBeInTheDocument();
  });

  it("affiche un message quand la liste est vide", () => {
    render(<GoodieList goodies={[]} />);
    expect(screen.getByText("Aucun goodie pour le moment.")).toBeInTheDocument();
    expect(screen.queryAllByRole("article")).toHaveLength(0);
  });

  it("donne une key à chaque carte (React ne se plaint pas)", () => {
    const erreurs = vi.spyOn(console, "error").mockImplementation(() => {});
    render(<GoodieList goodies={goodies} />);
    const plaintesDeKey = erreurs.mock.calls.filter((appel) => String(appel[0]).includes("key"));
    expect(plaintesDeKey).toHaveLength(0);
    expect(screen.getAllByRole("article")).toHaveLength(2);
  });
});
