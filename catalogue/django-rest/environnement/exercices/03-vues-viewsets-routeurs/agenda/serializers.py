from django.utils import timezone
from rest_framework import serializers

from .models import Asso, Evenement


class EvenementSerializer(serializers.ModelSerializer):
    asso = serializers.SlugRelatedField(slug_field="nom", queryset=Asso.objects.all())
    places_restantes = serializers.SerializerMethodField()

    class Meta:
        model = Evenement
        fields = ("id", "asso", "titre", "date", "places", "places_restantes", "ouvert")
        read_only_fields = ("id",)

    def get_places_restantes(self, obj):
        return obj.places - obj.inscriptions.count()

    def validate_places(self, value):
        if value < 1:
            raise serializers.ValidationError("Il faut au moins une place.")
        return value

    def validate(self, data):
        if data.get("ouvert", True) and data["date"] < timezone.now():
            raise serializers.ValidationError("Un événement passé ne peut pas être ouvert.")
        return data


class AssoSerializer(serializers.ModelSerializer):
    evenements = EvenementSerializer(many=True, read_only=True)

    class Meta:
        model = Asso
        fields = ("id", "nom", "evenements")
