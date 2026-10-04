# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement HTML/CSS/JavaScript.
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton environnement web Mentor (Node.js $(node --version 2>&1))."
    echo "Ton dossier de travail est /workspace : il est effacé quand l'environnement s'arrête."
    echo "Il n'y a pas de navigateur ici : lance « verifier-page index.html » pour voir ce qu'en ferait un navigateur."
fi
