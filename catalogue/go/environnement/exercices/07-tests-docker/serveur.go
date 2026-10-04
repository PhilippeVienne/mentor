package main

import (
	"encoding/json"
	"net/http"
	"sync"
)

type Jeu struct {
	Titre   string `json:"titre"`
	Joueurs int    `json:"joueurs"`
	Note    string `json:"note,omitempty"`
}

type serveur struct {
	mu   sync.Mutex
	jeux []Jeu
}

func (s serveur) liste(w http.ResponseWriter, r *http.Request) {
	s.mu.Lock()
	defer s.mu.Unlock()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(s.jeux)
}

func (s *serveur) ajoute(w http.ResponseWriter, r *http.Request) {
	var j Jeu
	if err := json.NewDecoder(r.Body).Decode(&j); err != nil {
		http.Error(w, `{"erreur":"JSON invalide"}`, http.StatusBadRequest)
		return
	}
	if j.Titre == "" {
		http.Error(w, `{"erreur":"titre obligatoire"}`, http.StatusBadRequest)
		return
	}
	s.mu.Lock()
	s.jeux = append(s.jeux, j)
	s.mu.Unlock()
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(j)
}
