---
id: variables-secrets
title: "Variables et secrets"
summary: "Où ranger un mot de passe de CI, et pourquoi jamais dans le dépôt."
minutes: 35
objectives:
  - Distinguer variable prédéfinie, variable du fichier et variable de projet
  - Protéger et masquer un secret
  - Lire un script qui obtient un jeton d'accès sans rien écrire en clair
  - Repérer un secret écrit en clair ou affiché dans un log
---

Ton job doit appeler une API avec un identifiant et un secret. Le réflexe de débutant·e : l'écrire dans le `.gitlab-ci.yml`. C'est exactement ce qu'il ne faut pas faire, car tout ce qui est commité reste dans l'historique Git.

## Les mots à connaître

- Un **secret** est une information qui donne un accès : mot de passe, clé d'API, jeton.
- Un **jeton** (en anglais *token*) est un long texte, généré par un service, que tu présentes à la place de ton mot de passe pour prouver que tu as le droit d'accéder à une ressource. Il est souvent temporaire, et il vaut un mot de passe : celui qui le détient a l'accès.
- Une **variable CI/CD** est un nom associé à une valeur que GitLab met à disposition des commandes d'un job. Dans une commande, `$NOM` désigne la valeur de la variable `NOM`.
- **Keycloak** est le serveur d'identité de l'équipe : c'est lui qui gère les comptes et délivre des jetons aux applications.

## Trois sources de variables

| Source | Exemple | Visible dans le dépôt ? |
| --- | --- | :---: |
| Prédéfinie par GitLab | `CI_COMMIT_BRANCH`, `CI_REGISTRY_IMAGE` | Non, fournie à l'exécution |
| Écrite dans le `.gitlab-ci.yml` | `DOCKER_DRIVER: overlay2` | **Oui** |
| Variable de projet (*Settings > CI/CD > Variables*) | `KEYCLOAK_CLIENT_SECRET` | Non |

Règle simple : ce qui est public et ne change pas va dans le fichier ; **tout ce qui est secret va dans les variables du projet**.

## Un exemple réel : le job planifié d'Adhésion

L'API d'Adhésion a un job qui obtient un jeton auprès de Keycloak, puis appelle une URL de l'API. Aucune valeur sensible n'est dans le fichier :

```yaml
traiter_file_cron:
  stage: maintenance
  image: curlimages/curl:8.8.0
  rules:
    - if: '$CI_PIPELINE_SOURCE == "schedule"'
    - when: never
  script:
    - |
      ACCESS_TOKEN=$(
        curl --silent --show-error --fail \
          --request POST \
          --user "$KEYCLOAK_CLIENT_ID:$KEYCLOAK_CLIENT_SECRET" \
          --data "grant_type=client_credentials" \
          "$KEYCLOAK_BASE_URL/realms/$KEYCLOAK_REALM/protocol/openid-connect/token" \
        | sed -n 's/.*"access_token":"\([^"]*\)".*/\1/p'
      )
    - test -n "$ACCESS_TOKEN"
```

### Le début du job

- `traiter_file_cron:` est le nom du job.
- `stage: maintenance` le range dans le **stage** (l'étape) `maintenance`. Les stages sont déclarés dans la liste `stages` du fichier ; ils s'exécutent dans l'ordre (leçon 2).
- `image: curlimages/curl:8.8.0` est l'image Docker dans laquelle le job tourne : une image minuscule qui contient `curl`, l'outil en ligne de commande qui envoie des requêtes HTTP.

### Les `rules`

- La première règle, `if: '$CI_PIPELINE_SOURCE == "schedule"'`, est vraie quand le pipeline est un pipeline **planifié** (lancé à heure fixe, comme un cron). Elle n'a pas de `when` : le job est donc créé normalement.
- La seconde, `- when: never`, n'a **aucune condition** : elle correspond à tout ce que la première a laissé passer. `when: never` signifie « ne jamais créer le job ». Résultat : le job n'existe que dans un pipeline planifié (menu *Build > Pipeline schedules*), jamais sur un push ou une merge request.

### Le script, pièce par pièce

Le `- |` ouvre un **bloc de texte sur plusieurs lignes** (en YAML, `|` garde les retours à la ligne). Le shell reçoit donc une seule grande commande :

- `ACCESS_TOKEN=$( … )` lance la commande entre parenthèses et range sa sortie dans la variable `ACCESS_TOKEN`.
- `curl` contacte le serveur Keycloak. Les `\` en fin de ligne disent « la commande continue à la ligne suivante » : ils servent uniquement à la lisibilité.
- `--silent` supprime la barre de progression ; `--show-error` affiche quand même un message si ça échoue.
- `--fail` fait échouer `curl` (code de sortie différent de 0) si le serveur répond par une erreur HTTP (401, 500…). Sans lui, `curl` réussirait même avec une page d'erreur.
- `--request POST` choisit la méthode HTTP : `POST` envoie des données au serveur, alors que `GET` (par défaut) ne fait que lire.
- `--user "$KEYCLOAK_CLIENT_ID:$KEYCLOAK_CLIENT_SECRET"` envoie un identifiant et un secret séparés par `:`. C'est l'authentification HTTP « basique » : l'application s'identifie, comme une personne le ferait avec son mot de passe. Ces deux valeurs viennent des variables du projet.
- `--data "grant_type=client_credentials"` envoie le contenu de la requête. `grant_type=client_credentials` est la formule du protocole OAuth 2 qui veut dire « je suis une application (et non une personne) : voici mon identifiant et mon secret, donne-moi un jeton d'accès ».
- L'adresse finale est l'URL du point d'accès de Keycloak qui délivre les jetons, construite avec deux variables (l'adresse du serveur et le *realm*, l'espace de comptes).

Keycloak répond par un texte au format JSON, par exemple `{"access_token":"abc123","expires_in":300}`. Le tube `|` envoie cette réponse à la commande suivante, `sed`, qui n'en garde que le jeton :

- `sed` transforme du texte ligne par ligne ; `-n` lui dit de n'afficher que ce qu'on lui demande explicitement.
- `'s/.*"access_token":"\([^"]*\)".*/\1/p'` est une expression « remplacer » (`s/ce qu'on cherche/par quoi remplacer/`). Chercher : `.*` (n'importe quels caractères), puis le texte littéral `"access_token":"`, puis `\( … \)`, une **parenthèse de capture** qui mémorise ce qu'elle contient, ici `[^"]*` (n'importe quels caractères sauf un guillemet, donc la valeur du jeton jusqu'au guillemet fermant), puis `"` et `.*` pour le reste de la ligne. Remplacer par `\1`, le contenu de la première parenthèse de capture : il ne reste que le jeton. Le `p` final demande d'afficher le résultat.

Enfin, `test -n "$ACCESS_TOKEN"` fait échouer le job si le jeton est vide. Mieux vaut un job rouge qu'un job vert qui n'a rien fait.

Ce qu'il faut retenir : `KEYCLOAK_CLIENT_ID`, `KEYCLOAK_CLIENT_SECRET`, `KEYCLOAK_BASE_URL` et `KEYCLOAK_REALM` sont définies dans les paramètres du projet, pas dans le fichier.

## Protéger et masquer

Au moment de créer une variable de projet, deux options comptent :

- **Masquée** (*Masked*) : GitLab remplace sa valeur par `[MASKED]` dans les logs du job (le **log** est le journal affiché pour chaque job). Pour qu'il puisse le faire, la valeur doit être sur une seule ligne, faire au moins 8 caractères et n'utiliser que des caractères simples (lettres, chiffres et quelques symboles comme `@ : . ~ + / =`) ; GitLab refuse de masquer une valeur avec des espaces.
- **Protégée** (*Protected*) : elle n'est fournie qu'aux pipelines des branches et tags protégés. Une **branche protégée** est une branche (par exemple `main`) que le projet a verrouillée : seules certaines personnes peuvent y pousser ou y fusionner, et on ne peut pas y réécrire l'historique. Un **tag protégé** est pareil pour un tag, l'étiquette posée sur un commit pour marquer une version (`v1.2.0`). Une variable protégée n'est donc pas donnée à une branche de travail quelconque, ce qui limite les dégâts si quelqu'un y glisse une commande pour afficher le secret.

:::danger Un secret commité est un secret perdu
Si un mot de passe ou un jeton apparaît dans un commit, même supprimé juste après, considère-le comme compromis : **révoque-le et remplace-le**, puis préviens l'équipe Infra. Réécrire l'historique ne suffit pas, il a pu être cloné entre-temps.
:::

:::tip Ne fais pas `echo` d'une variable secrète
Même masquée, une valeur peut fuiter transformée : le **base64** est un encodage qui transforme un texte en une autre suite de caractères (`abc` devient `YWJj`) ; une valeur affichée sous cette forme n'est plus reconnue par le masquage. Évite d'afficher des secrets dans les logs, y compris pour déboguer.
:::

## Entraîne-toi

:::info Ce que verifier-ci sait faire (et ne sait pas)
Dans ce labo, `verifier-ci` lit ton `.gitlab-ci.yml` et signale deux types de problèmes : une variable au nom évocateur (`…TOKEN`, `…PASSWORD`, `…SECRET`) dont la valeur est écrite **en clair** dans le fichier (c'est une erreur), et une commande qui **affiche** un secret dans le log (c'est un avertissement). Il n'a bien sûr aucun accès aux variables de ton vrai projet GitLab, et il ne remplace pas la relecture ni l'outil Secret Detection que tu verras à la leçon suivante. Toutes les valeurs de ce labo sont fausses.
:::

:::lab
engine: real
intro: |
  Ton dossier de travail contient un `.gitlab-ci.yml` à trois jobs (`build`, `deploy`, `notify`) et un fichier `valeurs.txt`. Un secret (faux) est écrit en clair, et un des jobs le laisse fuiter. Modifie les fichiers avec `nano` (Ctrl+O puis Entrée pour enregistrer, Ctrl+X pour quitter) ou l'éditeur de VS Code. Commence par `verifier-ci .gitlab-ci.yml` pour lire ce que l'outil signale.
commands:
  - cp -R /opt/exercices/04-secrets/. .
steps:
  - text: 'Le jeton `DEPLOY_TOKEN` est écrit en clair dans la section `variables:` du fichier. Retire cette ligne (sa vraie valeur irait dans *Settings > CI/CD > Variables*) : le job `deploy` continue d''utiliser `$DEPLOY_TOKEN`, GitLab la fournira. `verifier-ci .gitlab-ci.yml` ne doit plus signaler d''erreur'
    hint: 'Supprime la ligne `DEPLOY_TOKEN: "FAUX-…"` et garde `DOCKER_DRIVER`. Ne supprime pas `$DEPLOY_TOKEN` dans le script du job `deploy`.'
    checks:
      - command-succeeds: verifier-ci .gitlab-ci.yml
      - command-fails: grep -q 'FAUX-jeton' .gitlab-ci.yml
      - command-succeeds: "verifier-ci .gitlab-ci.yml --sans-variable-globale DEPLOY_TOKEN"
      - command-succeeds: "verifier-ci .gitlab-ci.yml --job deploy --cree --script-lance 'curl :: $DEPLOY_TOKEN'"
    solution:
      - write:
          .gitlab-ci.yml: |
            stages:
              - build
              - deploy
              - notify

            variables:
              DOCKER_DRIVER: overlay2

            build:
              stage: build
              image: alpine:3.20
              script:
                - echo "Construction de l'application"

            deploy:
              stage: deploy
              image: curlimages/curl:8.8.0
              script:
                - 'curl --fail --header "PRIVATE-TOKEN: $DEPLOY_TOKEN" "$DEPLOY_URL"'

            notify:
              stage: notify
              image: alpine:3.20
              script:
                - echo "Déploiement terminé"
                - echo "Jeton utilisé $DEPLOY_TOKEN"
  - text: 'Un des trois jobs affiche le secret dans son log. Repère-le avec `verifier-ci .gitlab-ci.yml` (cherche la ligne `AVERTISSEMENT`) et écris son nom dans un fichier `fuite.txt`'
    hint: 'Le message cite le nom du job entre guillemets : `job « … »`. Écris-le avec `echo NOM > fuite.txt`.'
    checks:
      - env-file-contains: [fuite.txt, '^\s*notify\s*$']
    solution:
      - echo notify > fuite.txt
  - text: 'Colmate la fuite : remplace la commande fautive par un test qui vérifie seulement que le secret est présent, `test -n "$DEPLOY_TOKEN"`. `verifier-ci --strict .gitlab-ci.yml` doit réussir'
    hint: 'Dans le job `notify`, la ligne `echo "Jeton utilisé $DEPLOY_TOKEN"` devient `- test -n "$DEPLOY_TOKEN"`.'
    checks:
      - command-succeeds: verifier-ci --strict .gitlab-ci.yml
      - command-succeeds: "verifier-ci .gitlab-ci.yml --job notify --cree --script-lance 'test :: -n :: $DEPLOY_TOKEN'"
    solution:
      - write:
          .gitlab-ci.yml: |
            stages:
              - build
              - deploy
              - notify

            variables:
              DOCKER_DRIVER: overlay2

            build:
              stage: build
              image: alpine:3.20
              script:
                - echo "Construction de l'application"

            deploy:
              stage: deploy
              image: curlimages/curl:8.8.0
              script:
                - 'curl --fail --header "PRIVATE-TOKEN: $DEPLOY_TOKEN" "$DEPLOY_URL"'

            notify:
              stage: notify
              image: alpine:3.20
              script:
                - echo "Déploiement terminé"
                - test -n "$DEPLOY_TOKEN"
  - text: 'GitLab ne masque pas n''importe quelle valeur. `valeurs.txt` contient quatre valeurs d''exemple, une par ligne. Teste chacune avec `verifier-ci --masquable ''valeur''` et recopie dans `masquables.txt` celles que GitLab accepterait de masquer, une par ligne'
    hint: 'Une valeur est refusée si elle fait moins de 8 caractères, si elle contient une espace ou un caractère inhabituel. Mets la valeur entre apostrophes pour que le shell ne la découpe pas.'
    checks:
      - command-succeeds: |
          test -s masquables.txt && test "$(sort -u masquables.txt | wc -l)" -eq 2 && while IFS= read -r v; do verifier-ci --masquable "$v" > /dev/null && grep -qxF -- "$v" valeurs.txt || exit 1; done < masquables.txt
    solution:
      - printf '%s\n' 'Zm9vYmFyMTIzNDU=' 'Tk3fQ9aL.p2X@77v' > masquables.txt
:::

## Vérifie tes acquis

:::quiz
Où ranger le secret d'un client Keycloak utilisé par un job ?

- [ ] Dans la section `variables:` du `.gitlab-ci.yml`
- [ ] Dans un fichier `secrets.txt` commité avec le code
- [x] Dans les variables CI/CD du projet, masquées

> Le fichier est versionné, donc lisible par toute personne qui a accès au dépôt.
:::

:::quiz
Qu'est-ce qu'une variable « protégée » ?

- [ ] Une variable dont la valeur est chiffrée dans les logs
- [x] Une variable disponible uniquement dans les pipelines des branches et tags protégés
- [ ] Une variable que personne ne peut modifier

> Protégée limite l'exposition ; masquée cache la valeur dans les logs. Ce sont deux options distinctes.
:::

:::quiz
Tu viens de commiter un jeton d'API par erreur, puis tu l'as supprimé dans le commit suivant. Que fais-tu ?

- [ ] Rien : il n'est plus dans la dernière version du code
- [x] Tu le révoques, tu en génères un nouveau et tu préviens l'équipe Infra
- [ ] Tu attends qu'il expire de lui-même

> L'historique Git conserve l'ancien commit : le secret doit être considéré comme compromis.
:::

:::quiz
Dans le job d'Adhésion, que fait le morceau `| sed -n 's/.*"access_token":"\([^"]*\)".*/\1/p'` ?

- [ ] Il chiffre la réponse de Keycloak avant de l'envoyer
- [ ] Il supprime le jeton de la réponse pour qu'il ne s'affiche pas
- [x] Il extrait de la réponse JSON la valeur du champ `access_token`

> `sed` repère `"access_token":"`, mémorise ce qui suit jusqu'au guillemet suivant et n'affiche que cette valeur.
:::
