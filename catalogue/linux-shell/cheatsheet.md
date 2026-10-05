## Fichiers et navigation

| Commande | Rôle |
| --- | --- |
| `pwd` | Affiche le dossier courant |
| `ls -la` | Liste le contenu, fichiers cachés compris |
| `cd chemin` / `cd ..` / `cd ~` | Va dans un dossier / remonte / retourne chez toi |
| `mkdir -p a/b` | Crée un dossier et ses parents |
| `cp` / `mv` / `rm` | Copie (`-r` pour un dossier) / déplace ou renomme / supprime (définitif !) |
| `cat` / `less` / `head` / `tail -f` | Lit un fichier / le fait défiler / début / fin en direct |

## Droits

| Notation | Sens |
| --- | --- |
| `rwx` | Lire (4), écrire (2), exécuter (1) |
| `chmod u+x f` | Ajoute l'exécution au propriétaire |
| `chmod 640 f` | `rw-` propriétaire, `r--` groupe, `---` autres |
| `chown u:g f` | Change propriétaire et groupe |
| `sudo cmd` | Exécute une commande avec les droits de `root` |

## Processus, services, journaux

| Commande | Rôle |
| --- | --- |
| `ps aux` / `top` | Liste les processus / activité en direct |
| `kill PID` / `kill -9 PID` | Arrêt propre / arrêt forcé |
| `systemctl status\|start\|stop\|restart svc` | Pilote un service |
| `systemctl enable svc` | Lance le service à chaque démarrage |
| `journalctl -u svc -n 50 -f` | 50 dernières lignes puis suivi en direct |

## Redirections et recherche

| Syntaxe | Rôle |
| --- | --- |
| `cmd > f` / `cmd >> f` | Écrase / ajoute dans un fichier |
| `cmd 2> f` / `cmd 2>&1` | Redirige les erreurs / les joint à la sortie |
| `cmd1 \| cmd2` | Envoie la sortie de `cmd1` à `cmd2` |
| `grep -inrv motif cible` | Cherche un texte (casse, numéros, récursif, inversé) |
| `find . -name "*.log"` | Cherche des fichiers par nom, taille, date |

## Script

```bash
#!/bin/bash
set -e                    # s'arrête à la première erreur
NOM="${1:-monde}"         # argument 1, avec valeur par défaut
if commande; then echo ok; else exit 1; fi
for F in *.log; do echo "$F"; done
```

Code de retour : `0` = succès, `$?` = celui de la dernière commande. Lance avec `chmod +x s.sh && ./s.sh`, déboguer avec `bash -x s.sh`.

## SSH

| Commande | Rôle |
| --- | --- |
| `ssh user@hôte` / `ssh -p 2222 …` | Ouvre une session / sur un autre port |
| `ssh-keygen -t ed25519` | Crée une paire de clés (la privée reste chez toi) |
| `ssh-copy-id user@hôte` | Dépose la clé publique |
| `scp f user@hôte:/chemin` | Copie un fichier par SSH |
