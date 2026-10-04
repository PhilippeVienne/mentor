from django.utils import timezone
from rest_framework import viewsets
from rest_framework.decorators import action
from rest_framework.permissions import IsAuthenticated
from rest_framework.response import Response

from .models import Asso, Evenement
from .permissions import HasRoleStaff, HasRoleStaffOrReadOnly
from .serializers import AssoSerializer, EvenementSerializer


class AssoViewSet(viewsets.ReadOnlyModelViewSet):
    queryset = Asso.objects.prefetch_related("evenements").order_by("nom")
    serializer_class = AssoSerializer


class EvenementViewSet(viewsets.ModelViewSet):
    queryset = Evenement.objects.select_related("asso").order_by("date")
    serializer_class = EvenementSerializer
    permission_classes = [IsAuthenticated, HasRoleStaffOrReadOnly]
    # À toi de jouer : filtres (filter_backends, search_fields, ordering_fields, filterset_fields),
    # pagination (pagination_class) et action d'export CSV.

    def get_queryset(self):
        queryset = super().get_queryset()
        if self.request.query_params.get("a_venir") == "true":
            queryset = queryset.filter(date__gte=timezone.now())
        return queryset

    @action(detail=True, methods=["post"], permission_classes=[HasRoleStaff])
    def fermer(self, request, pk=None):
        """Ferme les inscriptions à l'événement."""
        evenement = self.get_object()
        evenement.ouvert = False
        evenement.save(update_fields=["ouvert"])
        return Response(self.get_serializer(evenement).data)
