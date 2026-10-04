import { afterEach, describe, expect, it } from "vitest";
import { lireSecret } from "./config";

afterEach(() => {
  delete process.env.BETTER_AUTH_SECRET;
});

describe("lireSecret", () => {
  it("renvoie la variable d'environnement", () => {
    process.env.BETTER_AUTH_SECRET = "secret-de-test";
    expect(lireSecret()).toBe("secret-de-test");
  });

  it("lance une erreur si la variable est absente", () => {
    expect(() => lireSecret()).toThrow("BETTER_AUTH_SECRET manquant");
  });

  it("lance une erreur si la variable est vide", () => {
    process.env.BETTER_AUTH_SECRET = "";
    expect(() => lireSecret()).toThrow("BETTER_AUTH_SECRET manquant");
  });
});
