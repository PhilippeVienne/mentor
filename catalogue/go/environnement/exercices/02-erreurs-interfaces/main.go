package main

import (
	"errors"
	"fmt"

	"ludotheque/pret"
)

func main() {
	var p pret.Preteur = pret.NouveauStock("Catane", "Dixit", "Azul")

	fmt.Println(p.Preter("Catane", "Camille"))
	err := p.Preter("Catane", "Sam")
	fmt.Println(err)
	fmt.Println(errors.Is(err, pret.ErrIndisponible))

	_ = p.Preter("Dixit", "Camille")
	err = p.Preter("Azul", "Camille")

	var quota *pret.ErreurQuota
	if errors.As(err, &quota) {
		fmt.Println("quota atteint pour", quota.Membre)
	}
}
