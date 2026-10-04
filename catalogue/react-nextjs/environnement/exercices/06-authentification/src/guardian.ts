import { headers } from "next/headers";
import { redirect } from "next/navigation";
import { auth, type Role } from "./auth";

// À écrire (étape 2) : retrouver la session de la requête avec auth.api.getSession({ headers: await headers() }).
//  - pas de session : redirect("/admin/login") ;
//  - rôle de la personne absent de `roles` : redirect("/acces-refuse") ;
//  - sinon : renvoyer la session.
export async function exigerRole(roles: Role[]) {
  return null;
}
