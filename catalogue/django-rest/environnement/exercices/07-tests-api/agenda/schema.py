from rest_framework.schemas.openapi import AutoSchema


class SchemaAgenda(AutoSchema):
    """Contournement : DjangoFilterBackend n'a pas get_schema_operation_parameters avec DRF 3.18."""

    def get_filter_parameters(self, path, method):
        if not self.allows_filters(path, method):
            return []
        parametres = []
        for backend in self.view.filter_backends:
            if hasattr(backend, "get_schema_operation_parameters"):
                parametres += backend().get_schema_operation_parameters(self.view)
        return parametres
