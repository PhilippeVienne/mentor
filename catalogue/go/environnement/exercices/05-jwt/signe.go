package main

import (
	"time"

	"github.com/golang-jwt/jwt/v5"
)

// Signe fabrique un jeton HS256 valable pour la durée donnée (utile aux tests et à la démonstration).
func Signe(nom string, roles []string, duree time.Duration) (string, error) {
	claims := Claims{
		Nom:   nom,
		Roles: roles,
		RegisteredClaims: jwt.RegisteredClaims{
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(duree)),
		},
	}
	return jwt.NewWithClaims(jwt.SigningMethodHS256, claims).SignedString(secret)
}
