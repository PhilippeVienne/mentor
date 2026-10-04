package main

import (
	"log"
	"net/http"
)

func main() {
	salle := NouvelleSalle()
	http.HandleFunc("/ws", salle.Ws)
	log.Fatal(http.ListenAndServe(":8080", nil))
}
