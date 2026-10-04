package main

import "net/http"

// ajoute répond à POST /jeux : lit un Jeu en JSON dans le corps de la requête, l'ajoute à la liste
// et répond 201 avec le jeu en JSON. JSON invalide ou titre vide : réponse 400, et rien n'est ajouté.
func (s *serveur) ajoute(w http.ResponseWriter, r *http.Request) {
	panic("à écrire")
}
