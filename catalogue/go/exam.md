---
title: "Examen de validation — Go"
draw: 5
pass_mark: 80
minutes: 10
shuffle: true
---

Cet examen valide les bases de Go vues dans le cours : syntaxe, erreurs et interfaces, serveur HTTP et JSON, WebSocket, JWT, base de données, tests et image Docker.

:::quiz
Tu veux livrer un service Go sur un serveur qui n'a rien d'installé. Quel avantage de Go t'aide ici ?

- [ ] Le programme est interprété au démarrage par le serveur
- [ ] Le serveur doit seulement avoir un navigateur récent
- [x] La compilation produit un exécutable autonome
- [ ] Go télécharge ses bibliothèques à chaque exécution

> Un langage compilé fournit un fichier exécutable qui se lance sans traducteur.
:::

:::quiz
Que fait la commande `go mod init ludotheque` ?

- [x] Elle crée le fichier `go.mod` qui identifie le module
- [ ] Elle installe Go et ses outils sur l'ordinateur de développement
- [ ] Elle compile puis lance le programme du dossier courant
- [ ] Elle télécharge toutes les dépendances du dépôt

> `go.mod` porte le nom du module, la version de Go visée et les dépendances.
:::

:::quiz
Après `var n int`, sans autre instruction, quelle est la valeur de `n` ?

- [ ] `nil`
- [ ] Une valeur indéterminée
- [ ] Une erreur de compilation
- [x] 0

> Toute variable reçoit la valeur zéro de son type.
:::

:::quiz
Que se passe-t-il avec `for _, j := range jeux { j.Titre = "x" }` ?

- [ ] Tous les jeux de la liste sont modifiés, un par un, sur place
- [x] Seule une copie de chaque jeu est modifiée, la liste ne change pas
- [ ] La boucle provoque une erreur à la compilation
- [ ] Seul le premier jeu de la liste est modifié, puis la boucle s'arrête

> `j` est une copie : il faut écrire `jeux[i].Titre = "x"`.
:::

:::quiz
Une méthode doit modifier la structure sur laquelle elle est appelée. Quel récepteur choisis-tu ?

- [ ] Une valeur, comme `(j Jeu)`
- [ ] Une interface, comme `(j Preteur)`
- [x] Un pointeur, comme `(j *Jeu)`
- [ ] Une map, comme `(j map[string]Jeu)`

> Un récepteur valeur reçoit une copie ; seul un pointeur modifie l'original.
:::

:::quiz
Dans un paquet, quel identifiant est utilisable depuis un autre paquet ?

- [ ] `charger`, parce qu'il est en minuscule
- [x] `Charger`, parce qu'il commence par une majuscule
- [ ] Tous les identifiants, dès que le paquet qui les contient est importé
- [ ] Seuls ceux déclarés avec un mot-clé `public` placé avant leur nom

> En Go, la visibilité dépend uniquement de la majuscule initiale.
:::

:::quiz
Pourquoi écrit-on `fmt.Errorf("lecture : %w", err)` plutôt que `%v` ?

- [x] Pour que `errors.Is` retrouve encore l'erreur d'origine
- [ ] Pour que le message d'erreur s'affiche avec une mise en forme spéciale
- [ ] Parce que `%v` est interdit avec les erreurs par le compilateur
- [ ] Pour que l'erreur soit ignorée par `err != nil`

> `%w` enveloppe l'erreur ; `%v` ne garde que son texte.
:::

:::quiz
Quelle fonction cherche, dans une chaîne d'erreurs, une erreur d'un type précis pour lire ses champs ?

- [ ] `errors.Is`
- [ ] `errors.New`
- [ ] `errors.Join`
- [x] `errors.As`

> `errors.Is` compare à une valeur, `errors.As` extrait une erreur typée.
:::

:::quiz
Un type possède toutes les méthodes d'une interface. Que faut-il faire pour l'utiliser comme telle ?

- [ ] Écrire le mot-clé `implements` suivi de l'interface après le nom du type
- [x] Rien de plus, l'interface est satisfaite implicitement
- [ ] Déclarer le type dans le même fichier que l'interface
- [ ] Enregistrer le type auprès de l'interface dans une fonction `init()`

> Go vérifie seulement la présence des méthodes.
:::

:::quiz
Quel code de statut HTTP convient pour un JSON invalide envoyé par un client ?

- [ ] 200
- [ ] 500
- [x] 400
- [ ] 404

> `400` signale une requête mal formée ; `500` serait réservé à une panne du serveur.
:::

:::quiz
Dans un handler, à quel moment appelle-t-on `w.WriteHeader(http.StatusCreated)` ?

- [x] Avant d'écrire le corps de la réponse
- [ ] Après avoir fermé la connexion
- [ ] Dans le middleware, jamais dans le handler
- [ ] Après avoir écrit le corps de la réponse

> Une fois le corps commencé, le statut est déjà parti avec la valeur `200`.
:::

:::quiz
Pourquoi les handlers d'un serveur HTTP Go doivent-ils protéger les données partagées ?

- [ ] Go exécute les requêtes une par une, mais dans un ordre imprévisible
- [ ] Les handlers n'ont pas le droit de lire des variables globales
- [ ] Les données sont copiées à chaque requête
- [x] Chaque requête s'exécute dans sa propre goroutine, en parallèle des autres

> Sans mutex, deux requêtes qui modifient la même donnée peuvent la corrompre.
:::

:::quiz
Que fait un middleware comme `AuthenticationMiddleware` dans `mgmt` ?

- [ ] Il remplace le routeur et choisit la route de l'application
- [x] Il vérifie le jeton avant d'appeler le traitement de la route
- [ ] Il génère les jetons des utilisateurs·rices à chaque connexion
- [ ] Il compile les fichiers du projet

> Un middleware s'exécute avant (ou après) le handler pour y ajouter un contrôle commun.
:::

:::quiz
Quelle partie d'un JWT permet de détecter qu'il a été modifié ?

- [ ] L'en-tête seul
- [ ] Le champ `exp`
- [x] La signature
- [ ] Le nom du serveur qui l'a reçu

> La signature est recalculée avec la clé ; si le contenu change, elle ne correspond plus.
:::

:::quiz
Pourquoi un JWT ne doit-il jamais contenir de mot de passe ?

- [x] Son contenu est lisible par quiconque le possède, il n'est que signé
- [ ] Il est trop court pour contenir du texte, quelle que soit sa taille
- [ ] Il est chiffré, mais avec une clé de chiffrement trop faible
- [ ] Les mots de passe ne s'écrivent pas en JSON

> Signature et chiffrement sont deux choses différentes.
:::

:::quiz
`mgmt` vérifie `token.Header["alg"] == "RS256"`. Quel risque évite-t-il ?

- [ ] Un jeton trop volumineux pour passer dans un message réseau
- [x] Accepter un jeton signé avec un autre algorithme que celui prévu
- [ ] Un jeton qui n'a pas de nom
- [ ] Un navigateur qui bloque la connexion à cause d'une origine inconnue

> Il faut imposer l'algorithme côté serveur au lieu de croire celui annoncé par le jeton.
:::

:::quiz
Pourquoi `mgmt` attend-il le jeton dans le premier message du WebSocket ?

- [ ] Le protocole WebSocket interdit d'envoyer des en-têtes HTTP
- [ ] Le serveur Go ne sait pas lire les en-têtes d'une requête upgrade
- [ ] Les jetons sont trop longs pour tenir dans un en-tête HTTP
- [x] Un navigateur ne peut pas ajouter d'en-tête `Authorization` à un WebSocket

> L'API WebSocket du navigateur n'offre pas cette possibilité.
:::

:::quiz
Dans `event-planner-api`, quel outil décrit les tables et génère le code d'accès à PostgreSQL ?

- [ ] Macaron
- [ ] Echo
- [x] Ent
- [ ] Atlas

> Les schémas Ent sont dans `ent/schema/`, le code est généré par `go generate`.
:::

:::quiz
À quoi sert un `context.Context` passé à une méthode de dépôt ?

- [x] À annuler l'opération ou à lui imposer un délai
- [ ] À choisir la base de données que le dépôt va utiliser
- [ ] À stocker un mot de passe
- [ ] À convertir automatiquement les résultats de la requête en JSON

> Le contexte transporte annulation et échéance.
:::

:::quiz
Que renvoie `Only(ctx)` d'une requête Ent quand aucune ligne ne correspond ?

- [ ] Une ligne vide, sans aucune erreur, que l'appelant doit tester
- [x] Une erreur « introuvable », que `ent.IsNotFound` reconnaît
- [ ] La première ligne de la table, même si elle ne correspond pas
- [ ] Rien : le programme s'arrête

> `Only` exige exactement une ligne.
:::

:::quiz
Pourquoi `mgmt` perd-il l'état de ses connexions quand on le redémarre ?

- [ ] Go efface lui-même les variables globales à l'arrêt du programme
- [ ] Keycloak réinitialise les jetons
- [x] Cet état n'existe qu'en mémoire, il n'est enregistré dans aucune base
- [ ] Docker supprime les fichiers du conteneur à chaque redémarrage

> Une base de données (ou un fichier) est nécessaire pour survivre à un redémarrage.
:::

:::quiz
Comment s'appelle un fichier de test que `go test` reconnaît ?

- [ ] `test_service.go`
- [x] `service_test.go`
- [ ] `service.test`
- [ ] `tests/service.go`

> Le suffixe `_test.go` est obligatoire.
:::

:::quiz
Dans un test de handler, que fournit `httptest.NewRecorder()` ?

- [ ] Un vrai serveur réseau lancé sur un port aléatoire
- [ ] Une base de données de test
- [ ] Un client HTTP prêt à l'emploi pour interroger le serveur
- [x] Un `ResponseWriter` qui conserve le statut et le corps écrits

> Il remplace `w` pour inspecter la réponse sans réseau.
:::

:::quiz
Que lit `go vet ./...` ?

- [x] Le code source, pour y repérer des constructions suspectes sans l'exécuter
- [ ] Les journaux du serveur en production, pour y détecter des erreurs
- [ ] Uniquement les fichiers de test, ceux qui se terminent par `_test.go`
- [ ] Les images Docker construites

> C'est une analyse statique.
:::

:::quiz
Dans un Dockerfile en deux étapes, que contient l'image finale ?

- [ ] Le compilateur Go, les sources et les dépendances téléchargées
- [x] Le programme compilé copié depuis la première étape
- [ ] Toutes les couches de l'étape de compilation, caches compris
- [ ] Uniquement le fichier `go.mod`

> `COPY --from=build` ne récupère que ce qu'on lui demande.
:::

:::quiz
Dans le Dockerfile de `mgmt`, le second étage `alpine` est commenté. Quelle conséquence ?

- [ ] Le programme ne compile plus du tout dans l'image
- [ ] Le programme s'exécute sous Alpine
- [x] L'image livrée contient toute la chaîne d'outils Go, donc elle est lourde
- [ ] Les tests sont lancés automatiquement à la construction

> Sans second étage, l'image finale est l'image de compilation.
:::

:::quiz
Une fonction `somme(valeurs ...int)` est appelée avec `somme(1, 2, 3)`. Que reçoit-elle dans `valeurs` ?

- [ ] Une erreur, car il faut passer une seule valeur
- [ ] Seulement le premier nombre
- [x] Une slice qui contient 1, 2 et 3
- [ ] Un pointeur vers le nombre 3

> Un paramètre variadique est vu comme une slice du type indiqué.
:::

:::quiz
Quel texte produit `fmt.Sprintf("%s : %d jeux", "Camille", 2)` ?

- [ ] `%s : %d jeux`
- [ ] `Camille : %d jeux`
- [x] `Camille : 2 jeux`
- [ ] Rien : `Sprintf` affiche seulement le résultat dans le terminal

> `%s` reçoit le texte, `%d` l'entier, et `Sprintf` renvoie le résultat sans l'afficher.
:::

:::quiz
Un middleware retourne `http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { suivant.ServeHTTP(w, r) })`. Pourquoi la fonction anonyme peut-elle utiliser `suivant` ?

- [ ] Parce que `suivant` est une variable globale du paquet `http`
- [ ] Parce que Go passe automatiquement le paramètre `suivant` à chaque requête
- [ ] Parce que le routeur renseigne `suivant` avant chaque appel
- [x] Parce qu'une closure garde en mémoire les variables de la fonction qui l'a créée

> La fonction anonyme est créée dans `journal` et se souvient de son paramètre, même après le retour de `journal`.
:::

:::quiz
Pourquoi le serveur WebSocket doit-il comparer lui-même l'en-tête `Origin` à une liste de sites autorisés ?

- [ ] Parce que le navigateur n'envoie jamais cet en-tête
- [x] Parce que le CORS du navigateur ne limite pas les WebSocket, et qu'un site tiers pourrait sinon se connecter
- [ ] Parce que l'en-tête `Origin` contient le jeton JWT de la personne
- [ ] Parce que Go refuse d'ouvrir un WebSocket sans cette comparaison

> Sans contrôle de l'origine, n'importe quelle page ouverte dans le navigateur pourrait ouvrir la connexion.
:::
