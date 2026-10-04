from django.db import models


class Asso(models.Model):
    nom = models.CharField(max_length=100, unique=True)


class Evenement(models.Model):
    asso = models.ForeignKey(Asso, on_delete=models.CASCADE, related_name="evenements")
    titre = models.CharField(max_length=200)
    date = models.DateTimeField()
    places = models.PositiveIntegerField(default=30)
    ouvert = models.BooleanField(default=True)


class Inscription(models.Model):
    evenement = models.ForeignKey(Evenement, on_delete=models.CASCADE, related_name="inscriptions")
    email = models.EmailField()
