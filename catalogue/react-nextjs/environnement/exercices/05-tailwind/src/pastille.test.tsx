import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { classesDe } from "./classes";
import { Pastille } from "./Pastille";

describe("Pastille", () => {
  it("garde sa forme ronde", () => {
    const { container } = render(<Pastille couleur="rouge" />);
    expect(classesDe(container.firstElementChild as Element)).toEqual(
      expect.arrayContaining(["inline-block", "size-4", "rounded-full"]),
    );
  });

  it("donne une couleur de fond différente à chaque couleur", () => {
    const rouge = render(<Pastille couleur="rouge" />).container.firstElementChild as Element;
    const bleu = render(<Pastille couleur="bleu" />).container.firstElementChild as Element;
    expect(classesDe(rouge).some((c) => /^bg-/.test(c))).toBe(true);
    expect(classesDe(rouge).find((c) => /^bg-/.test(c))).not.toEqual(classesDe(bleu).find((c) => /^bg-/.test(c)));
  });

  it("utilise les classes bg-red-500 et bg-blue-500", () => {
    const rouge = render(<Pastille couleur="rouge" />).container.firstElementChild as Element;
    const bleu = render(<Pastille couleur="bleu" />).container.firstElementChild as Element;
    expect(classesDe(rouge)).toContain("bg-red-500");
    expect(classesDe(bleu)).toContain("bg-blue-500");
  });
});
