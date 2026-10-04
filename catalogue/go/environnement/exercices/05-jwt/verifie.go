package main

// Verifie décode un jeton brut (le texte « en-tête.contenu.signature ») et renvoie son contenu,
// ou une erreur si le jeton n'est pas acceptable.
// Utilise jwt.ParseWithClaims avec une fonction qui renvoie la clé `secret`.
//
// Version finale attendue :
//   - la signature doit correspondre à la clé ;
//   - le jeton doit avoir une date d'expiration (option jwt.WithExpirationRequired()) ;
//   - seul l'algorithme HS256 est accepté (option jwt.WithValidMethods).
func Verifie(brut string) (*Claims, error) {
	panic("à écrire")
}
