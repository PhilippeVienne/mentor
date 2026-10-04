import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("next/headers", () => ({ headers: async () => new Headers() }));
vi.mock("./auth", () => ({ auth: { api: { getSession: vi.fn() } } }));
vi.mock("./stock-db", () => ({ ecrireStock: vi.fn() }));

import { modifierStock } from "./actions";
import { auth } from "./auth";
import { ecrireStock } from "./stock-db";

const getSession = vi.mocked(auth.api.getSession);

beforeEach(() => {
  getSession.mockReset();
  vi.mocked(ecrireStock).mockReset();
});

describe("modifierStock", () => {
  it("ne touche pas à la base sans session", async () => {
    getSession.mockResolvedValue(null);
    await expect(modifierStock(1, 10)).rejects.toBeDefined();
    expect(ecrireStock).not.toHaveBeenCalled();
  });

  it("ne touche pas à la base pour un simple utilisateur", async () => {
    getSession.mockResolvedValue({ user: { id: "u1", role: "user" } });
    await expect(modifierStock(1, 10)).rejects.toBeDefined();
    expect(ecrireStock).not.toHaveBeenCalled();
  });

  it("écrit le stock pour un·e admin", async () => {
    getSession.mockResolvedValue({ user: { id: "u2", role: "admin" } });
    await modifierStock(1, 10);
    expect(ecrireStock).toHaveBeenCalledWith(1, 10);
  });
});
