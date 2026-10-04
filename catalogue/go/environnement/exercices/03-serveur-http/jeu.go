package main

import "sync"

// Jeu est la représentation JSON d'un jeu. Les étiquettes `json:"…"` fixent les noms des champs.
type Jeu struct {
	Titre   string `json:"titre"`
	Joueurs int    `json:"joueurs"`
	Note    string `json:"note,omitempty"`
}

// serveur garde la liste des jeux en mémoire ; le mutex protège la liste quand plusieurs requêtes arrivent ensemble.
type serveur struct {
	mu   sync.Mutex
	jeux []Jeu
}
