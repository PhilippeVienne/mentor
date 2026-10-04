# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement CI/CD.
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton environnement CI/CD Mentor."
    echo "Il n'y a pas de vrai runner GitLab ici : verifier-ci lit ton .gitlab-ci.yml et en contrôle la structure."
    echo "Outils : verifier-ci, verifier-renovate, lint-yaml. Ton dossier de travail est /workspace (effacé à l'arrêt)."
fi
