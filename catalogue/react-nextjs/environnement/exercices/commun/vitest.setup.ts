import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

// Après chaque test, on retire de la page fictive ce que le test y a affiché.
afterEach(() => {
  cleanup();
});
