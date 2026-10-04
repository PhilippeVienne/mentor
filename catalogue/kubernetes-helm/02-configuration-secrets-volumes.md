---
id: configuration-secrets-volumes
titre: "Configuration, secrets et volumes"
resume: "Sortir la configuration de l'image avec ConfigMap et Secret, et garder les fichiers avec un volume persistant."
duree: 45
objectifs:
  - Choisir entre ConfigMap et Secret pour une variable d'environnement
  - Expliquer pourquoi un Secret n'est pas chiffré par défaut
  - Lire un PersistentVolumeClaim et distinguer ReadWriteOnce de ReadWriteMany
---

## À quoi ça sert, et pourquoi ?

Imagine une recette de cuisine imprimée : le plat est le même chez tout le monde, mais la quantité de sel, elle, dépend des goûts. Une **image Docker** est la recette : on la construit une fois. Les réglages qui dépendent du lieu (l'adresse du serveur de connexion, le mot de passe de la base de données) sont le « sel » : on les fournit **au dernier moment**, depuis l'extérieur. Ainsi, tu ne reconstruis pas l'image pour changer une adresse, et un mot de passe ne se retrouve pas dans l'image que tout le monde peut télécharger.

Une même image Docker doit tourner partout : sur ton poste, en test, en production. Ce qui change d'un endroit à l'autre (l'adresse du SSO, la base de données, les clés d'API) ne doit donc **pas** être dans l'image. Kubernetes propose trois objets pour cela.

## ConfigMap : la configuration non sensible

Le chart de Vitrine range ses réglages publics (adresse du site, URL de Keycloak, identifiant de client) dans un ConfigMap :

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: vitrine-configmap
data:
  SITE_URL: "https://portail.example.org"
  ADHESION_CLIENT_ID: "vitrine"
  APP_DEBUG: "False"
```

Ligne par ligne : `kind: ConfigMap` dit le type d'objet ; `metadata.name` est son nom ; `data` est un dictionnaire `NOM: valeur`. Toutes les valeurs sont des **chaînes de caractères** : c'est pourquoi `False` est écrit entre guillemets.

## Secret : les valeurs sensibles

Le même chart crée un Secret pour les clés d'API et le mot de passe de la base. En voici une version avec des **valeurs factices** :

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: vitrine-secret
type: Opaque
stringData:
  MAILJET_API_KEY: "cle-factice-a-remplacer"
  ADHESION_CLIENT_SECRET: "secret-factice-a-remplacer"
```

Un Secret ressemble à un ConfigMap, avec `type: Opaque` (« contenu libre »). `stringData` te laisse écrire les valeurs en clair ; Kubernetes les stocke ensuite encodées en **base64**, une façon de réécrire du texte avec 64 caractères sûrs. Attention : base64 n'est **pas** du chiffrement, n'importe qui peut le décoder en une commande. L'intérêt d'un Secret est surtout de pouvoir lui donner des **droits d'accès** plus stricts qu'à un ConfigMap.

:::danger Jamais de vrais secrets dans Git
Un Secret commité (même « encodé ») est un secret divulgué. Dans les dépôts de l'équipe, les valeurs sensibles des charts sont à `nil` par défaut ou factices : on les fournit au moment du déploiement. Dans cette formation, tous les secrets sont des exemples.
:::

## Injecter la configuration dans le pod

Le déploiement lit les deux objets d'un coup avec `envFrom` :

```yaml
containers:
  - name: web
    image: registry.example.org/equipe/vitrine:master
    envFrom:
      - configMapRef:
          name: vitrine-configmap
      - secretRef:
          name: vitrine-secret
```

`envFrom` signifie « prends toutes les clés de cet objet » ; `configMapRef` et `secretRef` désignent l'objet par son nom. Chaque clé devient une **variable d'environnement** du conteneur (une valeur que le programme lit au démarrage, comme `SITE_URL`). Pour n'en prendre qu'une, on utilise `secretKeyRef` (c'est ce que fait le **CronJob** de sauvegarde vu dans le parcours *Sauvegardes* : un CronJob est un objet Kubernetes qui lance une tâche à heures régulières, comme la sauvegarde de la nuit) :

```yaml
env:
  - name: ACCESSKEY
    valueFrom:
      secretKeyRef:
        name: s3-credentials
        key: accessKey
```

Ligne par ligne : `env` est la liste des variables d'environnement écrites une par une ; `name: ACCESSKEY` est le nom de la variable vue par le programme ; `valueFrom.secretKeyRef` dit « va chercher la valeur dans un Secret » ; `name: s3-credentials` est le Secret visé et `key: accessKey` la clé à lire à l'intérieur. (S3 est un service de stockage de fichiers en ligne ; ici ce sont des identifiants factices.)

Un ConfigMap ou un Secret peut aussi être **monté en fichier**. Le chart d'Adhésion monte ainsi la configuration nginx (`nginx.conf`) dans `/etc/nginx/conf.d`, en lecture seule.

:::warning Changer un ConfigMap ne relance pas le pod
Les variables d'environnement sont lues **au démarrage** du conteneur. Après une modification, il faut relancer le déploiement : `kubectl rollout restart deployment/vitrine-web`.
:::

## Volumes : garder les fichiers

Le système de fichiers d'un conteneur disparaît avec lui, comme un brouillon jeté à la poubelle. Pour les fichiers à conserver (les médias déposés par les associations, par exemple), on utilise un **volume** : un espace de stockage qui survit aux conteneurs. On ne choisit pas le disque soi-même : on dépose une **demande de volume** (PersistentVolumeClaim, ou PVC), comme on réserve une place dans un parking sans choisir l'emplacement :

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: vitrine-mediafiles-pvc
spec:
  accessModes:
    - ReadWriteMany
  resources:
    requests:
      storage: 5Gi
```

Ligne par ligne : `kind: PersistentVolumeClaim` est le type d'objet, `metadata.name` son nom (qu'on citera dans le pod) et `spec.resources.requests.storage: 5Gi` demande 5 gibioctets ; `accessModes` indique qui pourra lire et écrire (voir le tableau plus bas). Le cluster fournit un volume correspondant ; le pod le **monte** ensuite dans un dossier de ses conteneurs :

```yaml
volumes:
  - name: media-volume
    persistentVolumeClaim:
      claimName: vitrine-mediafiles-pvc
containers:
  - name: web
    volumeMounts:
      - name: media-volume
        mountPath: /app/mediafiles
```

Dans ce bloc, `volumes` déclare le volume au niveau du pod (en le reliant au PVC par `claimName`) et `volumeMounts` choisit le dossier où il apparaît dans le conteneur (`mountPath`). Le mode d'accès compte, car un **nœud** est une machine du cluster et les pods d'un même déploiement peuvent être répartis sur plusieurs nœuds :

| Mode | Sens |
| --- | --- |
| `ReadWriteOnce` | lecture et écriture depuis **un seul** nœud |
| `ReadWriteMany` | lecture et écriture depuis **plusieurs** nœuds à la fois |

Le chart de Vitrine demande `ReadWriteMany` et son README précise qu'il faut « un provisionneur de volumes dynamique pour les volumes RWX ». Le fichier statique, lui, utilise un `emptyDir` : un dossier vide, **perdu** à chaque redémarrage du pod, recalculé au démarrage.

:::tip Sur le cluster de dev local
Le cluster de test local d'`infra-dev` (basé sur k3d, présenté en leçon 5) ne sait pas créer de volume `ReadWriteMany` (voir la leçon 5). Le fichier `keycloak-h2pvc.yaml` y demande donc un `ReadWriteOnce` de 1 Gio avec la classe `local-path`.
:::

:::info À confirmer avec l'équipe Infra
Où sont conservés les secrets de production, comment sont-ils transmis à `helm install`, et quelle classe de stockage fournit les volumes `ReadWriteMany` sur le cluster ? Les dépôts ne le disent pas.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Même atelier que pour la leçon 1 : `kubectl`, `kubeconform` et `verifier-k8s`, **sans cluster** (aucun objet n'est vraiment créé). Dans ton dossier de travail, `deployment.yaml` est un déploiement très simplifié de Vitrine qui cite un ConfigMap et un Secret **qui n'existent pas encore**. Écris-les, puis vérifie que tout est cohérent. Toutes les valeurs sont factices.
commandes:
  - cp /opt/exercices/02-configuration/deployment.yaml .
etapes:
  - texte: 'Fabrique le ConfigMap `vitrine-configmap` dans `configmap.yaml`, avec `SITE_URL` égale à `https://portail.example.org` et `APP_DEBUG` égale à `False` : `kubectl create configmap vitrine-configmap --from-literal=SITE_URL=… --from-literal=APP_DEBUG=False --dry-run=client -o yaml > configmap.yaml` (complète les `…`)'
    indice: 'Chaque `--from-literal=NOM=valeur` ajoute une entrée dans `data`. Contrôle ensuite avec `cat configmap.yaml`.'
    verif:
      - commande-reussit: 'kubeconform configmap.yaml'
      - commande-reussit: "verifier-k8s champ --kind ConfigMap --nom vitrine-configmap --chemin 'data.SITE_URL' --texte https://portail.example.org configmap.yaml"
      - commande-reussit: "verifier-k8s champ --kind ConfigMap --nom vitrine-configmap --chemin 'data.APP_DEBUG' --texte False configmap.yaml"
    solution:
      - kubectl create configmap vitrine-configmap --from-literal=SITE_URL=https://portail.example.org --from-literal=APP_DEBUG=False --dry-run=client -o yaml > configmap.yaml
  - texte: 'Fabrique de la même façon le Secret `vitrine-secret` dans `secret.yaml`, avec une clé `MAILJET_API_KEY` valant `cle-factice-a-remplacer` : `kubectl create secret generic vitrine-secret --from-literal=… --dry-run=client -o yaml > secret.yaml`'
    indice: 'Regarde le résultat avec `cat secret.yaml` : la valeur n''est plus lisible. Kubernetes l''a encodée en base64.'
    verif:
      - commande-reussit: 'kubeconform secret.yaml'
      - commande-reussit: "verifier-k8s champ --kind Secret --nom vitrine-secret --chemin 'data.MAILJET_API_KEY' --decode-base64 --texte cle-factice-a-remplacer secret.yaml"
    solution:
      - kubectl create secret generic vitrine-secret --from-literal=MAILJET_API_KEY=cle-factice-a-remplacer --dry-run=client -o yaml > secret.yaml
  - texte: 'Prouve que base64 n''est pas du chiffrement : extrais la valeur de `MAILJET_API_KEY` dans `secret.yaml`, décode-la avec `base64 -d` et écris le résultat dans `decode.txt` (par exemple `grep MAILJET_API_KEY secret.yaml | awk ''{print $2}'' | base64 -d > decode.txt`)'
    indice: '`awk ''{print $2}''` garde le deuxième mot de la ligne (la valeur encodée) ; `base64 -d` la décode. Ouvre `decode.txt` avec `cat`.'
    apres: [2]
    verif:
      - commande-reussit: "verifier-k8s champ --kind Secret --nom vitrine-secret --chemin 'data.MAILJET_API_KEY' --decode-base64 --fichier decode.txt secret.yaml"
    solution:
      - "grep MAILJET_API_KEY secret.yaml | awk '{print $2}' | base64 -d > decode.txt"
  - texte: 'Écris à la main (avec `nano pvc.yaml`) le PersistentVolumeClaim `vitrine-mediafiles-pvc` : 5 Gio (`5Gi`) en `ReadWriteMany`, comme dans la leçon'
    indice: 'Les clés sont `apiVersion: v1`, `kind: PersistentVolumeClaim`, `metadata.name`, puis `spec.accessModes` (une liste) et `spec.resources.requests.storage`. Valide avec `kubeconform pvc.yaml`.'
    verif:
      - commande-reussit: 'kubeconform pvc.yaml'
      - commande-reussit: "verifier-k8s champ --kind PersistentVolumeClaim --nom vitrine-mediafiles-pvc --chemin 'spec.accessModes' --egal '[ReadWriteMany]' pvc.yaml"
      - commande-reussit: "verifier-k8s champ --kind PersistentVolumeClaim --nom vitrine-mediafiles-pvc --chemin 'spec.resources.requests.storage' --texte 5Gi pvc.yaml"
    solution:
      - ecrire:
          pvc.yaml: |
            apiVersion: v1
            kind: PersistentVolumeClaim
            metadata:
              name: vitrine-mediafiles-pvc
            spec:
              accessModes:
                - ReadWriteMany
              resources:
                requests:
                  storage: 5Gi
  - texte: 'Dans `deployment.yaml`, monte ce volume : déclare-le dans `volumes` (nom `media-volume`, relié au PVC par `claimName`), puis ajoute au conteneur un `volumeMounts` avec `mountPath: /app/mediafiles`. Lance enfin `verifier-k8s references deployment.yaml configmap.yaml secret.yaml pvc.yaml`'
    indice: '`volumes` est une liste placée sous `spec` du pod (au même niveau que `containers`) ; `volumeMounts` est sous le conteneur. Le `name` du montage doit reprendre celui du volume.'
    apres: [1, 2, 4]
    verif:
      - commande-reussit: 'kubeconform deployment.yaml configmap.yaml secret.yaml pvc.yaml'
      - commande-reussit: "verifier-k8s champ --kind Deployment --nom vitrine-web --chemin 'spec.template.spec.volumes[name=media-volume].persistentVolumeClaim.claimName' --texte vitrine-mediafiles-pvc deployment.yaml"
      - commande-reussit: "verifier-k8s champ --kind Deployment --nom vitrine-web --chemin 'spec.template.spec.containers[0].volumeMounts[name=media-volume].mountPath' --texte /app/mediafiles deployment.yaml"
      - commande-reussit: 'verifier-k8s references deployment.yaml configmap.yaml secret.yaml pvc.yaml'
    solution:
      - ecrire:
          deployment.yaml: |
            apiVersion: apps/v1
            kind: Deployment
            metadata:
              name: vitrine-web
            spec:
              replicas: 1
              selector:
                matchLabels:
                  app: vitrine-web
              template:
                metadata:
                  labels:
                    app: vitrine-web
                spec:
                  volumes:
                    - name: media-volume
                      persistentVolumeClaim:
                        claimName: vitrine-mediafiles-pvc
                  containers:
                    - name: web
                      image: nginx:1.27
                      envFrom:
                        - configMapRef:
                            name: vitrine-configmap
                        - secretRef:
                            name: vitrine-secret
                      volumeMounts:
                        - name: media-volume
                          mountPath: /app/mediafiles
:::

## Vérifie tes acquis

:::quiz
Quelle donnée va dans un Secret plutôt que dans un ConfigMap ?

- [ ] L'URL publique du site
- [ ] Le nom de la base de données
- [x] Une clé d'API ou un mot de passe
- [ ] Le niveau de journalisation

> Les valeurs sensibles vont dans un Secret, qui peut être protégé par des droits d'accès plus stricts.
:::

:::quiz
Un Secret Kubernetes est stocké en base64. Que faut-il en conclure ?

- [ ] Il est chiffré et peut être commité dans Git
- [ ] Il n'est lisible que par le pod qui l'utilise
- [ ] Il devient illisible une fois monté en fichier
- [x] Il est simplement encodé : il ne faut pas le publier dans un dépôt

> Le base64 se décode en une commande. La protection vient des droits d'accès et de bonnes pratiques de stockage.
:::

:::quiz
Tu modifies un ConfigMap utilisé via `envFrom`. Que doit-on faire pour que l'application voie la nouvelle valeur ?

- [ ] Rien : les variables se mettent à jour seules
- [ ] Supprimer le service
- [x] Relancer les pods, par exemple avec `kubectl rollout restart`
- [ ] Recréer le PersistentVolumeClaim

> Les variables d'environnement sont figées au démarrage du conteneur.
:::

:::quiz
Que devient le contenu d'un volume `emptyDir` quand le pod est recréé ?

- [ ] Il est conservé dans le PVC voisin
- [x] Il est perdu
- [ ] Il est sauvegardé vers S3
- [ ] Il est copié sur l'autre pod

> `emptyDir` vit et meurt avec le pod : adapté à du temporaire, pas aux médias.
:::
