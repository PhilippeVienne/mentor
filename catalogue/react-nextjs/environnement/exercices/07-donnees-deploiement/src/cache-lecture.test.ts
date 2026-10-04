import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { lireOuCalculer } from "./cache";

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("lireOuCalculer", () => {
  it("ne charge la valeur qu'une fois tant qu'elle n'a pas expiré", async () => {
    const charger = vi.fn().mockResolvedValue("liste");
    expect(await lireOuCalculer("goodies:bde", charger)).toBe("liste");
    expect(await lireOuCalculer("goodies:bde", charger)).toBe("liste");
    expect(charger).toHaveBeenCalledTimes(1);
  });

  it("garde une valeur par clé", async () => {
    const chargerBde = vi.fn().mockResolvedValue("BDE");
    const chargerBda = vi.fn().mockResolvedValue("BDA");
    expect(await lireOuCalculer("goodies:bde:lecture", chargerBde)).toBe("BDE");
    expect(await lireOuCalculer("goodies:bda:lecture", chargerBda)).toBe("BDA");
    expect(await lireOuCalculer("goodies:bde:lecture", chargerBde)).toBe("BDE");
    expect(chargerBde).toHaveBeenCalledTimes(1);
  });

  it("recharge la valeur quand elle a expiré (300 secondes)", async () => {
    const charger = vi.fn().mockResolvedValueOnce("ancienne").mockResolvedValueOnce("nouvelle");
    expect(await lireOuCalculer("goodies:expire", charger)).toBe("ancienne");
    vi.advanceTimersByTime(299_000);
    expect(await lireOuCalculer("goodies:expire", charger)).toBe("ancienne");
    vi.advanceTimersByTime(2_000);
    expect(await lireOuCalculer("goodies:expire", charger)).toBe("nouvelle");
    expect(charger).toHaveBeenCalledTimes(2);
  });
});
