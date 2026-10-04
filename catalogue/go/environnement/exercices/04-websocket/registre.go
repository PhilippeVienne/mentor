package main

import "github.com/gorilla/websocket"

// Ajoute enregistre la connexion et le nom de la personne dans s.conns. Utilise le mutex.
func (s *Salle) Ajoute(c *websocket.Conn, nom string) {
	panic("à écrire")
}

// Retire supprime la connexion de s.conns (delete). Utilise le mutex.
func (s *Salle) Retire(c *websocket.Conn) {
	panic("à écrire")
}
