package main

// ConfigDepuisEnv compose l'adresse de connexion à la base à partir de variables d'environnement
// (jamais de mot de passe écrit dans le code) :
//   - DB_HOST : adresse du serveur, « localhost » par défaut ;
//   - DB_USER : nom d'utilisateur, obligatoire ;
//   - DB_PASSWORD : mot de passe, obligatoire.
//
// Elle renvoie "postgres://UTILISATEUR:MOTDEPASSE@HOTE/ludotheque", ou une erreur si une variable obligatoire manque.
func ConfigDepuisEnv() (string, error) {
	panic("à écrire")
}
