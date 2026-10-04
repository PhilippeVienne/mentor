package main

import "net/http"

// NouveauRouteur associe les motifs "GET /jeux" et "POST /jeux" aux handlers de s,
// puis enveloppe le tout avec le middleware journal.
func NouveauRouteur(s *serveur) http.Handler {
	panic("à écrire")
}
