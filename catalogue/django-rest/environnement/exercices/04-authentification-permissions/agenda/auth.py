import jwt
from django.conf import settings
from django.contrib.auth import get_user_model
from rest_framework import exceptions
from rest_framework.authentication import BaseAuthentication, get_authorization_header


class KeycloakJWTAuthentication(BaseAuthentication):
    # À toi de jouer : écris authenticate() puis authenticate_header().
    pass
