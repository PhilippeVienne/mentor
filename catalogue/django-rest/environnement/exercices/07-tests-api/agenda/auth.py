import jwt
from django.conf import settings
from django.contrib.auth import get_user_model
from rest_framework import exceptions
from rest_framework.authentication import BaseAuthentication, get_authorization_header


class KeycloakJWTAuthentication(BaseAuthentication):
    def authenticate(self, request):
        parts = get_authorization_header(request).split()
        if not parts or parts[0].lower() != b"bearer":
            return None
        if len(parts) != 2:
            raise exceptions.AuthenticationFailed("En-tête Authorization invalide.")
        try:
            claims = jwt.decode(
                parts[1],
                settings.OIDC_PUBLIC_KEY,
                algorithms=["RS256"],
                audience=settings.OIDC_CLIENT_ID,
            )
        except jwt.PyJWTError:
            raise exceptions.AuthenticationFailed("Jeton invalide ou expiré.")
        user, _ = get_user_model().objects.get_or_create(username=claims["email"])
        return user, claims

    def authenticate_header(self, request):
        return 'Bearer realm="agenda"'
