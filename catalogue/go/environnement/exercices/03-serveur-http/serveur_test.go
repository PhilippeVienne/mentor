package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"
)

func TestListe(t *testing.T) {
	s := &serveur{jeux: []Jeu{{Titre: "Azul", Joueurs: 4}}}
	rec := httptest.NewRecorder()
	s.liste(rec, httptest.NewRequest("GET", "/jeux", nil))

	if rec.Code != http.StatusOK {
		t.Errorf("statut = %d, attendu 200", rec.Code)
	}
	if ct := rec.Header().Get("Content-Type"); !strings.HasPrefix(ct, "application/json") {
		t.Errorf("Content-Type = %q, attendu application/json", ct)
	}
	var jeux []Jeu
	if err := json.Unmarshal(rec.Body.Bytes(), &jeux); err != nil {
		t.Fatalf("le corps n'est pas du JSON valide : %v (%q)", err, rec.Body.String())
	}
	if len(jeux) != 1 || jeux[0].Titre != "Azul" {
		t.Errorf("jeux = %v, attendu Azul", jeux)
	}
}

func TestAjouteValide(t *testing.T) {
	s := &serveur{}
	rec := httptest.NewRecorder()
	s.ajoute(rec, httptest.NewRequest("POST", "/jeux", strings.NewReader(`{"titre":"Azul","joueurs":4}`)))

	if rec.Code != http.StatusCreated {
		t.Fatalf("statut = %d, attendu 201", rec.Code)
	}
	var j Jeu
	if err := json.Unmarshal(rec.Body.Bytes(), &j); err != nil || j.Titre != "Azul" || j.Joueurs != 4 {
		t.Errorf("la réponse devrait être le jeu créé en JSON, reçu %q", rec.Body.String())
	}
	if len(s.jeux) != 1 {
		t.Errorf("le jeu devrait être ajouté à la liste, len = %d", len(s.jeux))
	}
}

func TestAjouteConcurrent(t *testing.T) {
	s := &serveur{}
	var wg sync.WaitGroup
	for i := 0; i < 100; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			corps := fmt.Sprintf(`{"titre":"Jeu %d","joueurs":2}`, i)
			s.ajoute(httptest.NewRecorder(), httptest.NewRequest("POST", "/jeux", strings.NewReader(corps)))
		}()
	}
	wg.Wait()
	if len(s.jeux) != 100 {
		t.Errorf("len = %d, attendu 100 : des ajouts simultanés ont été perdus (mutex ?)", len(s.jeux))
	}
}

func TestAjouteInvalide(t *testing.T) {
	for nom, corps := range map[string]string{
		"JSON cassé":     `{"titre":`,
		"titre manquant": `{"joueurs":4}`,
	} {
		t.Run(nom, func(t *testing.T) {
			s := &serveur{}
			rec := httptest.NewRecorder()
			s.ajoute(rec, httptest.NewRequest("POST", "/jeux", strings.NewReader(corps)))
			if rec.Code != http.StatusBadRequest {
				t.Errorf("statut = %d, attendu 400", rec.Code)
			}
			if len(s.jeux) != 0 {
				t.Error("rien ne doit être ajouté à la liste")
			}
		})
	}
}

func TestJournal(t *testing.T) {
	var sortie bytes.Buffer
	log.SetOutput(&sortie)
	log.SetFlags(0)
	defer log.SetOutput(io.Discard)

	appele := false
	h := journal(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		appele = true
		w.WriteHeader(http.StatusTeapot)
	}))
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest("GET", "/jeux", nil))

	if !appele || rec.Code != http.StatusTeapot {
		t.Error("le middleware doit appeler le handler suivant (suivant.ServeHTTP)")
	}
	if !strings.Contains(sortie.String(), "GET /jeux") {
		t.Errorf("le journal devrait contenir « GET /jeux », il contient %q", sortie.String())
	}
}

func TestRouteur(t *testing.T) {
	log.SetOutput(&bytes.Buffer{})
	r := NouveauRouteur(&serveur{})
	essai := func(methode, chemin, corps string) int {
		rec := httptest.NewRecorder()
		r.ServeHTTP(rec, httptest.NewRequest(methode, chemin, strings.NewReader(corps)))
		return rec.Code
	}
	if c := essai("POST", "/jeux", `{"titre":"Azul"}`); c != 201 {
		t.Errorf("POST /jeux = %d, attendu 201", c)
	}
	if c := essai("GET", "/jeux", ""); c != 200 {
		t.Errorf("GET /jeux = %d, attendu 200", c)
	}
	if c := essai("DELETE", "/jeux", ""); c != 405 {
		t.Errorf("DELETE /jeux = %d, attendu 405", c)
	}
	if c := essai("GET", "/inconnu", ""); c != 404 {
		t.Errorf("GET /inconnu = %d, attendu 404", c)
	}
}
