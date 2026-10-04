package main

import (
	"context"
	"errors"
)

// Jeu est un jeu enregistré, avec un numéro (ID).
type Jeu struct {
	ID    int
	Titre string
}

// ErrIntrouvable est renvoyée quand aucun jeu n'a le numéro demandé.
var ErrIntrouvable = errors.New("jeu introuvable")

// DepotJeux cache la base de données derrière une interface.
// Le contexte permet d'annuler la requête (client parti, délai dépassé).
type DepotJeux interface {
	Creer(ctx context.Context, titre string) (*Jeu, error)
	Trouver(ctx context.Context, id int) (*Jeu, error)
	Lister(ctx context.Context) ([]*Jeu, error)
}

// depotMemoire est un dépôt qui garde tout en mémoire, pour les tests et les démonstrations.
type depotMemoire struct {
	jeux    map[int]*Jeu
	suivant int // numéro du prochain jeu créé
}

func nouveauDepotMemoire() *depotMemoire {
	return &depotMemoire{jeux: map[int]*Jeu{}, suivant: 1}
}
