## Structure d'un pipeline

| Mot-clé | Rôle |
| --- | --- |
| `stages` | Ordre des étapes ; jobs d'un même stage en parallèle |
| `image` | Image Docker dans laquelle le job s'exécute |
| `before_script` / `script` | Commandes lancées dans l'ordre ; un code ≠ 0 fait échouer le job |
| `rules` | Conditions de création du job ; la première règle qui correspond gagne |
| `include` / `extends` | Réutiliser des fichiers, des modèles ou un autre job |
| `services` | Conteneurs voisins du job (`docker:dind`) |
| `allow_failure` | Le job peut échouer sans bloquer le pipeline |

## Variables utiles

`CI_COMMIT_BRANCH` · `CI_COMMIT_REF_SLUG` · `CI_PIPELINE_SOURCE` (`merge_request_event`, `schedule`…) · `CI_REGISTRY` · `CI_REGISTRY_IMAGE` · `CI_REGISTRY_USER` · `CI_REGISTRY_PASSWORD`

## Build et push d'une image

```yaml
before_script:
  - docker login -u "$CI_REGISTRY_USER" -p "$CI_REGISTRY_PASSWORD" $CI_REGISTRY
script:
  - docker build -t "$CI_REGISTRY_IMAGE:$CI_COMMIT_REF_SLUG" .
  - docker push "$CI_REGISTRY_IMAGE:$CI_COMMIT_REF_SLUG"
services:
  - docker:dind
```

## Analyses de sécurité

```yaml
include:
  - template: Jobs/SAST.gitlab-ci.yml
  - template: Jobs/Code-Quality.gitlab-ci.yml
  - template: Jobs/Dependency-Scanning.gitlab-ci.yml
  - template: Jobs/Secret-Detection.gitlab-ci.yml
```

Exclusions : `SAST_EXCLUDED_PATHS` et `DS_EXCLUDED_PATHS`.

## Secrets

Jamais dans le dépôt : variables CI/CD du projet, **masquées** (pas dans les logs) et **protégées** (branches protégées seulement). Secret commité = secret à révoquer.

## Quand ça rougit

1. Premier job rouge · 2. première erreur du log · 3. cause fréquente (YAML, `rules`, image, variable) · 4. une correction à la fois.

Code 127 = commande introuvable (vérifie l'`image`).

## Les outils du labo (hors ligne, ce ne sont pas des runners)

| Commande | Rôle |
| --- | --- |
| `verifier-ci fichier.yml` | Diagnostic de structure : stages, `script`, `rules`, `extends`, `include`, variables |
| `verifier-ci --montrer fichier.yml` | Affiche stages et jobs reconstitués |
| `verifier-ci --contexte NOM=VALEUR fichier.yml` | Simule les `rules` : quels jobs seraient créés |
| `verifier-renovate renovate.json` | Diagnostic d'un `renovate.json` |
