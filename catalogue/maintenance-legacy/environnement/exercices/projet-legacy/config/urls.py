from django.conf.urls import include, url

urlpatterns = [
    url(r"^adherents/", include("adherents.urls")),
]
