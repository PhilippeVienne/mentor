import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("next/headers", () => ({ headers: async () => new Headers() }));
vi.mock("./auth", () => ({ auth: { api: { getSession: vi.fn() } } }));

import { auth } from "./auth";
import { exigerRole } from "./guardian";

const getSession = vi.mocked(auth.api.getSession);

beforeEach(() => {
  getSession.mockReset();
});

describe("exigerRole", () => {
  it("redirige vers la connexion quand il n'y a pas de session", async () => {
    getSession.mockResolvedValue(null);
    await expect(exigerRole(["admin"])).rejects.toMatchObject({ digest: expect.stringContaining("/admin/login") });
  });

  it("redirige vers « accès refusé » quand le rôle ne convient pas", async () => {
    getSession.mockResolvedValue({ user: { id: "u1", role: "user" } });
    await expect(exigerRole(["admin"])).rejects.toMatchObject({ digest: expect.stringContaining("/acces-refuse") });
  });

  it("renvoie la session quand le rôle convient", async () => {
    const session = { user: { id: "u2", role: "caissier" as const } };
    getSession.mockResolvedValue(session);
    await expect(exigerRole(["admin", "caissier"])).resolves.toEqual(session);
  });
});
