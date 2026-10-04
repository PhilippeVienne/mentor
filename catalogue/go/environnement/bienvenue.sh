# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement Go.
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton environnement Go Mentor ($(go version 2>&1))."
    echo "Ton dossier de travail est /workspace : il est effacé quand l'environnement s'arrête."
    echo "Pas d'Internet ici : les modules des leçons sont déjà installés. Lance les tests avec : go test ./..."
fi
