import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ChoixQuantite } from "./ChoixQuantite";

describe("ChoixQuantite : clics", () => {
  it("commence à 1", () => {
    render(<ChoixQuantite stock={5} onAjouter={vi.fn()} />);
    expect(screen.getByText("1")).toBeInTheDocument();
  });

  it("monte d'une unité à chaque clic sur +", async () => {
    render(<ChoixQuantite stock={5} onAjouter={vi.fn()} />);
    await userEvent.click(screen.getByRole("button", { name: "+" }));
    expect(screen.getByText("2")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "+" }));
    expect(screen.getByText("3")).toBeInTheDocument();
  });

  it("descend d'une unité à chaque clic sur −", async () => {
    render(<ChoixQuantite stock={5} onAjouter={vi.fn()} />);
    await userEvent.click(screen.getByRole("button", { name: "+" }));
    await userEvent.click(screen.getByRole("button", { name: "+" }));
    await userEvent.click(screen.getByRole("button", { name: "−" }));
    expect(screen.getByText("2")).toBeInTheDocument();
  });
});
