from django.http import HttpResponseForbidden, JsonResponse

from .keycloak import verifier_jeton
from .models import Adherent
from .utils import etiquette


def liste(request):
    noms = [etiquette(a) for a in Adherent.objects.order_by("nom")]
    return JsonResponse({"adherents": noms})


def prive(request):
    jeton = request.headers.get("Authorization", "")
    if not verifier_jeton(jeton):
        return HttpResponseForbidden("Jeton refusé")
    return JsonResponse({"nombre": Adherent.objects.count()})
