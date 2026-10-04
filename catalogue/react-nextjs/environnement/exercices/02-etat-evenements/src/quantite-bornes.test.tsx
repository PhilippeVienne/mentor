import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ChoixQuantite } from "./ChoixQuantite";

describe("ChoixQuantite : bornes", () => {
  it("désactive − quand la quantité vaut 1", () => {
    render(<ChoixQuantite stock={3} onAjouter={vi.fn()} />);
    expect(screen.getByRole("button", { name: "−" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "+" })).toBeEnabled();
  });

  it("désactive + quand la quantité atteint le stock", async () => {
    render(<ChoixQuantite stock={2} onAjouter={vi.fn()} />);
    await userEvent.click(screen.getByRole("button", { name: "+" }));
    expect(screen.getByText("2")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "+" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "−" })).toBeEnabled();
  });

  it("n'affiche jamais plus que le stock", async () => {
    render(<ChoixQuantite stock={2} onAjouter={vi.fn()} />);
    const plus = screen.getByRole("button", { name: "+" });
    await userEvent.click(plus);
    await userEvent.click(plus);
    await userEvent.click(plus);
    expect(screen.getByText("2")).toBeInTheDocument();
    expect(screen.queryByText("3")).not.toBeInTheDocument();
  });
});
