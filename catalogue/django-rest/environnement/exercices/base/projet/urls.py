from django.urls import include, path

urlpatterns = [
    path("", include("agenda.urls")),
]
