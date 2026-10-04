package main

import (
	"fmt"
	"os"
	"time"
)

func main() {
	if err := Initialise([]byte(os.Getenv("JWT_SECRET"))); err != nil {
		fmt.Println("erreur :", err)
		os.Exit(1)
	}
	jeton, _ := Signe("Camille", []string{"bureau"}, time.Hour)
	c, err := Verifie(jeton)
	if err != nil {
		fmt.Println("jeton refusé :", err)
		os.Exit(1)
	}
	fmt.Println("jeton accepté pour", c.Nom, "- bureau :", c.ARole("bureau"))
}
