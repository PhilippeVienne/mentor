from rest_framework import viewsets

from .models import Asso, Evenement
from .serializers import AssoSerializer, EvenementSerializer


class AssoViewSet(viewsets.ReadOnlyModelViewSet):
    queryset = Asso.objects.prefetch_related("evenements").order_by("nom")
    serializer_class = AssoSerializer


class EvenementViewSet(viewsets.ModelViewSet):
    queryset = Evenement.objects.select_related("asso").order_by("date")
    serializer_class = EvenementSerializer
