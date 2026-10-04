from django.db import migrations, models


class Migration(migrations.Migration):
    initial = True

    dependencies = []

    operations = [
        migrations.CreateModel(
            name="Adherent",
            fields=[
                ("id", models.AutoField(auto_created=True, primary_key=True, serialize=False, verbose_name="ID")),
                ("nom", models.CharField(max_length=100, verbose_name="nom")),
                ("email", models.EmailField(blank=True, max_length=254, verbose_name="courriel")),
            ],
        ),
    ]
