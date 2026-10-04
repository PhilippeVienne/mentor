## Images

| Commande | Effet |
| --- | --- |
| `docker pull <image>` | Télécharge une image depuis un registry |
| `docker images` | Liste les images présentes en local |
| `docker rmi <image>` | Supprime une image (aucun conteneur ne doit l'utiliser) |

## Lancer un conteneur

| Commande | Effet |
| --- | --- |
| `docker run <image>` | Crée **et** démarre un conteneur |
| `docker run -d --name n -p 8080:80 <image>` | Détaché, nommé, port 8080 → 80 |
| `docker run -e VAR=valeur <image>` | Avec une variable d'environnement |
| `docker run --rm <image> <commande>` | Conteneur jetable, supprimé à l'arrêt |
| `docker run -it <image> sh` | Shell interactif dans un nouveau conteneur |

## Observer et gérer

| Commande | Effet |
| --- | --- |
| `docker ps` / `docker ps -a` | Conteneurs en cours / tous les conteneurs |
| `docker stop` / `start` / `restart <nom>` | Arrêter / démarrer / redémarrer |
| `docker rm <nom>` / `docker rm -f <nom>` | Supprimer / forcer la suppression |
| `docker logs <nom>` | Lire les journaux du conteneur |
| `docker exec -it <nom> sh` | Shell dans un conteneur en cours |
| `docker inspect <nom>` | Détails techniques (JSON) |

## En cas de problème

:::tip Le réflexe en 3 temps
1. `docker ps -a` : le conteneur est-il arrêté ? avec quel code de sortie ?
2. `docker logs <nom>` : que dit le message d'erreur ?
3. `docker run -it <image> sh` : explorer à la main.
:::
