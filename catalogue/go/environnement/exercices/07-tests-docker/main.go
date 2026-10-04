package main

import (
	"log"
	"net/http"
)

func main() {
	s := &serveur{}
	mux := http.NewServeMux()
	mux.HandleFunc("GET /jeux", s.liste)
	mux.HandleFunc("POST /jeux", s.ajoute)
	log.Fatal(http.ListenAndServe(":8080", mux))
}
