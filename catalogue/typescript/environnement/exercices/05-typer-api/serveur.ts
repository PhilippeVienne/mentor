// Faux serveur : pas de réseau dans l'environnement, on simule les réponses JSON.
export async function reponseCorrecte(): Promise<unknown> {
  return JSON.parse('[{"id":1,"titre":"Gala","places":200,"lieu":null}]');
}

export async function reponseCassee(): Promise<unknown> {
  // Le serveur a changé : le champ « places » a disparu.
  return JSON.parse('[{"id":1,"titre":"Gala","lieu":null}]');
}
