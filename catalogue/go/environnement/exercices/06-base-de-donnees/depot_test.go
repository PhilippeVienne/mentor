package main

import (
	"context"
	"errors"
	"strings"
	"testing"
)

func TestCreer(t *testing.T) {
	d := nouveauDepotMemoire()
	a, err := d.Creer(context.Background(), "Azul")
	if err != nil || a.ID != 1 || a.Titre != "Azul" {
		t.Fatalf("premier jeu = %+v, %v ; attendu ID 1 et titre Azul", a, err)
	}
	b, _ := d.Creer(context.Background(), "Catane")
	if b.ID != 2 {
		t.Errorf("deuxième jeu : ID = %d, attendu 2", b.ID)
	}
}

func TestTrouver(t *testing.T) {
	d := nouveauDepotMemoire()
	d.Creer(context.Background(), "Azul")

	j, err := d.Trouver(context.Background(), 1)
	if err != nil || j.Titre != "Azul" {
		t.Fatalf("Trouver(1) = %+v, %v", j, err)
	}
	_, err = d.Trouver(context.Background(), 42)
	if !errors.Is(err, ErrIntrouvable) {
		t.Fatalf("erreur = %v, attendu une erreur qui enveloppe ErrIntrouvable", err)
	}
	if !strings.Contains(err.Error(), "42") {
		t.Errorf("le message %q devrait citer le numéro 42", err.Error())
	}
}

func TestContexte(t *testing.T) {
	d := nouveauDepotMemoire()
	ctx, annule := context.WithCancel(context.Background())
	annule()

	if _, err := d.Creer(ctx, "Azul"); !errors.Is(err, context.Canceled) {
		t.Errorf("Creer avec un contexte annulé : erreur = %v, attendu context.Canceled", err)
	}
	if len(d.jeux) != 0 {
		t.Error("Creer ne doit rien enregistrer quand le contexte est annulé")
	}
	if _, err := d.Trouver(ctx, 1); !errors.Is(err, context.Canceled) {
		t.Errorf("Trouver avec un contexte annulé : erreur = %v, attendu context.Canceled", err)
	}
}

func TestLister(t *testing.T) {
	d := nouveauDepotMemoire()
	for _, titre := range []string{"Azul", "Catane", "Dixit", "Pandemic", "Splendor"} {
		d.Creer(context.Background(), titre)
	}
	jeux, err := d.Lister(context.Background())
	if err != nil || len(jeux) != 5 {
		t.Fatalf("Lister = %d jeux, %v ; attendu 5", len(jeux), err)
	}
	for i, j := range jeux {
		if j.ID != i+1 {
			t.Fatalf("le jeu %d a l'ID %d : la liste doit être triée par numéro", i, j.ID)
		}
	}
}

func TestConfigDepuisEnv(t *testing.T) {
	t.Setenv("DB_HOST", "")
	t.Setenv("DB_USER", "")
	t.Setenv("DB_PASSWORD", "")
	if _, err := ConfigDepuisEnv(); err == nil {
		t.Error("sans DB_USER ni DB_PASSWORD, une erreur est attendue")
	}

	t.Setenv("DB_USER", "ludo")
	if _, err := ConfigDepuisEnv(); err == nil {
		t.Error("sans DB_PASSWORD, une erreur est attendue")
	}

	t.Setenv("DB_PASSWORD", "mot-de-passe-factice")
	adresse, err := ConfigDepuisEnv()
	if err != nil {
		t.Fatalf("avec les variables obligatoires, pas d'erreur attendue : %v", err)
	}
	if adresse != "postgres://ludo:mot-de-passe-factice@localhost/ludotheque" {
		t.Errorf("adresse = %q (DB_HOST absent : « localhost » attendu)", adresse)
	}

	t.Setenv("DB_HOST", "db.example.org")
	if adresse, _ := ConfigDepuisEnv(); !strings.Contains(adresse, "@db.example.org/") {
		t.Errorf("adresse = %q, devrait utiliser DB_HOST", adresse)
	}
}
