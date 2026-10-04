import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { classesDe } from "./classes";
import { GrilleCategories } from "./GrilleCategories";

describe("GrilleCategories", () => {
  it("est une grille adaptative : 1 colonne, 2 dès sm, 3 dès lg", () => {
    render(
      <GrilleCategories>
        <p>Vêtements</p>
      </GrilleCategories>,
    );
    const grille = screen.getByText("Vêtements").parentElement as HTMLElement;
    expect(classesDe(grille)).toEqual(
      expect.arrayContaining(["grid", "gap-4", "sm:grid-cols-2", "lg:grid-cols-3"]),
    );
  });
});
