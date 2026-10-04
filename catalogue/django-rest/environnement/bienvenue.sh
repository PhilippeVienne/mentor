# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement Django REST framework.
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton environnement Django REST framework Mentor ($(python3 --version 2>&1), Django $(python3 -c 'import django; print(django.get_version())' 2>/dev/null))."
    echo "Ton dossier de travail est /workspace : il est effacé quand l'environnement s'arrête."
    echo "Lance les tests d'un exercice avec : pytest -q"
fi
