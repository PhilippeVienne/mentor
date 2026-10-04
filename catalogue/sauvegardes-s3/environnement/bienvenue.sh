# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement « Sauvegardes ».
. /opt/outils/mentor-labo.sh
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton environnement « Sauvegardes » Mentor."
    echo "Le labo démarre pour toi un stockage S3 (MinIO) et une base PostgreSQL ; tout est local et effacé à l'arrêt."
    echo "Identifiants FACTICES de labo : voir « echo \$URL \$ACCESSKEY ». Alias mc : labo. Ton dossier de travail est /workspace."
fi
