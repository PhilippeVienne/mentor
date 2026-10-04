from rest_framework import serializers

from .models import Asso, Evenement


class EvenementSerializer(serializers.ModelSerializer):
    # À toi de jouer : affiche l'association par son nom, ajoute `places_restantes` et les validations.

    class Meta:
        model = Evenement
        fields = ("id", "asso", "titre", "date", "places", "ouvert")


class AssoSerializer(serializers.ModelSerializer):
    # À toi de jouer : ajoute les événements de l'association.

    class Meta:
        model = Asso
        fields = ("id", "nom")
