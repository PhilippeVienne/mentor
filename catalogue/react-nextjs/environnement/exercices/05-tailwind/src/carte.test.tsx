import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Carte } from "./Carte";
import { classesDe } from "./classes";

describe("Carte", () => {
  it("met en forme la carte, son titre et sa description", () => {
    render(<Carte nom="Gourde Éco" description="Inox, 50 cl." />);
    expect(classesDe(screen.getByRole("article"))).toEqual(
      expect.arrayContaining(["rounded-lg", "bg-white", "p-4", "shadow-sm"]),
    );
    expect(classesDe(screen.getByRole("heading"))).toEqual(
      expect.arrayContaining(["text-lg", "font-semibold", "text-slate-900"]),
    );
    expect(classesDe(screen.getByText("Inox, 50 cl."))).toEqual(
      expect.arrayContaining(["mt-2", "text-sm", "text-slate-600"]),
    );
  });
});
