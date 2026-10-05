---
title: "Examen de validation — Maintenir du code hérité"
draw: 5
pass_mark: 80
minutes: 10
shuffle: true
---

Cet examen valide la méthode de montée de version : état des lieux, tests, Django par paliers, front React, Renovate et passation.

:::quiz
Pourquoi une version de Django qui n'est plus maintenue pose-t-elle problème ?

- [ ] Elle refuse de démarrer sur un serveur récent, quel que soit le système
- [x] Elle ne reçoit plus de correctifs de sécurité
- [ ] Elle supprime des données au bout d'un an
- [ ] Elle interdit l'usage de Git

> Sans maintenance, les failles découvertes ensuite restent ouvertes.
:::

:::quiz
Que signifie `Django==3.1.13` dans un `requirements.txt` ?

- [ ] Django 3.1.13 ou toute version plus récente
- [ ] Une version de Django comprise entre 3.1 et 3.2
- [x] Exactement la version 3.1.13
- [ ] La version de Django à ne pas installer

> Le double signe égal épingle une version précise.
:::

:::quiz
Dans le `package.json` de Planning, que veut dire `"react": "^16.13.1"` ?

- [ ] Exactement React 16.13.1
- [ ] Toute version de React, y compris la 18
- [ ] La version 16.13.1 est interdite
- [x] La version 16.13.1 ou toute version 16.x plus récente

> Le `^` autorise les mises à jour qui gardent le même numéro majeur.
:::

:::quiz
Pourquoi le `Dockerfile` fait-il partie de l'état des lieux ?

- [x] Il indique la version de Python, qui limite les versions de Django possibles
- [ ] Il contient les tests du projet, lancés à chaque fois que quelqu'un modifie le code
- [ ] Il liste les failles de sécurité connues
- [ ] Il décide du nom de la base de données

> Chaque version de Django n'accepte qu'une plage de versions de Python.
:::

:::quiz
Qu'est-ce qu'une version LTS de Django ?

- [x] Une version qui reçoit des correctifs de sécurité plus longtemps
- [ ] Une version réservée aux entreprises
- [ ] Une version sans aucune API dépréciée
- [ ] La toute dernière version publiée, quel que soit son support

> LTS signifie *Long-Term Support* : un support à long terme.
:::

:::quiz
Un projet est en Django 3.1 et doit arriver en 5.2. Quelle stratégie réduit le plus le risque ?

- [ ] Changer le numéro de version une seule fois, puis corriger au fil des erreurs
- [ ] Sauter de 3.1 à 4.0, puis de 4.0 à 5.2 sans passer par les LTS
- [ ] Réécrire l'application dans un autre framework, plus récent
- [x] S'arrêter sur chaque LTS (3.2, puis 4.2, puis 5.2), tests verts à chaque fois

> Chaque LTS est une étape stable où l'on teste, fusionne et déploie.
:::

:::quiz
Tu hérites d'une fonction que personne ne comprend plus et tu dois la modifier. Quel test écris-tu d'abord ?

- [ ] Un test qui mesure sa vitesse d'exécution sur une grosse base de données
- [ ] Un test qui vérifie sa conformité à un cahier des charges rédigé avant le projet
- [x] Un test qui note ce qu'elle renvoie aujourd'hui, pour repérer tout changement après ta modification
- [ ] Un test écrit uniquement avec les exemples de la documentation de Django

> C'est un test de caractérisation : on fige le comportement actuel, même s'il est discutable, pour détecter ce qui change.
:::

:::quiz
Un test dépend d'un serveur Keycloak qui n'existe pas en CI. Que fais-tu ?

- [ ] Je supprime le test, puisqu'il ne peut pas tourner en CI
- [x] Je remplace l'appel à Keycloak par un faux (*mock*)
- [ ] Je marque le pipeline comme réussi à la main
- [ ] Je lance Keycloak sur mon poste et j'ignore la CI

> Un faux répond de façon prévue : le test vérifie ton code, pas un service externe.
:::

:::quiz
Quelle commande fait échouer les tests si le code utilise une fonction dépréciée ?

- [ ] `python manage.py check --deploy`
- [ ] `pip check`
- [ ] `python manage.py makemigrations --check`
- [x] `python -W error::DeprecationWarning manage.py test`

> L'option `-W error::DeprecationWarning` transforme chaque avertissement en erreur.
:::

:::quiz
Les tests affichent `RemovedInDjango60Warning` à propos d'une fonction. Que dois-tu en conclure ?

- [ ] Que la migration de la base de données a échoué lors de la création de la base de test
- [x] Que le code utilise une fonction qui disparaîtra de Django 6.0 : il faut la remplacer
- [ ] Que Django 6.0 est déjà installé sur la machine
- [ ] Que le fichier contient une erreur de syntaxe Python

> Le numéro dans le nom indique la version qui retirera la fonction.
:::

:::quiz
Que se passe-t-il avec `from django.conf.urls import url` sous Django 5.2 ?

- [ ] Rien, c'est seulement déprécié
- [ ] Django traduit l'import automatiquement vers la nouvelle fonction au démarrage
- [x] Une `ImportError`, car `url` a été retiré ; on utilise `re_path`
- [ ] Les URL ne fonctionnent plus qu'en HTTP

> `url` a été supprimé dans Django 4.0.
:::

:::quiz
Que vérifie `python manage.py makemigrations --check --dry-run` ?

- [x] Qu'aucune nouvelle migration ne serait créée
- [ ] Que les tests réussissent sur toutes les versions de Python installées
- [ ] Que la base de production est sauvegardée
- [ ] Que les dépendances sont à jour

> Une montée de version ne doit pas modifier le schéma par surprise.
:::

:::quiz
Pourquoi remplacer `react-scripts` avant de monter React ?

- [ ] Parce que `react-scripts` est payant
- [ ] Parce que Vite installe automatiquement React 18
- [x] Pour ne changer qu'une chose à la fois et garder une application fonctionnelle
- [ ] Parce qu'il n'existe aucun autre moyen de lancer des tests avec une version récente de Node.js

> Si l'outil est déjà à jour, une erreur après la montée de React vient de React.
:::

:::quiz
Dans un projet Vite, comment s'appelle une variable d'environnement lisible par le navigateur ?

- [ ] `REACT_APP_NOM`
- [ ] `NODE_NOM`
- [x] `VITE_NOM`
- [ ] `PUBLIC_NOM` obligatoirement

> Vite n'expose au navigateur que les variables préfixées par `VITE_`.
:::

:::quiz
Que fait `"allowedVersions": "<3.3"` pour Django dans un `renovate.json` ?

- [ ] Il désinstalle Django 3.3 de tous les projets qui l'utilisent déjà
- [x] Il empêche Renovate de proposer Django 3.3 ou plus
- [ ] Il impose Django 3.3 à tous les projets
- [ ] Il désactive les alertes de sécurité

> Le plafond force une montée palier par palier.
:::

:::quiz
Quel est l'intérêt d'un tag Git posé avant un palier de montée de version ?

- [ ] Il lance la CI plus vite
- [ ] Il met à jour les dépendances
- [ ] Il remplace le journal de montée de version
- [x] Il fournit un point de retour fixe et facile à retrouver

> Un tag marque un commit précis : revenir en arrière devient une commande.
:::

:::quiz
Que doit contenir en priorité le journal d'une montée de version ?

- [x] Ce qui a été fait, pourquoi, ce qui a été laissé de côté et le retour arrière
- [ ] Uniquement la liste des commits
- [ ] Seulement le nom des relecteurs et relectrices
- [ ] Les mots de passe des environnements, pour que la personne suivante gagne du temps

> Ces informations sont celles dont la personne suivante aura besoin en cas de problème.
:::
