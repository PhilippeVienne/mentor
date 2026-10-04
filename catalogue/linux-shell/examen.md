---
titre: "Examen de validation — Linux et shell"
tirage: 5
seuil: 80
duree: 10
melange: true
---

Cet examen valide les bases du terminal Linux : fichiers, droits, services, recherche, scripts et SSH.

:::quiz
Quel chemin est un chemin absolu ?

- [ ] `../etc/hosts`
- [x] `/etc/hosts`
- [ ] `etc/hosts`

> Un chemin absolu commence par `/` et part de la racine du système.
:::
:::quiz
Quelle commande déplace `a.txt` dans `/tmp` ?

- [ ] `cp a.txt /tmp` (le fichier d'origine disparaît)
- [ ] `ls a.txt /tmp`
- [x] `mv a.txt /tmp/`

> `mv` déplace. `cp` copie et laisse l'original en place.
:::
:::quiz
Que signifie `drwxr-xr-x` en début de ligne de `ls -l` ?

- [x] Un dossier, modifiable seulement par son propriétaire
- [ ] Un fichier ordinaire exécutable par tout le monde
- [ ] Un lien cassé vers un autre dossier

> Le `d` initial indique un dossier. Seul le propriétaire a `w`, le groupe et les autres peuvent lire et traverser.
:::
:::quiz
Quel droit octal correspond à `rw-r-----` ?

- [ ] `755`
- [ ] `600`
- [x] `640`

> Propriétaire `rw-` = 6, groupe `r--` = 4, autres `---` = 0.
:::
:::quiz
Pourquoi éviter de tout lancer avec `sudo` ?

- [ ] Parce que les commandes deviennent plus lentes
- [x] Parce qu'une erreur de frappe s'exécute alors avec tous les droits
- [ ] Parce que `sudo` efface l'historique des commandes

> Les droits élevés doivent rester limités à la commande qui en a besoin.
:::
:::quiz
Quelle commande liste les processus en cours ?

- [x] `ps aux`
- [ ] `ls /proc/stop`
- [ ] `journalctl -p`

> `ps aux` affiche les processus, avec leur PID. `top` les montre en direct.
:::
:::quiz
Quel signal est envoyé par `kill -9 PID` ?

- [ ] `SIGTERM`, qui demande un arrêt propre
- [ ] `SIGHUP`, qui relit la configuration
- [x] `SIGKILL`, qui force l'arrêt

> `SIGKILL` ne laisse pas le processus se terminer proprement, d'où l'usage en dernier recours.
:::
:::quiz
Quelle commande lance le service `nginx` automatiquement à chaque démarrage de la machine ?

- [ ] `systemctl start nginx`
- [x] `systemctl enable nginx`
- [ ] `systemctl is-active nginx`

> `start` ne concerne que le moment présent, `enable` agit pour les démarrages suivants.
:::
:::quiz
Comment afficher les 100 dernières lignes de journal du service `postgresql` ?

- [x] `journalctl -u postgresql -n 100`
- [ ] `systemctl log postgresql 100`
- [ ] `tail -u postgresql 100`

> `-u` choisit le service, `-n` le nombre de lignes.
:::
:::quiz
Que contient `sortie.txt` après `echo "a" > sortie.txt` puis `echo "b" >> sortie.txt` ?

- [ ] Uniquement `b`
- [ ] Uniquement `a`
- [x] Deux lignes : `a` puis `b`

> Le premier `>` crée ou écrase, le second `>>` ajoute à la suite.
:::
:::quiz
Quelle commande compte le nombre de fichiers listés dans `/etc` ?

- [ ] `ls /etc > wc`
- [ ] `wc /etc | ls`
- [x] `ls /etc | wc -l`

> Le tube envoie la liste produite par `ls` à `wc -l`, qui compte les lignes.
:::
:::quiz
Quelle option de `grep` ignore la différence entre majuscules et minuscules ?

- [x] `-i`
- [ ] `-v`
- [ ] `-c`

> `-v` inverse la sélection et `-c` compte les lignes trouvées.
:::
:::quiz
Quelle commande cherche des fichiers dont le nom finit par `.tmp` dans le dossier courant ?

- [ ] `grep -r "*.tmp" .`
- [x] `find . -name "*.tmp"`
- [ ] `ls -tmp .`

> `find` cherche par nom, `grep` dans le contenu des fichiers.
:::
:::quiz
Que provoque `set -e` dans un script ?

- [ ] L'affichage des commandes avant exécution
- [ ] La suppression des variables inutilisées
- [x] L'arrêt du script dès qu'une commande échoue

> C'est la sécurité utilisée par `startme.sh` de l'équipe pour vérifier ses prérequis.
:::
:::quiz
Quel code de retour signale qu'une commande a réussi ?

- [ ] `1`
- [x] `0`
- [ ] `255`

> `0` = succès, toute autre valeur = erreur. La variable `$?` le donne.
:::
:::quiz
Quelle ligne rend un script `deploy.sh` exécutable puis le lance ?

- [ ] `exec deploy.sh && chmod 777`
- [x] `chmod +x deploy.sh` puis `./deploy.sh`
- [ ] `bash +x deploy.sh`

> Il faut le droit d'exécution, puis un chemin (`./`) pour le lancer depuis le dossier courant.
:::
:::quiz
Quelle partie d'une paire de clés SSH peut être copiée sur un serveur ?

- [ ] La clé privée, pour s'authentifier
- [ ] Les deux, par sécurité
- [x] La clé publique

> Seule la clé publique se dépose dans `authorized_keys`. La privée ne quitte jamais ta machine.
:::
:::quiz
Quelle commande copie `rapport.pdf` vers `/tmp` sur un serveur distant ?

- [x] `scp rapport.pdf alice@serveur.example.org:/tmp/`
- [ ] `cp rapport.pdf alice@serveur.example.org/tmp`
- [ ] `ssh-copy-id rapport.pdf /tmp`

> `scp` copie par SSH et le côté distant s'écrit `utilisateur@machine:chemin`.
:::
