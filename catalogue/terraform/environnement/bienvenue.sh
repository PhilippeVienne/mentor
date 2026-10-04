# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement Terraform.
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton environnement Terraform Mentor ($(terraform version 2>/dev/null | head -n 1))."
    echo "Ton dossier de travail est /workspace : il est effacé quand l'environnement s'arrête."
    echo "Ici, Terraform ne pilote que des fichiers locaux : ni Kubernetes, ni Helm, ni Internet."
fi
