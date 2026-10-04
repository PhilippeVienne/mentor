# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement « code hérité ».
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton environnement « code hérité » Mentor ($(python3 --version 2>&1))."
    echo "Quatre Django sont installés côte à côte : /opt/venvs/django31, django32, django42 et django52."
    echo "Lance une version précise avec : /opt/venvs/django42/bin/python manage.py test"
    echo "Ton dossier de travail est /workspace : il est effacé quand l'environnement s'arrête."
fi
