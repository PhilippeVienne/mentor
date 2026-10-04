# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement TypeScript.
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton environnement TypeScript Mentor (Node.js $(node --version), TypeScript $(tsc --version | cut -d' ' -f2))."
    echo "Ton dossier de travail est /workspace : il est effacé quand l'environnement s'arrête."
    echo "Vérifie les types avec : npx tsc --noEmit    Lance un fichier avec : npx tsx fichier.ts"
fi
