// Package pret gère les emprunts de la ludothèque.
package pret

import (
	"errors"
	"fmt"
)

// ErrIndisponible est une erreur « sentinelle » : on la compare avec errors.Is.
var ErrIndisponible = errors.New("jeu indisponible")

// ErreurQuota est une erreur typée : on la récupère avec errors.As.
type ErreurQuota struct {
	Membre string
	Max    int
}

func (e *ErreurQuota) Error() string {
	return fmt.Sprintf("%s a déjà %d jeux empruntés", e.Membre, e.Max)
}

// Stock est une implémentation en mémoire : quels jeux sont disponibles, et combien chaque membre en a emprunté.
type Stock struct {
	dispo    map[string]bool
	emprunts map[string]int
}

// Disponible dit si le jeu est dans le stock et non emprunté.
func (s *Stock) Disponible(titre string) bool {
	return s.dispo[titre]
}
