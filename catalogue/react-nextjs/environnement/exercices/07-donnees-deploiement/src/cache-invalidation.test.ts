import { describe, expect, it, vi } from "vitest";
import { invalider, lireOuCalculer } from "./cache";

describe("invalider", () => {
  it("force un nouveau chargement à la lecture suivante", async () => {
    const charger = vi.fn().mockResolvedValueOnce("v1").mockResolvedValueOnce("v2");
    expect(await lireOuCalculer("goodies:invalide", charger)).toBe("v1");
    expect(await lireOuCalculer("goodies:invalide", charger)).toBe("v1");
    expect(charger).toHaveBeenCalledTimes(1);

    invalider("goodies:invalide");
    expect(await lireOuCalculer("goodies:invalide", charger)).toBe("v2");
    expect(charger).toHaveBeenCalledTimes(2);
  });

  it("ne touche pas aux autres clés", async () => {
    const chargerA = vi.fn().mockResolvedValue("A");
    const chargerB = vi.fn().mockResolvedValue("B");
    await lireOuCalculer("cle:a", chargerA);
    await lireOuCalculer("cle:b", chargerB);
    invalider("cle:a");
    await lireOuCalculer("cle:a", chargerA);
    await lireOuCalculer("cle:b", chargerB);
    expect(chargerA).toHaveBeenCalledTimes(2);
    expect(chargerB).toHaveBeenCalledTimes(1);
  });
});
