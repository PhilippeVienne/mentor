package pret

import (
	"errors"
	"strings"
	"testing"
)

func TestNouveauStock(t *testing.T) {
	s := NouveauStock("Catane", "Dixit")
	if !s.Disponible("Catane") || !s.Disponible("Dixit") {
		t.Error("les titres donnés à NouveauStock devraient être disponibles")
	}
	if s.Disponible("Azul") {
		t.Error("Azul n'a pas été donné : il ne doit pas être disponible")
	}
	if NouveauStock().Disponible("Catane") {
		t.Error("un stock vide ne contient rien")
	}
}

func TestIndisponible(t *testing.T) {
	s := NouveauStock("Catane")
	err := s.Preter("Azul", "Camille")
	if !errors.Is(err, ErrIndisponible) {
		t.Fatalf("erreur = %v, attendu une erreur qui enveloppe ErrIndisponible (%%w)", err)
	}
	if !strings.Contains(err.Error(), `"Azul"`) {
		t.Errorf("le message %q devrait citer le titre entre guillemets (%%q)", err.Error())
	}
}

func TestPretReussi(t *testing.T) {
	s := NouveauStock("Catane")
	if err := s.Preter("Catane", "Camille"); err != nil {
		t.Fatalf("Preter a échoué : %v", err)
	}
	if s.Disponible("Catane") {
		t.Error("après le prêt, Catane ne doit plus être disponible")
	}
	if err := s.Preter("Catane", "Sam"); !errors.Is(err, ErrIndisponible) {
		t.Errorf("deuxième prêt : erreur = %v, attendu ErrIndisponible", err)
	}
}

func TestQuota(t *testing.T) {
	s := NouveauStock("Catane", "Dixit", "Azul")
	_ = s.Preter("Catane", "Camille")
	_ = s.Preter("Dixit", "Camille")
	err := s.Preter("Azul", "Camille")

	var quota *ErreurQuota
	if !errors.As(err, &quota) {
		t.Fatalf("erreur = %v, attendu un *ErreurQuota", err)
	}
	if quota.Membre != "Camille" || quota.Max != 2 {
		t.Errorf("quota = %+v, attendu Camille et 2", *quota)
	}
	if !s.Disponible("Azul") {
		t.Error("le prêt refusé ne doit pas retirer Azul du stock")
	}
}
