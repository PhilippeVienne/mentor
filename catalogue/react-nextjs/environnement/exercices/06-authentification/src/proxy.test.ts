import { describe, expect, it } from "vitest";
import { estProtege } from "./proxy";

describe("estProtege", () => {
  it.each(["/admin", "/admin/stock", "/admin/bde/produits"])("protège %s", (chemin) => {
    expect(estProtege(chemin)).toBe(true);
  });

  it.each(["/", "/boutique", "/bde/produit/12", "/administration"])("laisse %s public", (chemin) => {
    expect(estProtege(chemin)).toBe(false);
  });

  it.each(["/admin/login", "/api/auth", "/api/auth/sign-in/email"])("laisse %s public (connexion)", (chemin) => {
    expect(estProtege(chemin)).toBe(false);
  });
});
