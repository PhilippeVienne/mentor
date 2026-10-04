import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Panneau } from "./Panneau";

describe("Panneau", () => {
  it("affiche son titre dans un titre de niveau 2", () => {
    render(<Panneau titre="Nos goodies">contenu</Panneau>);
    expect(screen.getByRole("heading", { level: 2, name: "Nos goodies" })).toBeInTheDocument();
  });

  it("affiche ses enfants", () => {
    render(
      <Panneau titre="A">
        <p>Salut</p>
      </Panneau>,
    );
    expect(screen.getByText("Salut")).toBeInTheDocument();
  });
});
