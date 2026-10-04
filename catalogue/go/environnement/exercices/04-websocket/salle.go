package main

import (
	"fmt"
	"net/http"
	"sync"

	"github.com/gorilla/websocket"
)

// Salle garde la liste des connexions ouvertes (avec le nom de chaque personne).
// Plusieurs connexions s'exécutent en parallèle : le mutex protège la map conns.
type Salle struct {
	mu    sync.Mutex
	conns map[*websocket.Conn]string
}

func NouvelleSalle() *Salle {
	return &Salle{conns: map[*websocket.Conn]string{}}
}

// Taille renvoie le nombre de connexions enregistrées.
func (s *Salle) Taille() int {
	s.mu.Lock()
	defer s.mu.Unlock()
	return len(s.conns)
}

// Ws est le handler de la route /ws : /ws?nom=Camille
func (s *Salle) Ws(w http.ResponseWriter, r *http.Request) {
	nom := r.URL.Query().Get("nom")
	if nom == "" {
		http.Error(w, "nom obligatoire", http.StatusBadRequest)
		return
	}
	conn, err := mise.Upgrade(w, r, nil)
	if err != nil {
		return // Upgrade a déjà répondu au client
	}
	defer conn.Close()

	s.Ajoute(conn, nom)
	defer s.Retire(conn)

	for {
		_, msg, err := conn.ReadMessage()
		if err != nil {
			return // la personne a fermé sa page
		}
		s.Diffuse([]byte(fmt.Sprintf("%s : %s", nom, msg)))
	}
}
