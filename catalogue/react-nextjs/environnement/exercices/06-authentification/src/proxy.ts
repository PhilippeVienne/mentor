// Le filtre d'URL du site : quelles adresses exigent une session ?
// À corriger (étape 1) : seules les adresses de la zone /admin sont protégées, sauf la page de connexion
// (/admin/login) et les adresses de better-auth (/api/auth).
const PUBLIC: string[] = [];

export function estProtege(pathname: string): boolean {
  return pathname.startsWith("/admin");
}
