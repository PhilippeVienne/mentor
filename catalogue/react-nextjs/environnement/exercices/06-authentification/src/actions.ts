"use server";

import { exigerRole } from "./guardian";
import { ecrireStock } from "./stock-db";

// Une action serveur : n'importe qui peut l'appeler, même sans passer par ton interface.
// À corriger (étape 3) : seul·e un·e admin doit pouvoir modifier le stock.
export async function modifierStock(id: number, stock: number) {
  await ecrireStock(id, stock);
}
