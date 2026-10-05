# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement Docker.
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton terminal Docker Mentor (vrai moteur Docker, sur Debian)."
    echo "Ton dossier de travail est /workspace : il est effacé, comme tes conteneurs, quand l'environnement s'arrête."
    echo "Pas d'accès à Internet ici : les images des leçons viennent d'un miroir local de Docker Hub,"
    echo "et les paquets Python de la leçon 1 du dossier /opt/mentor/wheels."
fi
