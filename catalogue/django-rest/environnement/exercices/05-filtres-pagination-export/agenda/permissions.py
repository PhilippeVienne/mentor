from django.conf import settings
from rest_framework.permissions import SAFE_METHODS, BasePermission


def roles_du_jeton(request):
    claims = request.auth if isinstance(request.auth, dict) else {}
    acces = claims.get("resource_access", {}).get(settings.OIDC_CLIENT_ID, {})
    return set(acces.get("roles", []))


class HasRole(BasePermission):
    required_roles = []

    def has_permission(self, request, view):
        return set(self.required_roles) <= roles_du_jeton(request)


class HasRoleStaff(HasRole):
    required_roles = ["staff"]


class HasRoleStaffOrReadOnly(HasRoleStaff):
    def has_permission(self, request, view):
        return request.method in SAFE_METHODS or super().has_permission(request, view)
