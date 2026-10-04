"""Un petit client de l'API des événements, écrit avec le client de test de DRF (aucun serveur à lancer)."""
from rest_framework.test import APIClient

client = APIClient()


# À toi de jouer : ajoute ici, une par une, les fonctions lister(), lire(pk), creer(corps), supprimer(pk),
# introuvable() et invalide(). Chacune envoie une requête avec `client` et retourne la réponse.
