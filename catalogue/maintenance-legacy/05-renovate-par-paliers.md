---
id: renovate-par-paliers
titre: "Renovate : automatiser les mises à jour"
resume: "Régler Renovate pour qu'il avance par paliers sur un projet hérité, sans noyer l'équipe de merge requests."
duree: 30
objectifs:
  - Expliquer pourquoi des mises à jour régulières évitent un nouveau retard
  - Plafonner une dépendance avec `allowedVersions` pour avancer palier par palier
  - Grouper des mises à jour liées et lire le `renovate.json` d'un projet
---

Tu viens de passer trois semaines à monter un projet de Django 3.1 à 5.2. Dans deux ans, il faudra tout recommencer, sauf si les mises à jour deviennent un petit geste régulier plutôt qu'un chantier. C'est le rôle de Renovate.

## À quoi ça sert, et pourquoi après une montée ?

**Renovate** est un robot qui surveille les fichiers de dépendances d'un dépôt (`requirements.txt`, `package.json`, `Dockerfile`), repère les nouvelles versions et ouvre une **merge request** (une proposition de modification que l'équipe relit avant de la fusionner dans la branche principale) pour chacune. Au lieu d'un retard de cinq ans, tu as chaque semaine quelques petits changements à relire.

Cette leçon ne répète pas les bases. Elle suppose que tu connais le principe, expliqué dans la leçon « Mises à jour automatiques avec Renovate » du parcours *CI/CD avec GitLab* : lis-la d'abord si besoin. Ici, on traite le cas particulier d'un projet **hérité**.

## Le problème d'un projet en retard

Si tu actives Renovate sur PlanningAPI sans réglage, il proposera d'un coup Django 5.2, un saut de sept versions de fonctionnalités. La merge request échouera, personne n'osera la fusionner, et le robot sera ignoré. Il faut donc lui dire de **ne proposer que le palier suivant**.

## Lire le fichier de l'équipe

Voici le `renovate.json` de l'API d'Adhésion, sans le jeton chiffré d'accès à GitHub. Le connecteur Paiement a le même :

```json
{
  "extends": ["config:base"],
  "labels": ["Dependencies", "Maintenance", "Merge request::Needs review"],
  "vulnerabilityAlerts": {
    "labels": ["Priority::Critical"]
  },
  "masterIssue": true,
  "masterIssueApproval": true
}
```

- `extends` reprend une configuration de base.
- `labels` colle des étiquettes sur chaque merge request du robot.
- `vulnerabilityAlerts` ajoute l'étiquette `Priority::Critical` aux mises à jour qui corrigent une faille connue.
- `masterIssue` et `masterIssueApproval` créent une *issue* (une fiche de suivi, comme un ticket) qui liste les mises à jour possibles ; la merge request n'est ouverte que lorsque quelqu'un coche la case.

:::warning Des noms anciens
À ma connaissance, `masterIssue` et `masterIssueApproval` sont d'**anciens noms** : Renovate les a remplacés par `dependencyDashboard` et `dependencyDashboardApproval` (le « tableau de bord des dépendances »), et `config:base` par `config:recommended`. Renovate sait en général lire les anciens noms et les convertir, mais ne les recopie pas dans un nouveau fichier : vérifie dans la documentation de Renovate avant de modifier un fichier existant. Le labo te fait faire cette mise à jour.
:::

Constat à faire avec l'équipe : le `requirements.txt` du connecteur Paiement indique encore Django 3.0.7, malgré ce fichier. On ne peut pas savoir, depuis les dépôts, si le robot est actif sur ce projet ni pourquoi les mises à jour n'ont pas été fusionnées : c'est à vérifier.

## Avancer par paliers

Deux réglages suffisent. Dans la section `packageRules` (des règles qui s'appliquent à des paquets précis) :

```json
{
  "extends": ["config:recommended"],
  "labels": ["Dependencies", "Maintenance", "Merge request::Needs review"],
  "packageRules": [
    {
      "matchPackageNames": ["django"],
      "allowedVersions": "<3.3"
    },
    {
      "matchPackageNames": ["djangorestframework", "django-import-export"],
      "groupName": "dépendances Django"
    }
  ]
}
```

- `"allowedVersions": "<3.3"` interdit à Renovate de proposer Django 3.3 ou plus : il s'arrête donc à 3.2 (LTS) et ses corrections. Une fois le palier fusionné, tu relèves le plafond (`<4.3`, puis `<5.3`).
- `"groupName"` regroupe plusieurs paquets liés dans **une seule** merge request, plus simple à tester que trois séparées.
- `config:recommended` est le nom actuel de la configuration de base appelée `config:base` dans le fichier de l'équipe (à ma connaissance ; vérifie dans la documentation de Renovate avant de copier).

Ce JSON a été vérifié comme syntaxiquement valide, mais pas exécuté par Renovate lui-même. L'outil `renovate-config-validator` de Renovate permet de le contrôler complètement.

:::tip Le plafond est une dette visible
Un `allowedVersions` oublié bloque silencieusement les mises à jour. Ajoute un commentaire dans la merge request qui le pose, et une ligne dans la documentation du projet pour le relever à chaque palier.
:::

:::warning Un pipeline vert ne suffit pas
Un *pipeline* (la suite d'étapes automatiques de la CI) vert veut dire « les tests passent ». Renovate s'appuie sur les tests : sans eux (leçon précédente), une merge request verte ne prouve rien. Relis le *changelog* lié par le robot (la liste des changements publiée par les auteurs de la dépendance) avant de fusionner.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Le `renovate.json` de l'API d'Adhésion (sans son jeton) et son `requirements.txt` sont dans ton dossier de travail. Tu les adaptes pour un projet en retard. Renovate n'est pas installé ici : le serveur vérifie que ton fichier est un JSON valide et contient les bons réglages, mais il ne lance pas le robot.
commandes:
  - cp -R /opt/exercices/05-renovate/. .
etapes:
  - texte: 'Remplace la configuration de base `config:base` par son nom actuel, `config:recommended`'
    indice: 'Dans `extends`, un seul changement : `"extends": ["config:recommended"]`. Édite avec `nano renovate.json`.'
    verif:
      - commande-reussit: '/opt/outils/verifier-legacy renovate 1'
    solution:
      - sed -i 's/config:base/config:recommended/' renovate.json

  - texte: 'Remplace `masterIssue` et `masterIssueApproval` par leurs noms actuels, `dependencyDashboard` et `dependencyDashboardApproval`'
    indice: 'Les deux réglages gardent la valeur `true`. Contrôle avec `grep -n Dashboard renovate.json`.'
    verif:
      - commande-reussit: '/opt/outils/verifier-legacy renovate 2'
    solution:
      - sed -i 's/masterIssueApproval/dependencyDashboardApproval/; s/masterIssue/dependencyDashboard/' renovate.json

  - texte: 'Ajoute une règle `packageRules` qui plafonne `django` avec `"allowedVersions": "<3.3"`'
    indice: 'Une liste `packageRules` contenant `{ "matchPackageNames": ["django"], "allowedVersions": "<3.3" }`. Attention aux virgules : le fichier doit rester un JSON valide.'
    verif:
      - commande-reussit: '/opt/outils/verifier-legacy renovate 3'
    solution:
      - |-
        python3 - <<'PY'
        import json
        d = json.load(open("renovate.json"))
        d["packageRules"] = [{"matchPackageNames": ["django"], "allowedVersions": "<3.3"}]
        json.dump(d, open("renovate.json", "w"), indent=2)
        PY

  - texte: 'Ajoute une seconde règle qui regroupe `djangorestframework` et `django-import-export` dans un lot nommé `dépendances Django`'
    indice: 'Une règle avec `"matchPackageNames": ["djangorestframework", "django-import-export"]` et `"groupName": "dépendances Django"`.'
    apres: [3]
    verif:
      - commande-reussit: '/opt/outils/verifier-legacy renovate 4'
    solution:
      - |-
        python3 - <<'PY'
        import json
        d = json.load(open("renovate.json"))
        d["packageRules"].append({"matchPackageNames": ["djangorestframework", "django-import-export"], "groupName": "dépendances Django"})
        json.dump(d, open("renovate.json", "w"), indent=2, ensure_ascii=False)
        PY

  - texte: 'Le palier 3.2 est fusionné : relève le plafond de Django à `<4.3`, et garde les deux règles'
    indice: 'Change seulement la valeur de `allowedVersions`, de `<3.3` à `<4.3`.'
    apres: [3, 4]
    verif:
      - commande-reussit: '/opt/outils/verifier-legacy renovate 5'
    solution:
      - sed -i 's/<3.3/<4.3/' renovate.json
:::

## Vérifie tes acquis

:::quiz
Pourquoi plafonner Django à `<3.3` avec `allowedVersions` sur un projet en 3.1 ?

- [ ] Pour désactiver Renovate
- [ ] Parce que Django 3.3 contient une faille
- [ ] Pour que Renovate ignore les mises à jour de sécurité
- [x] Pour qu'il propose seulement le palier suivant, et non un saut de plusieurs versions

> Le plafond force une montée par paliers ; tu le relèves après chaque palier fusionné.
:::

:::quiz
À quoi sert `groupName` dans une règle `packageRules` ?

- [x] À regrouper plusieurs mises à jour dans une seule merge request
- [ ] À renommer le paquet dans `requirements.txt`
- [ ] À créer une branche protégée
- [ ] À choisir le nom du robot

> Des paquets qui évoluent ensemble sont plus simples à tester et à relire en un seul lot.
:::

:::quiz
Dans le `renovate.json` d'Adhésion, que fait le réglage `"masterIssueApproval": true` (aujourd'hui `dependencyDashboardApproval`) ?

- [ ] Il fusionne automatiquement les merge requests
- [ ] Il interdit les mises à jour de sécurité
- [x] Il n'ouvre une merge request que lorsque quelqu'un coche la case dans l'issue de suivi
- [ ] Il envoie un courriel à chaque mise à jour

> L'approbation par case à cocher évite d'être submergé par les merge requests du robot.
:::
