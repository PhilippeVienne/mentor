package main

import "context"

// Lister renvoie tous les jeux, triés par numéro croissant (utilise sort.Slice ou slices.SortFunc).
func (d *depotMemoire) Lister(ctx context.Context) ([]*Jeu, error) {
	panic("à écrire")
}
