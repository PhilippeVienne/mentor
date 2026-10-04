import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { POST } from "../app/api/cron_jobs/route";

const requete = (autorisation?: string) =>
  new Request("http://localhost/api/cron_jobs", {
    method: "POST",
    headers: autorisation ? { authorization: autorisation } : {},
  });

beforeEach(() => {
  process.env.CRON_SECRET = "secret-de-test";
});

afterEach(() => {
  delete process.env.CRON_SECRET;
});

describe("route /api/cron_jobs", () => {
  it("refuse une requête sans en-tête Authorization", async () => {
    const reponse = await POST(requete());
    expect(reponse.status).toBe(401);
  });

  it("refuse un mauvais secret", async () => {
    const reponse = await POST(requete("Bearer autre-chose"));
    expect(reponse.status).toBe(401);
    expect(await reponse.json()).toEqual({ error: "Unauthorized" });
  });

  it("accepte le bon secret", async () => {
    const reponse = await POST(requete("Bearer secret-de-test"));
    expect(reponse.status).toBe(200);
    expect(await reponse.json()).toEqual({ message: "ok" });
  });

  it("refuse tout si le secret n'est pas défini", async () => {
    delete process.env.CRON_SECRET;
    const reponse = await POST(requete("Bearer undefined"));
    expect(reponse.status).toBe(401);
  });
});
