package main

import "testing"

func TestDescription(t *testing.T) {
	j := Jeu{Titre: "Azul", Joueurs: 4, Dispo: true}
	if got, want := j.Description(), "Azul (4 joueur·se·s)"; got != want {
		t.Errorf("Description() = %q, attendu %q", got, want)
	}
}

func TestEmprunter(t *testing.T) {
	j := Jeu{Titre: "Azul", Joueurs: 4, Dispo: true}
	j.Emprunter()
	if j.Dispo {
		t.Error("après Emprunter, le jeu d'origine devrait avoir Dispo à false (récepteur pointeur ?)")
	}
}

func TestDisponibles(t *testing.T) {
	catalogue := []Jeu{
		{Titre: "Catane", Dispo: false},
		{Titre: "Dixit", Dispo: true},
		{Titre: "Azul", Dispo: true},
	}
	got := Disponibles(catalogue)
	if len(got) != 2 || got[0].Titre != "Dixit" || got[1].Titre != "Azul" {
		t.Errorf("Disponibles = %v, attendu Dixit puis Azul", got)
	}
	if len(Disponibles(nil)) != 0 {
		t.Error("Disponibles(nil) devrait être vide")
	}
}

func TestIndexParTitre(t *testing.T) {
	index := IndexParTitre([]Jeu{{Titre: "Catane", Joueurs: 4}, {Titre: "Dixit", Joueurs: 6}})
	if len(index) != 2 {
		t.Fatalf("len(index) = %d, attendu 2", len(index))
	}
	if j, ok := index["Dixit"]; !ok || j.Joueurs != 6 {
		t.Errorf(`index["Dixit"] = %v, %v`, j, ok)
	}
	if _, ok := index["Azul"]; ok {
		t.Error(`"Azul" ne devrait pas être dans l'index`)
	}
}

func TestEmprunterTous(t *testing.T) {
	catalogue := []Jeu{{Titre: "Catane", Dispo: true}, {Titre: "Dixit", Dispo: true}}
	EmprunterTous(catalogue)
	for _, j := range catalogue {
		if j.Dispo {
			t.Errorf("%s est encore disponible : le catalogue d'origine n'a pas été modifié", j.Titre)
		}
	}
}
