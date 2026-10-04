import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Bouton } from "./Bouton";
import { classesDe } from "./classes";

describe("Bouton", () => {
  it("a une couleur, un survol et un état désactivé", () => {
    render(<Bouton>Ajouter</Bouton>);
    expect(classesDe(screen.getByRole("button"))).toEqual(
      expect.arrayContaining([
        "rounded-md",
        "px-4",
        "py-2",
        "text-white",
        "bg-sky-600",
        "hover:bg-sky-700",
        "disabled:opacity-50",
      ]),
    );
  });

  it("reste un vrai bouton désactivable", () => {
    render(<Bouton disabled>Ajouter</Bouton>);
    expect(screen.getByRole("button")).toBeDisabled();
  });
});
