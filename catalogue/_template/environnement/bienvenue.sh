# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement.
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton environnement Mentor !"
    echo "Ton dossier de travail est /workspace. Il est effacé quand l'environnement s'arrête."
fi
