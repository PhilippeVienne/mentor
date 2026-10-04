import django.db.models.deletion
from django.db import migrations, models


class Migration(migrations.Migration):

    initial = True

    dependencies = [
    ]

    operations = [
        migrations.CreateModel(
            name='Asso',
            fields=[
                ('id', models.BigAutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('nom', models.CharField(max_length=100, unique=True)),
            ],
        ),
        migrations.CreateModel(
            name='Evenement',
            fields=[
                ('id', models.BigAutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('titre', models.CharField(max_length=200)),
                ('date', models.DateTimeField()),
                ('places', models.PositiveIntegerField(default=30)),
                ('ouvert', models.BooleanField(default=True)),
                ('asso', models.ForeignKey(on_delete=django.db.models.deletion.CASCADE, related_name='evenements', to='agenda.asso')),
            ],
        ),
        migrations.CreateModel(
            name='Inscription',
            fields=[
                ('id', models.BigAutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('email', models.EmailField(max_length=254)),
                ('evenement', models.ForeignKey(on_delete=django.db.models.deletion.CASCADE, related_name='inscriptions', to='agenda.evenement')),
            ],
        ),
    ]
