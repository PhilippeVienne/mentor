# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement React et Next.js.
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton environnement React Mentor (Node.js $(node --version), React $(node -p "require('react').version"))."
    echo "Ton dossier de travail est /workspace : il est effacé quand l'environnement s'arrête."
    echo "Lance les tests avec : npx vitest run    Vérifie les types avec : npx tsc --noEmit"
fi
