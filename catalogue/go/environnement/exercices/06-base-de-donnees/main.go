package main

import (
	"context"
	"fmt"
	"time"
)

func main() {
	var depot DepotJeux = nouveauDepotMemoire()

	ctx, annule := context.WithTimeout(context.Background(), 2*time.Second)
	defer annule()

	depot.Creer(ctx, "Azul")
	depot.Creer(ctx, "Catane")
	jeux, _ := depot.Lister(ctx)
	for _, j := range jeux {
		fmt.Println(j.ID, j.Titre)
	}
	if _, err := depot.Trouver(ctx, 42); err != nil {
		fmt.Println("erreur attendue :", err)
	}
}
