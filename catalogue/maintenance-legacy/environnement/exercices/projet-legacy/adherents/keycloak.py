"""Appel au serveur d'authentification (Keycloak). Il n'existe pas dans l'environnement d'exercice."""
import urllib.request

URL_KEYCLOAK = "http://keycloak.exemple.invalid/realms/exemple/protocol/openid-connect/userinfo"


def verifier_jeton(jeton):
    """Demande à Keycloak si le jeton est valide (échoue sans réseau)."""
    requete = urllib.request.Request(URL_KEYCLOAK, headers={"Authorization": f"Bearer {jeton}"})
    with urllib.request.urlopen(requete, timeout=2) as reponse:
        return reponse.status == 200
