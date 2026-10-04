package main

import (
	"net/http"

	"github.com/gorilla/websocket"
)

// OrigineAttendue est le seul site autorisé à ouvrir un WebSocket vers ce serveur.
const OrigineAttendue = "https://ludotheque.example.org"

// mise transforme une requête HTTP en WebSocket (« upgrade »).
// CheckOrigin reçoit la requête et répond true pour l'accepter, false pour la refuser.
var mise = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		// À écrire : compare l'en-tête « Origin » de la requête à OrigineAttendue.
		panic("à écrire")
	},
}
