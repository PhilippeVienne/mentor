package main

import "context"

// Creer enregistre un jeu avec le prochain numéro (1, puis 2, puis 3…) et le renvoie.
// Si le contexte est déjà annulé (ctx.Err() != nil), renvoie cette erreur sans rien créer.
func (d *depotMemoire) Creer(ctx context.Context, titre string) (*Jeu, error) {
	panic("à écrire")
}
