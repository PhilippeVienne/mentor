# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement Kubernetes et Helm.
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton atelier Kubernetes et Helm de l'équipe."
    echo "Il n'y a PAS de cluster ici : tu écris et valides des fichiers (kubeconform, helm lint, helm template)."
    echo "Ton dossier de travail est /workspace : il est effacé quand l'environnement s'arrête."
fi
