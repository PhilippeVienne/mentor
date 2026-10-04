package main

import (
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

// Clé factice, réservée aux tests.
var cleDeTest = []byte("cle-factice-pour-les-tests-uniquement")

func prepare(t *testing.T) {
	t.Helper()
	secret = nil
	if err := Initialise(cleDeTest); err != nil {
		t.Fatalf("Initialise avec une clé non vide a échoué : %v", err)
	}
}

func TestInitialise(t *testing.T) {
	secret = nil
	if Initialise(nil) == nil || Initialise([]byte{}) == nil {
		t.Error("une clé vide doit être refusée")
	}
	if err := Initialise([]byte("abc")); err != nil {
		t.Errorf("une clé non vide doit être acceptée : %v", err)
	}
	if string(secret) != "abc" {
		t.Errorf("la clé acceptée doit être enregistrée dans secret, secret = %q", secret)
	}
}

func TestARole(t *testing.T) {
	c := Claims{Roles: []string{"membre", "bureau"}}
	if !c.ARole("bureau") || !c.ARole("membre") {
		t.Error("les rôles de la liste devraient être reconnus")
	}
	if c.ARole("admin") || (Claims{}).ARole("bureau") {
		t.Error("un rôle absent ne doit pas être reconnu")
	}
}

func TestVerifieValide(t *testing.T) {
	prepare(t)
	jeton, err := Signe("Camille", []string{"bureau"}, time.Hour)
	if err != nil {
		t.Fatal(err)
	}
	c, err := Verifie(jeton)
	if err != nil {
		t.Fatalf("un jeton valide devrait être accepté : %v", err)
	}
	if c.Nom != "Camille" || !c.ARole("bureau") {
		t.Errorf("contenu = %+v", c)
	}

	if _, err := Verifie("pas.un.jeton"); err == nil {
		t.Error("un texte quelconque doit être refusé")
	}
	autre, _ := jwt.NewWithClaims(jwt.SigningMethodHS256, Claims{Nom: "Pirate",
		RegisteredClaims: jwt.RegisteredClaims{ExpiresAt: jwt.NewNumericDate(time.Now().Add(time.Hour))},
	}).SignedString([]byte("une-autre-cle"))
	if _, err := Verifie(autre); err == nil {
		t.Error("un jeton signé avec une autre clé doit être refusé")
	}
}

func TestVerifieExpire(t *testing.T) {
	prepare(t)
	jeton, _ := Signe("Camille", nil, -time.Minute)
	if _, err := Verifie(jeton); err == nil {
		t.Error("un jeton expiré doit être refusé")
	}
}

func TestVerifieSansExpiration(t *testing.T) {
	prepare(t)
	jeton, _ := jwt.NewWithClaims(jwt.SigningMethodHS256, Claims{Nom: "Camille"}).SignedString(cleDeTest)
	if _, err := Verifie(jeton); err == nil {
		t.Error("un jeton sans date d'expiration doit être refusé (jwt.WithExpirationRequired)")
	}
}

func TestVerifieAlgorithme(t *testing.T) {
	prepare(t)
	// Même clé, mais algorithme HS512 : le serveur n'attend que HS256.
	jeton, _ := jwt.NewWithClaims(jwt.SigningMethodHS512, Claims{Nom: "Camille",
		RegisteredClaims: jwt.RegisteredClaims{ExpiresAt: jwt.NewNumericDate(time.Now().Add(time.Hour))},
	}).SignedString(cleDeTest)
	if _, err := Verifie(jeton); err == nil {
		t.Error("un jeton signé en HS512 doit être refusé (jwt.WithValidMethods)")
	}
}
