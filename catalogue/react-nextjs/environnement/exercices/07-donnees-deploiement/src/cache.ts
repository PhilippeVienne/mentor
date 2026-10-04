// Un cache en mémoire, selon le motif « cache-aside ». Pour l'instant il ne cache rien.
// À écrire (étapes 4 et 5), en gardant une entrée par clé avec sa date d'expiration.
const TTL_SECONDES = 300;

export async function lireOuCalculer<T>(cle: string, charger: () => Promise<T>): Promise<T> {
  return charger();
}

export function invalider(cle: string): void {}
