package main

// secret est la clé qui signe et vérifie les jetons (algorithme HS256).
var secret []byte

// Initialise enregistre la clé. Elle refuse une clé vide : pas de secret, pas de démarrage.
// (Jamais de secret par défaut écrit dans le code.)
func Initialise(cle []byte) error {
	panic("à écrire")
}
