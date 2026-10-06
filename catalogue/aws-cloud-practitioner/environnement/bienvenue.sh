# Message affiché à l'ouverture d'un terminal (shell de connexion) dans l'environnement AWS.
. /opt/outils/mentor-aws.sh
if [ -n "$PS1" ]; then
    echo "Bienvenue dans ton labo AWS Mentor : un émulateur LOCAL d'AWS (MiniStack), pas le vrai AWS."
    echo "Rien n'est facturé, rien ne sort de cet environnement, et tout est effacé à l'arrêt."
    echo "La commande aws parle déjà à l'émulateur (compte fictif 000000000000, région eu-west-3). Essaie : aws sts get-caller-identity"
    echo "Si une commande aws ne répond pas : demarrer-aws"
fi
