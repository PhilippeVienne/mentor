"""Outils de test pour les jetons JWT : aucune clé réelle, les paires RSA sont fabriquées au lancement des tests."""
import time

import jwt
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import rsa


def _paire():
    privee = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    pem_privee = privee.private_bytes(
        serialization.Encoding.PEM,
        serialization.PrivateFormat.PKCS8,
        serialization.NoEncryption(),
    ).decode()
    pem_publique = (
        privee.public_key()
        .public_bytes(serialization.Encoding.PEM, serialization.PublicFormat.SubjectPublicKeyInfo)
        .decode()
    )
    return pem_privee, pem_publique


CLE_PRIVEE, CLE_PUBLIQUE = _paire()
AUTRE_CLE_PRIVEE, _ = _paire()


def fabriquer_jeton(email="alice@example.org", roles=(), expire_dans=3600, cle=None, audience="agenda-api"):
    """Retourne un jeton JWT signé (RS256) ; `roles` sont rangés dans resource_access comme le fait Keycloak."""
    claims = {
        "email": email,
        "aud": audience,
        "exp": int(time.time()) + expire_dans,
        "resource_access": {"agenda-api": {"roles": list(roles)}},
    }
    return jwt.encode(claims, cle or CLE_PRIVEE, algorithm="RS256")
