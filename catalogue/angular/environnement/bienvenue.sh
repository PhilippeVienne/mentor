# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement Angular.
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton environnement Angular Mentor (Angular 20, Node.js $(node --version 2>&1))."
    echo "Ton dossier de travail est /workspace : il est effacé quand l'environnement s'arrête."
    echo "Pas de navigateur ni de réseau ici : « ngc -p tsconfig.json --noEmit » vérifie les gabarits, « tester » lance les tests."
fi
