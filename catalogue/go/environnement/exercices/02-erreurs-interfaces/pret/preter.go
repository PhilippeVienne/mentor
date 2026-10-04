package pret

// Preter prête le titre au membre.
//   - titre indisponible : renvoie une erreur qui enveloppe ErrIndisponible ;
//   - membre ayant déjà 2 jeux : renvoie un *ErreurQuota ;
//   - sinon : le jeu n'est plus disponible, l'emprunt est compté, et la méthode renvoie nil.
func (s *Stock) Preter(titre, membre string) error {
	panic("à écrire")
}
