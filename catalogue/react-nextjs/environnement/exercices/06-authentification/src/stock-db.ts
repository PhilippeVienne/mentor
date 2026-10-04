// Une fausse base de données : `ecrireStock` modifie le stock d'un produit.
export async function ecrireStock(id: number, stock: number): Promise<void> {
  console.log(`stock du produit ${id} : ${stock}`);
}
