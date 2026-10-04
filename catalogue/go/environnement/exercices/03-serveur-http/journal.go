package main

import "net/http"

// journal est un middleware : il enveloppe un handler. Pour chaque requête, il écrit une ligne
// « MÉTHODE CHEMIN » avec log.Println(r.Method, r.URL.Path), puis appelle le handler suivant.
func journal(suivant http.Handler) http.Handler {
	panic("à écrire")
}
