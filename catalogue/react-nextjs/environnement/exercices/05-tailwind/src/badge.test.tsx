import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Badge } from "./Badge";

describe("Badge", () => {
  it("applique la couleur reçue avec l'attribut style", () => {
    render(<Badge texte="Nouveau" couleur="#104e64" />);
    expect(screen.getByText("Nouveau")).toHaveStyle({ backgroundColor: "#104e64" });
  });

  it("utilise une couleur par défaut quand aucune n'est fournie", () => {
    render(<Badge texte="Nouveau" />);
    expect(screen.getByText("Nouveau")).toHaveStyle({ backgroundColor: "#104e64" });
  });
});
