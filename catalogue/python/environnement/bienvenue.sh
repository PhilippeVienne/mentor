# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement Python.
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton environnement Python Mentor ($(python3 --version 2>&1))."
    echo "Ton dossier de travail est /workspace : il est effacé quand l'environnement s'arrête."
    echo "Lance les tests d'un exercice avec : pytest -q"
fi
