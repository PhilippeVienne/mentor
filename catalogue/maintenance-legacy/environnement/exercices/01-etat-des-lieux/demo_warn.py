from django.conf import settings
settings.configure()
from django.utils.html import format_html
print(format_html("<b>Bonjour</b>"))
