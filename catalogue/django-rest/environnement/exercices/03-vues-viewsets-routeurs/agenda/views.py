from django.utils import timezone
from rest_framework import viewsets
from rest_framework.decorators import action
from rest_framework.response import Response

from .models import Asso, Evenement
from .serializers import AssoSerializer, EvenementSerializer

# À toi de jouer : écris ici EvenementViewSet puis AssoViewSet.
