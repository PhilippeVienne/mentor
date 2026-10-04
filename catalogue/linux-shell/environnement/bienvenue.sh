# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement Linux et shell.
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton terminal Linux Mentor (Debian)."
    echo "Ton dossier de travail est /workspace : il est effacé quand l'environnement s'arrête."
    echo "Les fichiers d'entraînement sont dans /opt/exercices (le labo de chaque leçon les copie pour toi)."
fi
