from django.urls import include, path
from rest_framework import routers

from . import views

router = routers.DefaultRouter()
router.register("evenements", views.EvenementViewSet)
router.register("assos", views.AssoViewSet)

urlpatterns = [
    path("v1/", include(router.urls)),
]
