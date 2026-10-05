---
id: charts-helm
title: "Écrire et utiliser un chart Helm"
summary: "Lire le chart de Vitrine, comprendre values.yaml et les templates, puis installer, mettre à jour et annuler une release."
minutes: 55
objectives:
  - Décrire le rôle de Chart.yaml, values.yaml et du dossier templates
  - Surcharger une valeur avec --set ou -f et visualiser le résultat avec helm template
  - Distinguer un chart, une release et une révision
---

## À quoi ça sert, et pourquoi ?

Dans les leçons précédentes, tu as vu cinq objets (Deployment, Service, ConfigMap, Secret, Ingress), chacun dans un fichier YAML. Pour déployer le même site en test et en production, il faudrait copier ces fichiers et changer à la main les noms, les adresses et les tailles : on se trompe vite, et les copies divergent.

**Helm** règle ce problème. C'est un **gestionnaire de paquets pour Kubernetes**, comme `apt` pour Debian ou `pip` pour Python. Il fonctionne comme un **publipostage** : tu écris la lettre une fois, avec des trous (« Cher `NOM` »), et tu fournis un tableau de valeurs différent pour chaque destinataire.

- Un **chart** est le paquet : un dossier de modèles YAML et de valeurs par défaut.
- Une **release** est une installation d'un chart sous un nom donné (`helm install ma-release …`).
- Chaque `helm upgrade` crée une nouvelle **révision** de la release, ce qui permet de revenir en arrière.

## Anatomie d'un chart

Le dossier `helm/` de Vitrine (et `helmchart/adhesion/` pour Adhésion) suit la structure standard :

```text
helm/
├── Chart.yaml        ← identité du chart, version, dépendances
├── values.yaml       ← valeurs par défaut, modifiables par la personne qui déploie
└── templates/        ← les modèles YAML (Deployment, Service, Ingress…)
    ├── _helpers.tpl  ← morceaux réutilisables (noms, labels)
    └── NOTES.txt     ← message affiché après l'installation
```

Un **modèle** (*template*) est un fichier YAML avec des trous. Un **fichier de valeurs** donne ce qu'on met dans les trous. Dans `Chart.yaml`, `apiVersion: v2` indique un chart pour **Helm 3**. Le chart de Vitrine déclare aussi des **dépendances** (une base PostgreSQL, deux charts de sauvegarde), chacune activée par une condition comme `postgresql.enabled`.

## Un mini chart inspiré de Vitrine

Pour comprendre sans te noyer, voici un chart réduit, `mini-portail`. Il est fourni dans l'atelier de cette leçon, où il a été contrôlé avec `helm lint` et `helm template` (les deux commandes tournent sans cluster). Il contient cinq fichiers : `Chart.yaml`, `values.yaml` et trois modèles (Deployment, Service, Ingress) plus un ConfigMap. Commençons par `Chart.yaml`, la fiche d'identité :

```yaml
apiVersion: v2
name: mini-portail
description: Mini chart de formation inspiré du chart de Vitrine
type: application
version: 0.1.0
appVersion: "1.0"
```

- `apiVersion: v2` : le format du fichier, celui de Helm 3 ;
- `name` : le nom du chart (le dossier porte le même) ;
- `description` : une phrase pour les humains ;
- `type: application` : un chart qui déploie une application (l'autre type, `library`, ne contient que des morceaux réutilisables) ;
- `version` : la version du **chart** lui-même ; on l'augmente à chaque modification du chart ;
- `appVersion` : la version de l'**application** qu'il déploie, à titre d'information.

`values.yaml` regroupe tout ce qui peut changer d'un environnement à l'autre :

```yaml
image:
  repository: registry.example.org/equipe/vitrine
  tag: master
  pullPolicy: IfNotPresent

replicas: 1

ingress:
  enabled: true
  host: portail.172.17.0.1.nip.io

env:
  siteUrl: http://portail.172.17.0.1.nip.io
  appDebug: false
```

- `image.repository` et `image.tag` : le nom de l'image et son étiquette, séparés pour pouvoir changer seulement la version ;
- `image.pullPolicy: IfNotPresent` : Kubernetes ne télécharge l'image que si le nœud ne l'a pas déjà ;
- `replicas` : le nombre de copies voulu ;
- `ingress.enabled` : un **interrupteur** (`true` ou `false`) qui décide si l'Ingress est fabriqué ; `ingress.host` : le nom de domaine, ici un nom `nip.io` de développement ;
- `env.siteUrl` et `env.appDebug` : les réglages de l'application (adresse publique, mode de mise au point) ; le modèle `configmap.yaml` ci-dessous les range dans un ConfigMap.

Voici maintenant les modèles, dans `templates/`. D'abord `deployment.yaml`, le Deployment de la leçon 1 dans lequel les parties variables sont remplacées par des trous, avec en plus un `envFrom` qui lit le ConfigMap (leçon 2) :

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: "{{ .Release.Name }}-web"
spec:
  replicas: {{ .Values.replicas }}
  selector:
    matchLabels:
      app: "{{ .Release.Name }}-web"
  template:
    metadata:
      labels:
        app: "{{ .Release.Name }}-web"
    spec:
      containers:
        - name: web
          image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
          imagePullPolicy: {{ .Values.image.pullPolicy | quote }}
          ports:
            - name: django
              containerPort: 8000
          envFrom:
            - configMapRef:
                name: "{{ .Release.Name }}-config"
```

Les doubles accolades `{{ … }}` sont les « trous » : Helm les remplace au moment du **rendu** (la fabrication du YAML final) :

- `.Release.Name` : le nom donné à l'installation. Il est mis devant chaque nom d'objet pour que deux installations du même chart ne se marchent pas dessus ;
- `.Values.…` : une valeur de `values.yaml` (ou celle qui la remplace) ; `.Values.replicas` donne `1`, `.Values.image.tag` donne `master` ;
- `| quote` : le « tuyau » `|` passe la valeur à la fonction `quote`, qui l'entoure de guillemets ;
- les lignes sans accolades (`apiVersion`, `kind`, `selector`…) sont recopiées telles quelles ;
- `envFrom` / `configMapRef` : comme en leçon 2, le conteneur lit toutes les clés du ConfigMap nommé `<release>-config`.

Le modèle `templates/service.yaml` est le Service de la leçon 1 : `name: "{{ .Release.Name }}-svc"`, un `selector` `app: "{{ .Release.Name }}-web"` et le port 80 qui vise le port nommé `django`. Le modèle `templates/configmap.yaml` range les deux réglages de `env` :

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: "{{ .Release.Name }}-config"
data:
  SITE_URL: {{ .Values.env.siteUrl | quote }}
  APP_DEBUG: {{ .Values.env.appDebug | quote }}
```

`| quote` est ici indispensable : `appDebug: false` est un booléen pour YAML, alors qu'un ConfigMap n'accepte que des **chaînes** (leçon 2). Enfin, `templates/ingress.yaml` est le modèle **conditionnel** :

```yaml
{{- if .Values.ingress.enabled }}
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: "{{ .Release.Name }}-ingress"
spec:
  rules:
    - host: {{ .Values.ingress.host | quote }}
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: "{{ .Release.Name }}-svc"
                port:
                  number: 80
{{- end }}
```

- `{{- if .Values.ingress.enabled }}` … `{{- end }}` : tout ce qui est entre les deux n'est généré **que si** `ingress.enabled` est vrai. Le tiret de `{{-` supprime l'espace ou le saut de ligne qui le précède, pour ne pas laisser de ligne vide dans le résultat. C'est ainsi que les charts de l'équipe activent un Ingress, l'autoscaling ou les sauvegardes ;
- le reste est l'Ingress de la leçon 3, avec le nom de domaine pris dans `ingress.host` et le service `<release>-svc` comme destination.

## Voir avant de déployer : helm template

`helm template` génère le YAML final **sans toucher au cluster**. C'est le meilleur réflexe de relecture ; `helm lint` vérifie en plus que le chart est bien formé :

```bash
helm lint ./mini-portail
helm template demo ./mini-portail
```

`helm lint ./mini-portail` contrôle le dossier `./mini-portail` ; `helm template demo ./mini-portail` fabrique le YAML pour une release appelée `demo` (le premier argument est le nom de la release, le second le dossier du chart). Pour la release `demo`, le déploiement rendu contient :

```console
apiVersion: apps/v1
kind: Deployment
metadata:
  name: "demo-web"
spec:
  replicas: 1
  ...
        - name: web
          image: "registry.example.org/equipe/vitrine:master"
```

## Surcharger les valeurs

Ne modifie pas `values.yaml` pour un environnement : surcharge-le.

```bash
helm template demo ./mini-portail --set replicas=2
helm template demo ./mini-portail --set ingress.enabled=false
helm template demo ./mini-portail -f ma-config.yaml
```

Une valeur imbriquée se désigne par un chemin avec des points : `ingress.enabled` est la clé `enabled` rangée sous `ingress` dans `values.yaml`.

- `--set clé=valeur` : pour une valeur isolée. Avec `replicas=2`, le Deployment rendu contient `replicas: 2` ; avec `ingress.enabled=false`, la condition de `ingress.yaml` est fausse et **aucun objet Ingress** n'apparaît dans le rendu ;
- `-f fichier.yaml` : un fichier de valeurs, avec seulement ce qui change ; il est fusionné avec `values.yaml`. De même, `--set env.siteUrl=http://autre.exemple.invalid` change l'adresse écrite dans `SITE_URL` du ConfigMap.

:::warning Une clé mal orthographiée est ignorée sans erreur
Si tu écris `replica: 2` au lieu de `replicas: 2` dans ton fichier de valeurs, Helm ne dit rien : la clé `replica` n'est lue par aucun modèle, et le rendu garde `replicas: 1`. Relis toujours le résultat de `helm template` : c'est ce que tu feras dans l'atelier.
:::

## Cycle de vie d'une release

D'après le README du chart de Vitrine, on clone le dépôt et on installe depuis le dossier (l'équipe n'a pas de dépôt de charts, c'est-à-dire de serveur où publier ses charts pour que d'autres les téléchargent) :

```bash
helm install ma-release helm/                   # première installation
helm upgrade ma-release helm/ -f prod.yaml      # mise à jour : nouvelle révision
helm history ma-release                         # liste des révisions
helm rollback ma-release 1                      # retour à la révision 1
helm uninstall ma-release                       # supprime la release
```

- `install ma-release helm/` : « installe le chart du dossier `helm/` sous le nom `ma-release` » ;
- `upgrade … -f prod.yaml` : réapplique le chart avec, en plus, les valeurs du fichier `prod.yaml` ;
- `history` : affiche les révisions numérotées ; `rollback … 1` rétablit l'état de la révision 1 (et crée une nouvelle révision).

:::warning Les pièges des charts réels
- Dans les `values.yaml` de Vitrine, la clé de la base est écrite `postrgresql` (avec une faute) : le modèle utilise la même orthographe, qui fonctionne donc, mais une surcharge `postgresql.postgresql.password` serait **silencieusement ignorée**. Vérifie toujours le rendu avec `helm template`.
- Le Secret de Vitrine génère sa clé Django avec `randAlphaNum 50` : une **nouvelle** valeur est tirée à chaque `helm upgrade`. Tu peux donc invalider des sessions sans le vouloir.
- Les valeurs sensibles sont à `nil` par défaut : ne les écris pas dans un fichier commité.
:::

:::info À confirmer avec l'équipe Infra
Quelle version de Helm utilise l'équipe Infra, et d'où sont lancés les déploiements (poste d'un membre, CI, Terraform) ? Les charts de Vitrine, d'Adhésion et d'OpenEvent sont tous trois installés depuis un dossier du dépôt ; le dépôt du pôle utilise en plus Terraform (`helm_release`).
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Dans ton dossier de travail, le chart `mini-portail` de la leçon est presque complet : il manque son Ingress et son ConfigMap, et le fichier `values-prod.yaml` contient une faute. **Il n'y a pas de cluster dans cet atelier** : tu utilises `helm lint`, `helm template` et `helm package` (qui travaillent hors ligne), jamais `helm install`. Tout ce que tu valides est donc le **YAML produit**, pas son fonctionnement sur un vrai cluster.
commands:
  - cp -R /opt/exercices/04-helm/. .
steps:
  - text: 'Rends le chart pour une release appelée `demo` et garde le résultat dans un fichier : `helm template demo ./mini-portail > rendu.yaml`, puis lis-le avec `cat rendu.yaml`'
    hint: 'Tu dois y retrouver un Service et un Deployment dont les noms commencent par `demo-`. Il n''y a pas encore d''Ingress.'
    checks:
      - command-succeeds: "verifier-k8s champ --kind Deployment --nom demo-web --existe rendu.yaml"
      - command-succeeds: "verifier-k8s champ --kind Service --nom demo-svc --existe rendu.yaml"
    solution:
      - helm template demo ./mini-portail > rendu.yaml
  - text: 'Le fichier `values-prod.yaml` veut 2 copies et l''étiquette `v1.4.2`, mais `helm template demo ./mini-portail -f values-prod.yaml` affiche encore `replicas: 1`. Trouve la faute de frappe dans `values-prod.yaml` et corrige-la'
    hint: 'Compare le nom de la clé avec celui de `mini-portail/values.yaml` : Helm ignore en silence une clé qu''aucun modèle ne lit.'
    checks:
      - command-succeeds: "verifier-k8s champ --chemin 'replicas' --egal 2 values-prod.yaml"
      - command-succeeds: "verifier-k8s champ --chemin 'image.tag' --texte v1.4.2 values-prod.yaml"
      - command-succeeds: "helm template demo ./mini-portail -f values-prod.yaml | verifier-k8s champ --kind Deployment --nom demo-web --chemin 'spec.replicas' --egal 2 -"
      - command-succeeds: "helm template demo ./mini-portail -f values-prod.yaml | verifier-k8s champ --kind Deployment --nom demo-web --chemin 'spec.template.spec.containers[0].image' --texte registry.example.org/equipe/vitrine:v1.4.2 -"
    solution:
      - sed -i 's/^replica:/replicas:/' values-prod.yaml
  - text: 'Crée le modèle `mini-portail/templates/ingress.yaml` de la leçon, avec sa condition `{{- if .Values.ingress.enabled }}` … `{{- end }}`. Contrôle les deux cas : `helm template demo ./mini-portail` produit un Ingress, `helm template demo ./mini-portail --set ingress.enabled=false` n''en produit aucun'
    hint: 'Recopie le modèle de la leçon avec `nano mini-portail/templates/ingress.yaml`. Le service visé est `"{{ .Release.Name }}-svc"`. Termine par `helm lint ./mini-portail`.'
    checks:
      - command-succeeds: "helm template demo ./mini-portail | verifier-k8s champ --kind Ingress --chemin 'spec.rules[0].host' --texte portail.172.17.0.1.nip.io -"
      - command-succeeds: "helm template demo ./mini-portail --set ingress.host=portail.exemple.invalid | verifier-k8s champ --kind Ingress --chemin 'spec.rules[0].host' --texte portail.exemple.invalid -"
      - command-succeeds: "helm template demo ./mini-portail | verifier-k8s champ --kind Ingress --chemin 'spec.rules[0].http.paths[0].backend.service.name' --texte demo-svc -"
      - command-succeeds: "helm template demo ./mini-portail --set ingress.enabled=false | verifier-k8s champ --kind Ingress --absent -"
      - command-succeeds: 'helm template demo ./mini-portail | kubeconform -'
    solution:
      - write:
          mini-portail/templates/ingress.yaml: |
            {{- if .Values.ingress.enabled }}
            apiVersion: networking.k8s.io/v1
            kind: Ingress
            metadata:
              name: "{{ .Release.Name }}-ingress"
            spec:
              rules:
                - host: {{ .Values.ingress.host | quote }}
                  http:
                    paths:
                      - path: /
                        pathType: Prefix
                        backend:
                          service:
                            name: "{{ .Release.Name }}-svc"
                            port:
                              number: 80
            {{- end }}
  - text: 'Utilise enfin les valeurs `env.siteUrl` et `env.appDebug`, aujourd''hui inutilisées : crée `mini-portail/templates/configmap.yaml` (ConfigMap `"{{ .Release.Name }}-config"` avec `SITE_URL` et `APP_DEBUG`, entourés de `| quote`) et ajoute à `templates/deployment.yaml` l''`envFrom` qui le lit'
    hint: 'Le modèle du ConfigMap est dans la leçon. Dans le déploiement, `envFrom` est au même niveau que `ports`, dans le conteneur. Contrôle avec `helm template demo ./mini-portail | verifier-k8s references -`.'
    after: [3]
    checks:
      - command-succeeds: "helm template demo ./mini-portail --set env.siteUrl=http://test.exemple.invalid | verifier-k8s champ --kind ConfigMap --nom demo-config --chemin 'data.SITE_URL' --texte http://test.exemple.invalid -"
      - command-succeeds: "helm template demo ./mini-portail --set env.appDebug=true | verifier-k8s champ --kind ConfigMap --nom demo-config --chemin 'data.APP_DEBUG' --texte true -"
      - command-succeeds: "helm template demo ./mini-portail | verifier-k8s champ --kind ConfigMap --nom demo-config --chemin 'data.SITE_URL' --texte http://portail.172.17.0.1.nip.io -"
      - command-succeeds: "helm template demo ./mini-portail | verifier-k8s champ --kind Deployment --nom demo-web --chemin 'spec.template.spec.containers[0].envFrom[0].configMapRef.name' --texte demo-config -"
      - command-succeeds: 'helm template demo ./mini-portail | verifier-k8s references -'
      - command-succeeds: 'helm template demo ./mini-portail | kubeconform -'
    solution:
      - write:
          mini-portail/templates/configmap.yaml: |
            apiVersion: v1
            kind: ConfigMap
            metadata:
              name: "{{ .Release.Name }}-config"
            data:
              SITE_URL: {{ .Values.env.siteUrl | quote }}
              APP_DEBUG: {{ .Values.env.appDebug | quote }}
      - write:
          mini-portail/templates/deployment.yaml: |
            apiVersion: apps/v1
            kind: Deployment
            metadata:
              name: "{{ .Release.Name }}-web"
            spec:
              replicas: {{ .Values.replicas }}
              selector:
                matchLabels:
                  app: "{{ .Release.Name }}-web"
              template:
                metadata:
                  labels:
                    app: "{{ .Release.Name }}-web"
                spec:
                  containers:
                    - name: web
                      image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
                      imagePullPolicy: {{ .Values.image.pullPolicy | quote }}
                      ports:
                        - name: django
                          containerPort: 8000
                      envFrom:
                        - configMapRef:
                            name: "{{ .Release.Name }}-config"
  - text: 'Passe la `version` de `mini-portail/Chart.yaml` à `0.2.0`, vérifie le chart avec `helm lint ./mini-portail`, puis fabrique le paquet avec `helm package ./mini-portail` : il crée `mini-portail-0.2.0.tgz`'
    hint: 'Le nom de l''archive vient de `name` et `version` dans `Chart.yaml`. C''est ce type de fichier qu''on publierait dans un dépôt de charts.'
    after: [4]
    checks:
      - command-succeeds: 'helm lint ./mini-portail'
      - env-file-exists: mini-portail-0.2.0.tgz
      - command-succeeds: "verifier-k8s champ --chemin 'version' --texte 0.2.0 mini-portail/Chart.yaml"
      - command-succeeds: "helm show chart mini-portail-0.2.0.tgz | verifier-k8s champ --chemin version --texte 0.2.0 -"
    solution:
      - "sed -i 's/^version: 0.1.0/version: 0.2.0/' mini-portail/Chart.yaml"
      - helm package ./mini-portail
:::

## Vérifie tes acquis

:::quiz
Quelle commande affiche le YAML produit par un chart sans rien créer dans le cluster ?

- [ ] `helm install --dry`
- [ ] `helm get manifest`
- [x] `helm template`
- [ ] `helm history`

> `helm template` rend les modèles localement, sans contacter le cluster.
:::

:::quiz
Quel est le rôle du fichier `values.yaml` ?

- [ ] Il liste les dépendances du chart
- [ ] Il contient le nom de la release
- [x] Il fournit les valeurs par défaut utilisées par les modèles
- [ ] Il remplace le Dockerfile

> La personne qui déploie surcharge ces valeurs avec `--set` ou `-f`, sans modifier le chart.
:::

:::quiz
Tu lances `helm upgrade` trois fois sur la même release. Combien de révisions apparaissent dans `helm history` ?

- [ ] Une seule
- [ ] Trois
- [x] Quatre, en comptant l'installation initiale
- [ ] Aucune, l'historique n'existe qu'avec Tiller

> Chaque installation ou mise à jour ajoute une révision, ce qui rend `helm rollback` possible.
:::

:::quiz
Que fait `{{- if .Values.ingress.enabled }}` en tête de `ingress.yaml` ?

- [ ] Il crée l'Ingress dans tous les cas
- [x] Il ne génère l'Ingress que si la valeur est vraie
- [ ] Il supprime l'Ingress existant
- [ ] Il interroge le cluster pour savoir si un contrôleur existe

> Cette condition est évaluée au rendu : le fichier est vide quand la valeur est fausse.
:::
