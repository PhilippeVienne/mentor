---
id: dockerfile
titre: Écrire un Dockerfile
resume: "Construire ta propre image à partir d'une application Python."
duree: 20
objectifs:
  - "Écrire un Dockerfile pour une application Python/Flask"
  - "Construire une image avec `docker build` et la lancer"
  - "Expliquer les couches d'une image et ordonner les instructions pour profiter du cache"
  - "Distinguer `RUN` (construction) et `CMD` (démarrage)"
---

Jusqu'ici tu as utilisé des images toutes faites. Un **Dockerfile** est la recette pour fabriquer la *tienne* : chaque instruction ajoute une **couche** à l'image.

```mermaid
flowchart LR
    A[Dockerfile<br/>+ ton code] -->|docker build| B[(Image)]
    B -->|docker run| C[Conteneur 1]
    B -->|docker run| D[Conteneur 2]
```

## Les instructions essentielles

| Instruction | Rôle |
| --- | --- |
| `FROM` | Image de départ (ex. `python:3.13-slim`). Toujours la première. |
| `WORKDIR` | Dossier de travail dans l'image (créé s'il n'existe pas). |
| `COPY` | Copie des fichiers de ton projet vers l'image. |
| `RUN` | Exécute une commande **à la construction** (ex. installer les dépendances). |
| `EXPOSE` | Documente le port d'écoute. Ne publie rien : c'est `-p` qui le fait. |
| `CMD` | Commande lancée **au démarrage** du conteneur. |

## Un exemple complet

L'application Flask du projet (`app.py` + `requirements.txt`) est déjà dans le dossier du labo. Voici son Dockerfile :

```dockerfile file=Dockerfile
FROM python:3.13-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt
COPY . .
EXPOSE 5000
CMD ["python", "app.py"]
```

## Construire et lancer

```shell run
docker build -t demo-app .
docker run -d --name app -p 5000:5000 demo-app
curl localhost:5000
```

`-t demo-app` donne un nom à l'image et le `.` final indique le **contexte de build** : le dossier courant.

## Les couches et le cache

Chaque instruction produit une couche. Docker **réutilise** les couches inchangées : c'est ce qui rend les rebuilds si rapides.

![Après modification du code, seules les couches COPY . . et suivantes sont reconstruites](images/couches-cache.svg)

:::tip L'ordre compte
Une couche invalidée invalide **toutes les suivantes**. En copiant `requirements.txt` *avant* le reste du code, l'installation des dépendances n'est refaite que si elles changent. Modifie `app.py` et reconstruis : tu verras `CACHED` devant les étapes situées avant la modification (dans le simulateur, bien visible sur `COPY requirements.txt` et `RUN pip install`), alors que `COPY . .` et la suite sont refaits.
:::

:::info CMD sous forme de liste
Écris `CMD ["python", "app.py"]` (forme *exec*) plutôt que `CMD python app.py` : le programme devient le processus principal du conteneur et reçoit directement les signaux de `docker stop`.
:::

:::warning RUN ≠ CMD
`RUN` s'exécute à la **construction** de l'image (il laisse une couche). `CMD` s'exécute au **démarrage** du conteneur. Confondre les deux est l'erreur n°1.
:::

## À retenir

- Un Dockerfile = une recette ; `docker build -t nom .` fabrique l'image.
- Mets ce qui change rarement en premier (dépendances), ce qui change souvent en dernier (code).
- `EXPOSE` documente, `-p` publie.

## Entraîne-toi

:::labo
intro: |
  Le projet contient `app.py` et `requirements.txt`. Écris le Dockerfile (bouton « Créer ce fichier » dans la leçon, ou `nano Dockerfile`) puis construis ton image.
fichiers:
  app.py: |
    from flask import Flask

    app = Flask(__name__)


    @app.route("/")
    def hello():
        return "Bonjour depuis Mentor !"


    if __name__ == "__main__":
        app.run(host="0.0.0.0", port=5000)
  requirements.txt: |
    flask==3.0.3
etapes:
  - texte: 'Crée un fichier `Dockerfile` qui commence par `FROM`'
    indice: Clique sur « Créer ce fichier dans le labo » sous le Dockerfile de la leçon.
    verif:
      - fichier-contient: [Dockerfile, '^FROM ']
    solution:
      - ecrire:
          Dockerfile: |
            FROM python:3.13-slim
            WORKDIR /app
            COPY requirements.txt .
            RUN pip install -r requirements.txt
            COPY . .
            EXPOSE 5000
            CMD ["python", "app.py"]
  - texte: "Construis l'image : `docker build -t demo-app .`"
    indice: "`docker build` avec `-t` pour le nom de l'image, et un point final : le contexte de build est le dossier courant."
    verif:
      - image-presente: demo-app
    solution:
      - docker build -t demo-app .
  - texte: 'Lance-la : détachée, nommée `app`, port `5000`'
    indice: "C'est un `docker run` comme celui de nginx, mais avec ton image : le port 5000 est celui d'écoute de Flask."
    verif:
      - conteneur-actif: app
    solution:
      - 'docker run -d --name app -p 5000:5000 demo-app'
  - texte: 'Interroge ton application : `curl localhost:5000`'
    indice: "Même principe que pour nginx : le port publié côté machine."
    apres: [3]
    verif:
      - commande: '^curl .*5000'
    solution:
      - 'curl localhost:5000'
  - texte: 'Modifie `app.py` (change le message), puis reconstruis : observe le `CACHED`'
    indice: "Édite `app.py` (clique sur le fichier dans le labo), puis refais exactement le même `docker build` qu'avant."
    verif:
      - commande-compte: [^docker build, 2]
      - fichier-modifie: app.py
    solution:
      - ecrire:
          app.py: |
            from flask import Flask

            app = Flask(__name__)


            @app.route("/")
            def hello():
                return "Bonjour depuis Mentor !"


            if __name__ == "__main__":
                app.run(host="0.0.0.0", port=5000)
            # modifié
      - docker build -t demo-app .
:::

## Vérifie tes acquis

:::quiz
Quelle est la différence entre RUN et CMD ?

- [ ] Aucune
- [x] RUN s'exécute à la construction de l'image ; CMD au démarrage du conteneur
- [ ] CMD s'exécute à la construction ; RUN au démarrage
- [ ] RUN est réservé à pip

> RUN produit une couche de l'image (ex. dépendances installées). CMD définit ce qui est lancé quand on démarre un conteneur.
:::

:::quiz
Pourquoi copier requirements.txt avant le reste du code ?

- [ ] Parce que pip refuse de s'exécuter avant la copie du reste du code
- [x] Pour profiter du cache : les dépendances ne sont réinstallées que si requirements.txt change
- [ ] Pour réduire la taille de l'image
- [ ] Pour que l'image finale soit plus petite

> Une couche invalidée invalide toutes les suivantes. Mieux vaut mettre ce qui change rarement en premier.
:::

:::quiz
EXPOSE 5000 rend-il le port accessible depuis ta machine ?

- [ ] Oui, automatiquement
- [x] Non : il documente le port ; il faut publier avec -p
- [ ] Oui, mais seulement en local
- [ ] Oui, dès que le conteneur est lancé avec -d

> EXPOSE est une indication. C'est -p (ou ports: dans Compose) qui crée la redirection.
:::
