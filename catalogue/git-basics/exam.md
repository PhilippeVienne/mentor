---
title: "Examen de validation — Git basics"
draw: 12
pass_mark: 80
minutes: 20
shuffle: true
---

Cet examen s'adresse aux personnes qui maîtrisent déjà Git (commits, historique, branches, conflits, dépôts distants, flux de travail en équipe) et veulent **valider le parcours sans refaire les labos** ni les étapes des leçons. Si tu découvres le sujet, commence plutôt par les leçons : elles t'apportent aussi de l'XP.

## Comment ça se passe

- **12 questions** sont tirées au hasard dans un pool de **48 questions** couvrant toutes les leçons du parcours, puis **mélangées** (les réponses aussi).
- Tu as **20 minutes** et une seule tentative en cours à la fois.
- Il faut **au moins 80 %** de bonnes réponses pour réussir et valider le parcours.
- En cas d'échec, un court délai t'est demandé avant de pouvoir réessayer : le pool est tiré à nouveau, donc les questions changent.
- Une fois l'examen rendu, tu obtiens la **correction détaillée** de chaque question, avec l'explication.

## Le pool de questions

:::quiz
Tu supprimes par erreur le dossier caché `.git` d'un projet (sans rien avoir poussé sur un serveur). Que perds-tu ?

- [x] L'historique local ; les fichiers actuels restent, sans être versionnés
- [ ] Tes fichiers de travail, qui sont supprimés avec le dossier caché
- [ ] Seulement le dernier commit : le reste est reconstruit automatiquement depuis les fichiers
- [ ] Rien : Git garde une copie de l'historique dans ton dossier personnel

> Le dossier `.git` contient l'historique (commits, branches, configuration du dépôt). Les fichiers visibles du projet sont à côté, pas dedans : ils survivent, mais sans `.git` le dossier n'est plus un dépôt.
:::

:::quiz
Tu es dans le train, sans connexion Internet, avec un dépôt Git déjà cloné. Laquelle de ces actions reste possible ?

- [ ] Pousser tes commits vers GitLab pour les sauvegarder
- [ ] Récupérer les commits d'un·e coéquipier·ère avec `git pull`
- [ ] Ouvrir une merge request
- [x] Créer des commits et des branches, consulter l'historique

> Commit, branche, log, diff, merge sont des opérations locales. Seules les commandes qui parlent au serveur (`fetch`, `pull`, `push`, `clone`) ont besoin du réseau, et la merge request est une fonction de GitLab, pas de Git.
:::

:::quiz
Que change `git config --global user.name "Alice Martin"` ?

- [ ] Le nom du dossier du projet courant, utilisé pour nommer le dépôt
- [ ] Ton identifiant de connexion sur GitLab, pour t'authentifier au push
- [x] Le nom inscrit dans tous tes futurs commits, sur tous tes dépôts
- [ ] Le nom de la branche par défaut des nouveaux dépôts

> Cette configuration sert à signer tes commits (nom et adresse e-mail). `--global` la range dans ton fichier de configuration personnel, donc elle s'applique à tous les dépôts, sans toucher à GitLab ni aux noms de branches.
:::

:::quiz
Dans un dossier qui contient déjà un fichier `README.md`, tu lances `git init` puis `git status`. Que vois-tu pour `README.md` ?

- [ ] Il est déjà dans le premier commit, créé par `git init`
- [x] Il est listé comme fichier non suivi (*untracked*)
- [ ] Il est indexé et prêt à être commité
- [ ] Git ne le liste pas tant que tu n'as pas fait de commit

> `git init` crée seulement le dépôt vide (le dossier `.git`). Aucun fichier n'est suivi tant qu'on ne l'a pas ajouté avec `git add` : `git status` les montre dans « Fichiers non suivis ».
:::

:::quiz
Qu'est-ce qu'un commit ?

- [ ] La liste des seules lignes modifiées depuis la veille, sans le reste du projet
- [x] Un instantané du projet, avec un message, un auteur et une date
- [ ] Une copie du projet envoyée sur le serveur de l'équipe
- [ ] Un fichier de sauvegarde qu'on peut ouvrir avec un éditeur de texte

> Un commit enregistre l'état du projet à un instant donné et le décrit. Il reste local tant qu'on ne l'envoie pas avec `git push`, et on le consulte avec des commandes comme `git log` ou `git show`.
:::

:::quiz
Une collègue te dit : « Mon code est sur Git, donc il est forcément sur GitLab. » Que lui réponds-tu ?

- [x] Faux : il faut un `git push` pour que le code arrive sur GitLab
- [ ] Vrai : chaque commit est envoyé automatiquement sur GitLab dès qu'il est créé
- [ ] Faux : Git ne peut pas travailler avec GitLab, il faut un autre outil dédié
- [ ] Vrai : GitLab est le programme qui enregistre les commits sur ton poste

> Git est l'outil installé sur ta machine ; GitLab est un service qui héberge des dépôts. Rien n'est envoyé sans commande explicite.
:::

:::quiz
`index.html` est déjà suivi par Git. Tu le modifies, puis tu lances `git commit -m "Change le titre"` sans rien d'autre. Que se passe-t-il ?

- [ ] Un commit est créé avec ta modification, car le fichier est déjà suivi
- [ ] Un commit vide est créé avec le message donné, sans changement
- [ ] Git ouvre un éditeur pour te demander ce que tu veux commiter
- [x] Aucun commit : Git signale que rien n'a été ajouté à la validation

> Un commit ne contient que ce qui est dans l'index. Sans `git add index.html` (ou `git commit -a`), la modification reste dans le dossier de travail et Git refuse de créer un commit sans changement indexé.
:::

:::quiz
Voici la fin d'une sortie de `git status` :

    Modifications qui seront validées :
            nouveau fichier : a.txt

    Fichiers non suivis :
            b.txt

Que contiendra le prochain commit ?

- [ ] `a.txt` et `b.txt`
- [ ] Seulement `b.txt`
- [x] Seulement `a.txt`
- [ ] Rien : il faut d'abord un `git add .` pour tout valider

> La rubrique « Modifications qui seront validées » décrit le contenu de l'index, donc du prochain commit. Les fichiers non suivis, comme `b.txt`, n'en font pas partie tant qu'on ne les ajoute pas.
:::

:::quiz
`notes.txt` est un nouveau fichier jamais ajouté, et `index.html` est suivi et modifié. Que fait `git commit -am "Mise à jour"` ?

- [ ] Elle commite `index.html` et `notes.txt`
- [ ] Elle commite seulement `notes.txt`
- [x] Elle commite `index.html`, mais pas `notes.txt`
- [ ] Elle échoue car `-a` et `-m` ne peuvent pas être combinés

> L'option `-a` indexe automatiquement les fichiers déjà suivis qui ont été modifiés ou supprimés, mais jamais les nouveaux fichiers : ceux-là demandent un `git add` explicite.
:::

:::quiz
Lequel de ces messages de commit sera le plus utile dans six mois ?

- [ ] Modifs
- [x] Corrige le calcul de la TVA sur les remises
- [ ] fix
- [ ] Dernière version avant la réunion

> Un bon message décrit ce que fait le commit, à l'impératif, en une ligne courte. « fix » ou « Modifs » n'apprennent rien à la personne qui lira l'historique, et la dernière proposition parle du contexte du moment plutôt que du changement.
:::

:::quiz
Tu as lancé `git add rapport.md`, puis tu modifies encore `rapport.md` avant de commiter. Que contient le commit ?

- [x] La version de `rapport.md` telle qu'elle était au moment du `git add`
- [ ] La version la plus récente du fichier, car Git relit le fichier au commit
- [ ] Les deux versions, enregistrées côte à côte
- [ ] Rien : le fichier est refusé tant qu'il change

> `git add` copie le contenu du fichier dans l'index à cet instant précis. La modification postérieure apparaît à nouveau dans « Modifications qui ne seront pas validées » : il faut un second `git add` pour l'inclure.
:::

:::quiz
Pourquoi relire `git status` avant de taper `git add .` ?

- [ ] Parce que `git add .` ne fonctionne que si `git status` a été lancé juste avant
- [ ] Parce que `git status` met à jour l'index avant chaque ajout
- [ ] Parce que `git add .` supprime les fichiers qui ne sont pas encore suivis
- [x] Parce que `.` ajoute tout, y compris des fichiers qu'on ne veut pas versionner

> `git add .` indexe tout ce qui a changé sous le dossier courant. Un mot de passe, un fichier `.env` ou un gros fichier généré peut s'y glisser : la lecture de `git status` est le dernier filet de sécurité.
:::

:::quiz
Après ton tout premier commit, Git affiche `[main (commit racine) 8f26519] Initialise le projet`. Que signifie `(commit racine)` ?

- [ ] C'est un commit réalisé avec les droits administrateur
- [ ] C'est le commit qui a été poussé sur le serveur
- [ ] C'est un commit qui contient tous les fichiers du système
- [x] C'est le premier commit de la branche : il n'a pas de parent

> Un commit racine est un commit sans parent, donc le point de départ de l'historique. `8f26519` est l'abréviation de son identifiant.
:::

:::quiz
Que désigne `HEAD~2` ?

- [ ] Le deuxième commit de l'historique du projet depuis le début
- [ ] Le deuxième fichier modifié depuis le dernier commit
- [x] Le commit situé deux crans avant le commit courant
- [ ] Les deux derniers commits, pris ensemble

> `HEAD` est le commit sur lequel tu es, `HEAD~1` son parent et `HEAD~2` son grand-parent. C'est un identifiant relatif, pratique avec `git show`, `git diff` ou `git reset`.
:::

:::quiz
Tu viens d'indexer `a.txt` avec `git add a.txt` après l'avoir modifié. `git diff` n'affiche rien. Pourquoi ?

- [ ] Parce que `git add` a annulé ta modification dans le dossier de travail du projet
- [x] Parce que `git diff` compare le dossier de travail à l'index, qui sont identiques ici
- [ ] Parce que `git diff` ne fonctionne que sur les fichiers déjà commités dans l'historique
- [ ] Parce qu'il faut d'abord lancer `git status` pour rafraîchir l'index

> Pour voir ce qui va partir au prochain commit, il faut `git diff --staged` : cette variante compare l'index au dernier commit.
:::

:::quiz
Tu as indexé un fichier par erreur, mais tu veux garder tes modifications dans le fichier. Quelle commande convient ?

- [x] `git restore --staged fichier`
- [ ] `git restore fichier`
- [ ] `git reset --hard`
- [ ] `git rm fichier`

> `git restore --staged` retire le fichier de l'index sans toucher au contenu du dossier de travail. `git restore fichier` sans option, `git reset --hard` et `git rm` détruisent ou suppriment ton travail.
:::

:::quiz
Tu as des modifications non commitées dans ton dossier de travail et tu lances `git reset --hard HEAD~1`. Quelle est la conséquence ?

- [x] Le dernier commit disparaît et tes modifications non commitées sont perdues
- [ ] Seul le dernier commit disparaît, tes modifications non commitées sont conservées
- [ ] Rien ne change tant que tu n'as pas fait de `git add`
- [ ] Git refuse la commande tant que le dossier de travail n'est pas propre

> L'option `--hard` aligne l'index et le dossier de travail sur le commit cible : tout ce qui n'a pas été commité est écrasé, sans possibilité de récupération.
:::

:::quiz
Tu as commité un fichier `mots_de_passe.txt` par erreur, sans avoir poussé. Tu veux défaire le commit mais garder le fichier pour le supprimer ensuite. Quelle commande ?

- [ ] `git reset --hard HEAD~1`
- [ ] `git revert HEAD`
- [ ] `git restore HEAD~1`
- [x] `git reset HEAD~1`

> Le `reset` par défaut (mode *mixed*) recule la branche et désindexe, en laissant les fichiers dans le dossier de travail. `--hard` effacerait le fichier, `revert` ajouterait un commit inverse (adapté après un push), et `restore` ne déplace pas la branche.
:::

:::quiz
Un commit contenant une erreur a déjà été poussé sur `main` et tes collègues l'ont récupéré. Quelle est la bonne façon de l'annuler ?

- [ ] `git reset --hard` puis `git push --force`
- [ ] Supprimer le dossier `.git` et recloner
- [x] `git revert <commit>` puis pousser le nouveau commit
- [ ] `git commit --amend` puis pousser de force

> `git revert` crée un commit qui annule le précédent sans réécrire l'historique partagé. Réécrire l'historique d'une branche déjà récupérée par d'autres les oblige à réparer leurs copies.
:::

:::quiz
Tu viens de commiter avec une faute dans le message et tu n'as rien poussé. Que fait `git commit --amend -m "Nouveau message"` ?

- [ ] Il modifie le message sans changer l'identifiant du commit
- [x] Il remplace le dernier commit par un nouveau, avec un autre identifiant
- [ ] Il ajoute un second commit qui contient seulement le message
- [ ] Il annule le commit et supprime ses fichiers

> Un commit est identifié par un hash calculé à partir de son contenu et de son message : changer le message produit un nouveau commit. C'est inoffensif en local, mais à éviter une fois le commit partagé.
:::

:::quiz
Quelle est la différence entre `git branch fonctionnalite` et `git switch -c fonctionnalite` ?

- [ ] La première copie tous les fichiers dans un dossier, la seconde non
- [x] La première ne bascule pas ; la seconde crée la branche puis y bascule
- [ ] La seconde crée aussi la branche sur le serveur distant en même temps
- [ ] Il n'y a aucune différence, ce sont deux écritures de la même commande

> Dans les deux cas la branche est un simple pointeur vers le commit courant. Seule `git switch -c` change aussi la branche sur laquelle tu travailles.
:::

:::quiz
Sur `feature`, tu as commité `contact.html`. Tu fais `git switch main` et le fichier disparaît de ton dossier. Est-ce un problème ?

- [x] Non : ton dossier reflète `main` ; le fichier est toujours dans `feature`
- [ ] Oui : le commit a été supprimé de `feature` en changeant de branche
- [ ] Oui : il faut refaire `git add` pour récupérer le fichier dans le dossier
- [ ] Non : mais le fichier est perdu définitivement tant que tu restes sur `main`

> Changer de branche remplace le contenu suivi du dossier de travail par celui de la branche cible. Le commit de `feature` est intact, et le fichier réapparaît dès que tu reviens dessus.
:::

:::quiz
Dans quel cas Git réalise-t-il une fusion en simple avance rapide (*fast-forward*) ?

- [ ] Quand les deux branches modifient le même fichier de la même façon
- [ ] Quand on utilise l'option `-m` pour donner un message de fusion
- [ ] Quand les deux branches ont le même nom sur le serveur et en local
- [x] Quand la branche cible n'a reçu aucun commit depuis la création de l'autre

> Si l'historique de la cible est contenu dans celui de la branche à fusionner, Git déplace simplement le pointeur : aucun commit de fusion n'est créé et l'historique reste linéaire.
:::

:::quiz
Tu veux intégrer `feature` dans `main`. Que fais-tu ?

- [ ] Se placer sur `feature`, puis lancer `git merge main` pour l'envoyer dans `main`
- [ ] Lancer `git merge feature main` depuis n'importe quelle branche
- [x] Se placer sur `main` (`git switch main`), puis lancer `git merge feature`
- [ ] Lancer `git branch -d main` puis `git switch feature`

> `git merge` intègre la branche nommée dans la branche courante : on se place donc sur celle qui doit recevoir les changements. Depuis `feature`, `git merge main` ferait l'inverse.
:::

:::quiz
`git branch -d ancienne` répond « error : la branche 'ancienne' n'est pas complètement fusionnée ». Que signifie ce message ?

- [ ] La branche est vide, il n'y a rien à supprimer à l'intérieur
- [ ] La branche a des conflits non résolus avec la branche courante
- [x] Elle porte des commits absents de la branche courante
- [ ] Tu n'as pas les droits pour supprimer cette branche du dépôt

> Git protège le travail non intégré. `-D` force la suppression, mais les commits qu'elle était seule à porter ne sont alors plus atteignables facilement.
:::

:::quiz
Que se passe-t-il dans le dépôt quand tu crées une nouvelle branche ?

- [ ] Tout le projet est dupliqué dans un nouveau dossier à côté du dépôt
- [x] Un pointeur est créé vers le commit courant, sans copier de fichier
- [ ] Tous les commits sont copiés avec de nouveaux identifiants
- [ ] Un nouveau dépôt est initialisé à l'intérieur du dossier `.git`

> Une branche est une simple étiquette mobile vers un commit. C'est ce qui rend leur création instantanée, et pourquoi on peut en ouvrir autant que nécessaire.
:::

:::quiz
Tu as modifié `README.md` sans commiter, et la branche vers laquelle tu veux basculer contient une version différente de `README.md`. Que fait `git switch` ?

- [x] Il refuse de basculer et signale que tes modifications seraient écrasées
- [ ] Il bascule et écrase silencieusement tes modifications locales
- [ ] Il bascule et fusionne automatiquement les deux versions du fichier
- [ ] Il crée un commit temporaire de tes modifications, puis bascule

> Git ne détruit pas de travail non commité sans te le dire. Il faut d'abord commiter, ou annuler tes modifications. Si le fichier était identique sur les deux branches, ta modification t'aurait suivi.
:::

:::quiz
Dans quel cas Git fusionne-t-il deux branches sans aucun conflit ?

- [ ] Quand elles ont été créées le même jour par la même personne
- [ ] Quand elles contiennent exactement le même nombre de commits
- [ ] Jamais : toute fusion de deux branches provoque un conflit
- [x] Quand elles touchent des fichiers ou des zones de fichier différents

> Un conflit n'apparaît que lorsque les deux côtés ont modifié la même zone de façon différente. Dans les autres cas, Git sait combiner les changements seul.
:::

:::quiz
Voici un extrait de fichier pendant une fusion de `titre-demo` dans `main` :

    <<<<<<< HEAD
    <h1>Accueil</h1>
    =======
    <h1>Bienvenue chez Mentor</h1>
    >>>>>>> titre-demo

Quelle ligne vient de la branche `titre-demo` ?

- [ ] `<h1>Accueil</h1>`
- [ ] Les deux lignes, car elles sont toutes deux dans le fichier
- [ ] Aucune : les marqueurs sont écrits par `main`
- [x] `<h1>Bienvenue chez Mentor</h1>`

> Entre `<<<<<<< HEAD` et `=======` se trouve la version de la branche courante (`main`) ; entre `=======` et `>>>>>>>` celle de la branche fusionnée (`titre-demo`).
:::

:::quiz
Tu as corrigé le fichier en conflit, mais tu lances `git commit` sans avoir fait `git add` dessus. Que répond Git ?

- [ ] Il crée normalement le commit de fusion avec le fichier corrigé
- [ ] Il relance automatiquement la fusion depuis le début de la branche
- [x] Il refuse : des fichiers non fusionnés restent à marquer comme résolus
- [ ] Il supprime le fichier pour éviter toute ambiguïté

> Pendant une fusion, `git add fichier` signifie « ce conflit est réglé ». Tant que des fichiers restent marqués non fusionnés, `git commit` est refusé.
:::

:::quiz
Au milieu d'une fusion en conflit, tu veux tout annuler et revenir à l'état d'avant. Quelle commande ?

- [ ] `git reset --soft HEAD~1`
- [x] `git merge --abort`
- [ ] `git branch -d` sur la branche fusionnée
- [ ] `git commit --amend`

> `git merge --abort` abandonne la fusion en cours et restaure le dossier de travail tel qu'avant la tentative.
:::

:::quiz
Tu as résolu un conflit mais tu as oublié de supprimer une ligne `<<<<<<< HEAD`. Tu fais `git add` puis `git commit`. Que fait Git ?

- [x] Il accepte : c'est à toi de vérifier que les marqueurs ont disparu
- [ ] Il refuse, car il détecte les marqueurs de conflit restant dans le fichier
- [ ] Il retire lui-même les marqueurs qui restent dans le fichier
- [ ] Il restaure automatiquement les deux versions du fichier

> Git ne contrôle pas le contenu : `git add` signifie simplement « résolu ». Les marqueurs seront alors commités et feront échouer la compilation ou la page. Relire le fichier (ou `git diff`) avant d'indexer est un bon réflexe.
:::

:::quiz
Deux personnes ont modifié la même ligne. Laquelle de ces résolutions est valide ?

- [x] Garder un côté, l'autre, ou les combiner, puis supprimer les marqueurs
- [ ] Obligatoirement garder la version de la branche courante, `HEAD`
- [ ] Obligatoirement garder la version de la branche fusionnée, la plus récente
- [ ] Laisser les deux versions avec les marqueurs, puis commiter

> Un conflit demande une décision humaine : on peut garder un côté, l'autre ou un mélange. L'important est que le fichier final soit cohérent et sans marqueurs.
:::

:::quiz
Laquelle de ces pratiques réduit le plus le risque de conflits dans une équipe ?

- [ ] Travailler plusieurs semaines sur une branche sans la synchroniser
- [ ] Modifier les mêmes fichiers que tes collègues pour rester aligné·e
- [ ] Toujours utiliser `--force` au moment de pousser
- [x] Faire des branches courtes et récupérer souvent les changements de `main`

> Plus une branche vit longtemps, plus elle s'éloigne de `main`. De petites branches synchronisées régulièrement donnent de petits conflits, plus faciles à résoudre.
:::

:::quiz
Que désigne `origin` dans la plupart des projets ?

- [ ] La première branche créée dans le projet, par convention
- [ ] Le premier commit de l'historique, d'où part tout le reste
- [x] Le nom par défaut du dépôt distant d'où l'on a cloné
- [ ] L'utilisateur·rice qui a créé le dépôt sur le serveur

> C'est un simple alias pour l'URL du serveur, défini par `git clone` ou `git remote add origin <url>`. On pourrait le nommer autrement, mais la convention est universelle.
:::

:::quiz
Ton `git push` est rejeté avec « ! [rejeté] main -> main (non-fast-forward) ». Que s'est-il passé ?

- [ ] Tu n'as pas fait de commit depuis trop longtemps, le serveur t'a rejeté·e
- [x] Le serveur a des commits que tu n'as pas : récupère-les avant de pousser
- [ ] Le serveur est en panne, il faut réessayer plus tard
- [ ] Ton nom d'utilisateur·rice Git n'est pas configuré sur cette machine

> Quelqu'un a poussé avant toi. Il faut faire `git pull` (fetch puis fusion), régler d'éventuels conflits, puis repousser. Forcer le push écraserait leur travail.
:::

:::quiz
Tu lances `git fetch`, puis `git status` affiche « Votre branche est en retard sur 'origin/main' de 1 commit ». Où est ce commit ?

- [ ] Dans tes fichiers de travail, que `fetch` a déjà mis à jour pour toi
- [x] Dans ton dépôt local (`origin/main`), pas encore dans ta branche
- [ ] Uniquement sur le serveur, car `fetch` ne télécharge rien sans `pull`
- [ ] Il a été fusionné dans ta branche `main` par `fetch`

> `git fetch` télécharge les nouveautés et met à jour `origin/main`, sans toucher à ta branche ni à ton dossier de travail. `git pull` ajoute la fusion.
:::

:::quiz
Quel est l'intérêt de l'option `-u` dans `git push -u origin main` ?

- [x] Elle mémorise la branche amont : ensuite `git push` suffit, sans argument
- [ ] Elle pousse aussi toutes les autres branches locales vers le serveur
- [ ] Elle force la mise à jour même si le serveur a des commits en plus
- [ ] Elle met à jour (*update*) le serveur avec les fichiers non suivis

> Le lien entre branche locale et branche distante est enregistré une fois pour toutes. `-u` est un raccourci de `--set-upstream`.
:::

:::quiz
Que fait `git clone git@gitlab.example.org:equipe/formation-git.git` ?

- [ ] Il télécharge seulement le dernier commit, sans l'historique du projet
- [ ] Il crée un nouveau dépôt vide sur GitLab avec ce nom
- [ ] Il envoie le contenu du dossier courant vers le serveur GitLab
- [x] Il télécharge tout l'historique et configure `origin` automatiquement

> `git clone` copie le dépôt complet (historique compris) et règle `origin` sur l'URL d'origine. On peut ensuite travailler hors ligne et synchroniser avec `fetch`, `pull` et `push`.
:::

:::quiz
Tu vois « Votre branche est en avance sur 'origin/main' de 2 commits ». Que faut-il faire pour publier ton travail ?

- [ ] Lancer `git pull`, car ton dépôt local est en retard sur le serveur
- [ ] Lancer `git fetch`, qui envoie aussi tes commits au serveur
- [x] Lancer `git push` : tu as deux commits que le serveur n'a pas
- [ ] Rien : GitLab récupère tes commits locaux tout seul

> « En avance » signifie que tu as des commits locaux absents du serveur ; « en retard » signifie l'inverse. Seul `git push` envoie tes commits.
:::

:::quiz
Pourquoi `git push --force` est-il déconseillé sur une branche partagée ?

- [ ] Il est beaucoup plus lent qu'un push normal sur une grosse branche
- [ ] Il supprime aussi tous tes fichiers locaux non commités du dossier
- [x] Il peut écraser les commits de tes collègues sur le serveur
- [ ] Il désactive la merge request en cours sur la branche

> Le serveur refuse normalement un push qui perdrait des commits. `--force` lève cette protection : les commits poussés par d'autres entre-temps sont écrasés.
:::

:::quiz
Pourquoi travaille-t-on sur une branche dédiée plutôt que directement sur `main` ?

- [ ] Parce que Git interdit techniquement les commits directs sur `main`
- [x] Pour garder `main` stable et relire les changements avant de les intégrer
- [ ] Parce que les branches rendent le dépôt plus léger sur le disque
- [ ] Parce que `main` ne peut contenir qu'un seul commit à la fois

> Dans le flux « feature branch + merge request », `main` reste toujours fonctionnel. Ce n'est pas une interdiction technique de Git, mais une règle d'équipe, souvent appliquée par une protection de branche sur GitLab.
:::

:::quiz
Ta branche `fix-typo` est publiée sur GitLab. Tu veux qu'elle soit relue et testée avant d'arriver dans `main`. Que fais-tu ?

- [x] J'ouvre une merge request de `fix-typo` vers `main`
- [ ] Je fusionne `fix-typo` dans `main` en local, puis je pousse `main`
- [ ] Je pousse `fix-typo` avec `--force` pour que la CI se déclenche
- [ ] Je supprime `fix-typo` et je refais mes commits directement sur `main`

> La merge request est la demande de fusion de GitLab (l'équivalent d'une *pull request* sur GitHub) : on y discute, on relit et on laisse les tests tourner avant la fusion. Fusionner en local puis pousser `main` contourne la relecture, et `--force` réécrit l'historique partagé sans rien déclencher de plus.
:::

:::quiz
Un fichier `.env` a été commité par erreur. Tu l'ajoutes ensuite à `.gitignore`. Que se passe-t-il ?

- [ ] Git le supprime automatiquement de tout l'historique du dépôt
- [ ] Git ignore désormais le fichier et ne l'enverra plus jamais, même déjà commité
- [ ] Git refuse tous les commits suivants tant que le fichier est présent
- [x] Git continue de suivre le fichier : il faut le désindexer (`git rm --cached`)

> `.gitignore` n'agit que sur les fichiers non suivis. Pour un fichier déjà suivi, il faut le désindexer, et si le fichier contenait un secret, le révoquer car il reste dans l'historique.
:::

:::quiz
Un mot de passe a été commité puis supprimé au commit suivant. Est-ce réglé ?

- [ ] Oui, il n'existe plus dans le dossier de travail du projet
- [ ] Oui, car les anciens commits ne sont pas lisibles par les autres
- [ ] Oui, à condition de fermer puis de rouvrir le terminal
- [x] Non : il reste dans l'historique, il faut le changer ou le révoquer

> N'importe qui ayant accès à l'historique peut retrouver l'ancien contenu. Un secret exposé doit être considéré comme compromis, quel que soit le commit qui l'a retiré.
:::

:::quiz
Quel commit respecte le mieux le principe d'un commit « atomique » ?

- [ ] Un commit qui mélange une correction, une nouvelle fonctionnalité et un reformatage global
- [ ] Un commit par fichier, quel que soit leur lien
- [x] Un commit qui corrige un seul bug, accompagné de son test
- [ ] Un unique commit en fin de semaine regroupant tout

> Un commit atomique porte un changement logique et cohérent. Il se relit, s'annule (`revert`) et se retrouve plus facilement qu'un commit fourre-tout.
:::

:::quiz
Ta merge request vient d'être fusionnée dans `main` sur GitLab. Que fais-tu ensuite en local ?

- [ ] Rien : ton dépôt local est mis à jour automatiquement par GitLab à la fusion
- [x] `git switch main`, `git pull`, puis `git branch -d` pour la branche
- [ ] `git push --force` pour synchroniser ton dépôt avec le serveur
- [ ] `git clone` à nouveau, pour repartir de zéro avec le dépôt

> La fusion a eu lieu sur le serveur. Il faut récupérer `main` à jour avec `git pull`, puis supprimer la branche locale devenue inutile.
:::

:::quiz
Quelle règle suit-on généralement pour écrire le titre d'un commit ?

- [x] Une ligne courte à l'impératif qui décrit ce que fait le commit
- [ ] Le nom de l'auteur suivi de la date
- [ ] Une description détaillée de tout le projet
- [ ] Un identifiant tiré au hasard pour garantir l'unicité

> L'auteur et la date sont déjà enregistrés par Git. Le titre doit se lire comme « ce commit… » suivi d'un verbe, par exemple « Ajoute la validation du formulaire ».
:::
