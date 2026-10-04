## Construire et publier

| Commande | Effet |
| --- | --- |
| `docker build -t nom:tag .` | Construit une image depuis le Dockerfile du dossier |
| `docker build -f Autre.Dockerfile .` | Utilise un autre fichier de recette |
| `docker tag <source> <destination>` | Ajoute une étiquette (ex. pour un registry) |
| `docker login <registry>` | Se connecter à un registry |
| `docker push <image:tag>` | Publier une image |

## Dockerfile

| Instruction | Rôle |
| --- | --- |
| `FROM image:tag` | Image de départ (versions épinglées !) |
| `WORKDIR /app` | Dossier de travail |
| `COPY src dst` | Copie des fichiers dans l'image |
| `RUN commande` | Exécutée **à la construction** |
| `CMD ["prog", "arg"]` | Exécutée **au démarrage** |
| `EXPOSE 5000` | Documente le port (ne le publie pas) |
| `USER app` | Exécute en non-root |

## Volumes et réseaux

| Commande | Effet |
| --- | --- |
| `docker volume create` / `ls` / `rm` / `inspect` | Gérer les volumes |
| `docker run -v nom:/chemin <image>` | Monter un volume nommé |
| `docker run -v $(pwd):/chemin <image>` | Bind mount du dossier courant |
| `docker network create` / `ls` / `rm` / `inspect` | Gérer les réseaux |
| `docker run --network n <image>` | Rattacher un conteneur à un réseau |

## Docker Compose

| Commande | Effet |
| --- | --- |
| `docker compose up -d` | Démarre (et recrée si besoin) les services |
| `docker compose ps` | État des services |
| `docker compose logs <service>` | Journaux d'un service |
| `docker compose stop` | Arrête sans supprimer |
| `docker compose down` | Supprime conteneurs et réseau |
| `docker compose down -v` | …et aussi les volumes |

:::warning Pas de secret dans une image
Une image publiée est lisible par tous ses utilisateur·rice·s, couche par couche. Fournis les secrets au démarrage (variables d'environnement, `.env` non commité).
:::
