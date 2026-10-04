import csv

from django.http import HttpResponse
from django.utils import timezone
from django_filters.rest_framework import DjangoFilterBackend
from rest_framework import filters, viewsets
from rest_framework.decorators import action
from rest_framework.permissions import IsAuthenticated
from rest_framework.response import Response

from .models import Asso, Evenement
from .pagination import EvenementsPagination
from .permissions import HasRoleStaff, HasRoleStaffOrReadOnly
from .serializers import AssoSerializer, EvenementSerializer


class AssoViewSet(viewsets.ReadOnlyModelViewSet):
    queryset = Asso.objects.prefetch_related("evenements").order_by("nom")
    serializer_class = AssoSerializer


class EvenementViewSet(viewsets.ModelViewSet):
    queryset = Evenement.objects.select_related("asso").order_by("date")
    serializer_class = EvenementSerializer
    permission_classes = [IsAuthenticated, HasRoleStaffOrReadOnly]
    pagination_class = EvenementsPagination
    filter_backends = [filters.SearchFilter, filters.OrderingFilter, DjangoFilterBackend]
    search_fields = ["titre", "^asso__nom"]
    ordering_fields = ["date", "places"]
    filterset_fields = ["ouvert", "asso"]

    def get_queryset(self):
        queryset = super().get_queryset()
        if self.request.query_params.get("a_venir") == "true":
            queryset = queryset.filter(date__gte=timezone.now())
        return queryset

    @action(detail=True, methods=["post"], permission_classes=[HasRoleStaff])
    def fermer(self, request, pk=None):
        evenement = self.get_object()
        evenement.ouvert = False
        evenement.save(update_fields=["ouvert"])
        return Response(self.get_serializer(evenement).data)

    @action(detail=False, methods=["get"], permission_classes=[HasRoleStaff])
    def export(self, request):
        reponse = HttpResponse(content_type="text/csv")
        reponse["Content-Disposition"] = 'attachment; filename="evenements.csv"'
        writer = csv.writer(reponse)
        writer.writerow(["titre", "asso", "date"])
        for e in self.filter_queryset(self.get_queryset()):
            writer.writerow([e.titre, e.asso.nom, e.date.isoformat()])
        return reponse
