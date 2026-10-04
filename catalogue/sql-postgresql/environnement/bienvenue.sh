# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement SQL.
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton environnement PostgreSQL Mentor."
    echo "Ouvre la base du labo avec : psql   (quitte avec \\q). Ton dossier de travail est /workspace."
    echo "Le serveur tourne dans ce conteneur ; tout est effacé quand l'environnement s'arrête."
fi
