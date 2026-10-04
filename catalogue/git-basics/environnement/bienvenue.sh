# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement Git basics.
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton terminal Git Mentor (vrai Git, sur Debian)."
    echo "Ton dossier de travail est /workspace : il est effacé quand l'environnement s'arrête."
    echo "Le « GitLab » de la leçon 6 est un dépôt local : les commandes sont les mêmes que sur gitlab.example.org."
fi
