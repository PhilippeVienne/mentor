import { describe, expect, it } from "vitest";
import { entetesAuthorization } from "./keycloak";

describe("entetesAuthorization", () => {
  it("ajoute le jeton dans un en-tête Bearer", () => {
    expect(entetesAuthorization("abc.def.ghi")).toEqual({ Authorization: "Bearer abc.def.ghi" });
  });

  it("n'envoie aucun en-tête sans jeton", () => {
    expect(entetesAuthorization(null)).toEqual({});
  });
});
