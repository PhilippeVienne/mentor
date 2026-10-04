package main

import "github.com/golang-jwt/jwt/v5"

// Claims décrit le contenu du jeton : nos champs, plus les champs standards (exp, iss…)
// apportés par la structure embarquée jwt.RegisteredClaims.
type Claims struct {
	Nom   string   `json:"name"`
	Roles []string `json:"roles"`
	jwt.RegisteredClaims
}
