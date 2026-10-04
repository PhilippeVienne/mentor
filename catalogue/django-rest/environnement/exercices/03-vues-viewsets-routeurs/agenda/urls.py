from django.urls import include, path
from rest_framework import routers

from . import views

router = routers.DefaultRouter()
# À toi de jouer : enregistre ici tes viewsets avec router.register(...).

urlpatterns = [
    path("v1/", include(router.urls)),
]
