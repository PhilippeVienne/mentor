from django.urls import path

from . import views

urlpatterns = [
    path("", views.liste),
    path("prive/", views.prive),
]
