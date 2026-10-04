package main

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/gorilla/websocket"
)

func adresse(srv *httptest.Server, nom string) string {
	return "ws" + strings.TrimPrefix(srv.URL, "http") + "/ws?nom=" + nom
}

func TestOrigine(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(NouvelleSalle().Ws))
	defer srv.Close()

	bon := http.Header{"Origin": {OrigineAttendue}}
	c, _, err := websocket.DefaultDialer.Dial(adresse(srv, "Camille"), bon)
	if err != nil {
		t.Fatalf("l'origine attendue devrait être acceptée : %v", err)
	}
	c.Close()

	mauvais := http.Header{"Origin": {"https://pirate.example.net"}}
	_, resp, err := websocket.DefaultDialer.Dial(adresse(srv, "Camille"), mauvais)
	if err == nil {
		t.Fatal("une autre origine devrait être refusée")
	}
	if resp == nil || resp.StatusCode != http.StatusForbidden {
		t.Errorf("réponse = %v, attendu 403", resp)
	}
}

func TestRegistre(t *testing.T) {
	s := NouvelleSalle()
	c := &websocket.Conn{}
	s.Ajoute(c, "Camille")
	if s.Taille() != 1 {
		t.Fatalf("Taille = %d, attendu 1", s.Taille())
	}
	s.Retire(c)
	if s.Taille() != 0 {
		t.Fatalf("Taille = %d, attendu 0 après Retire", s.Taille())
	}

	// Beaucoup de goroutines en même temps : sans mutex, Go s'arrête avec « concurrent map writes ».
	var wg sync.WaitGroup
	for i := 0; i < 200; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			c := &websocket.Conn{}
			for k := 0; k < 100; k++ {
				s.Ajoute(c, "x")
				s.Retire(c)
			}
		}()
	}
	wg.Wait()
	if s.Taille() != 0 {
		t.Errorf("Taille = %d, attendu 0", s.Taille())
	}
}

func attendre(t *testing.T, s *Salle, n int) {
	t.Helper()
	fin := time.Now().Add(3 * time.Second)
	for s.Taille() != n {
		if time.Now().After(fin) {
			t.Fatalf("Taille = %d, attendu %d (Ajoute / Retire sont-elles écrites ?)", s.Taille(), n)
		}
		time.Sleep(10 * time.Millisecond)
	}
}

func TestDiffuse(t *testing.T) {
	salle := NouvelleSalle()
	srv := httptest.NewServer(http.HandlerFunc(salle.Ws))
	defer srv.Close()

	entete := http.Header{"Origin": {OrigineAttendue}}
	a, _, err := websocket.DefaultDialer.Dial(adresse(srv, "Camille"), entete)
	if err != nil {
		t.Fatal(err)
	}
	defer a.Close()
	b, _, err := websocket.DefaultDialer.Dial(adresse(srv, "Sam"), entete)
	if err != nil {
		t.Fatal(err)
	}
	defer b.Close()
	attendre(t, salle, 2)

	if err := a.WriteMessage(websocket.TextMessage, []byte("salut")); err != nil {
		t.Fatal(err)
	}
	for nom, c := range map[string]*websocket.Conn{"Camille": a, "Sam": b} {
		c.SetReadDeadline(time.Now().Add(3 * time.Second))
		_, msg, err := c.ReadMessage()
		if err != nil {
			t.Fatalf("%s n'a rien reçu : %v", nom, err)
		}
		if string(msg) != "Camille : salut" {
			t.Errorf("%s a reçu %q, attendu %q", nom, msg, "Camille : salut")
		}
	}

	// Quand une personne part, la salle l'oublie.
	a.Close()
	attendre(t, salle, 1)
}
