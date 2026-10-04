package main

import "context"

// Trouver renvoie le jeu qui a ce numéro. S'il n'existe pas, renvoie une erreur qui enveloppe ErrIntrouvable
// et cite le numéro, par exemple « jeu 42 : jeu introuvable ».
// Si le contexte est déjà annulé (ctx.Err() != nil), renvoie cette erreur.
func (d *depotMemoire) Trouver(ctx context.Context, id int) (*Jeu, error) {
	panic("à écrire")
}
