package main

import "fmt"

func main() {
	catalogue := []Jeu{
		{Titre: "Catane", Joueurs: 4, Dispo: true},
		{Titre: "Dixit", Joueurs: 6, Dispo: true},
	}

	catalogue[0].Emprunter()

	index := IndexParTitre(catalogue)
	fmt.Println(index["Dixit"].Description())
	fmt.Println(len(Disponibles(catalogue)), "jeu disponible sur", len(catalogue))

	EmprunterTous(catalogue)
	fmt.Println(len(Disponibles(catalogue)), "jeu disponible sur", len(catalogue))
}
