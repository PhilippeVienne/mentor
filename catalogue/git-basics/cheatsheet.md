## Démarrer

| Commande | Effet |
| --- | --- |
| `git init` | Crée un dépôt dans le dossier courant |
| `git clone <url>` | Copie un dépôt distant en local |
| `git config --global user.name "Nom"` | Définit ton nom (à faire une fois) |
| `git config --global user.email "mail"` | Définit ton adresse mail |

## Au quotidien

| Commande | Effet |
| --- | --- |
| `git status` | Où j'en suis ? (à lancer tout le temps) |
| `git add <fichier>` | Ajoute un fichier à l'index |
| `git add .` | Ajoute tous les changements à l'index |
| `git commit -m "message"` | Crée un commit avec ce qui est dans l'index |
| `git commit -am "message"` | `add` des fichiers déjà suivis + `commit` |
| `git diff` | Voit les changements non indexés |
| `git diff --staged` | Voit les changements indexés |

## Historique

| Commande | Effet |
| --- | --- |
| `git log` | Historique détaillé |
| `git log --oneline` | Historique compact |
| `git log --oneline --graph --all` | Historique avec toutes les branches |
| `git show <commit>` | Détail d'un commit |

## Annuler

| Commande | Effet |
| --- | --- |
| `git restore <fichier>` | Abandonne les modifications non indexées d'un fichier |
| `git restore --staged <fichier>` | Retire un fichier de l'index |
| `git commit --amend -m "msg"` | Corrige le dernier commit |
| `git reset HEAD~1` | Défait le dernier commit, garde les fichiers |
| `git reset --hard HEAD~1` | Défait le dernier commit **et** les fichiers (danger) |
| `git revert <commit>` | Crée un commit qui annule un commit passé |

## Branches

| Commande | Effet |
| --- | --- |
| `git branch` | Liste les branches |
| `git switch -c <nom>` | Crée une branche et bascule dessus |
| `git switch <nom>` | Change de branche |
| `git merge <branche>` | Fusionne une branche dans la courante |
| `git merge --abort` | Annule une fusion en conflit |
| `git branch -d <nom>` | Supprime une branche fusionnée |

## Collaborer

| Commande | Effet |
| --- | --- |
| `git remote add origin <url>` | Déclare le dépôt distant |
| `git remote -v` | Liste les dépôts distants |
| `git push -u origin <branche>` | Publie une branche et mémorise l'amont |
| `git push` | Envoie tes commits |
| `git fetch` | Télécharge sans fusionner |
| `git pull` | Télécharge et fusionne |
