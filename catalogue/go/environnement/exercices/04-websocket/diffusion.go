package main

// Diffuse envoie le message (texte) à toutes les connexions de la salle.
// Une seule personne à la fois doit écrire dans une connexion : garde le mutex pendant toute la boucle.
// Si l'écriture échoue pour une connexion, affiche l'erreur avec log.Println et continue avec les autres.
func (s *Salle) Diffuse(msg []byte) {
	panic("à écrire")
}
