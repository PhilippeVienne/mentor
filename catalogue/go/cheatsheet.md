## Commandes de l'outil `go`

| Commande | Rôle |
| --- | --- |
| `go version` | Affiche la version de Go installée |
| `go mod init nom` | Crée un module (`go.mod`) |
| `go mod tidy` | Ajoute les dépendances utilisées, retire les inutiles |
| `go run .` | Compile le dossier courant et l'exécute |
| `go build -o nom` | Compile en un fichier exécutable |
| `go test ./...` | Lance tous les tests du module |
| `go vet ./...` | Cherche des constructions suspectes |
| `go generate ./ent` | Lance les générateurs (code Ent) |

## Syntaxe essentielle

| Besoin | Écriture |
| --- | --- |
| Variable avec type deviné | `x := 3` |
| Valeur zéro | `""`, `0`, `false`, `nil` |
| Structure | `type Jeu struct { Titre string }` |
| Liste (slice) | `jeux := []Jeu{}` puis `append(jeux, j)` |
| Table (map) | `m := map[string]Jeu{}` ; `v, ok := m["clé"]` |
| Méthode (copie) | `func (j Jeu) Nom() string` |
| Méthode (modifie l'original) | `func (j *Jeu) Emprunter()` |
| Exporté (visible hors du paquet) | Nom qui commence par une **majuscule** |
| Piège `range` | `for _, j := range l` copie : utiliser `l[i]` |
| Formater un texte | `fmt.Sprintf("%s a %d ans", nom, age)` : `%s` texte, `%d` entier, `%q` texte entre guillemets, `%v` n'importe quoi, `%w` erreur enveloppée |
| Paramètre variadique | `func f(titres ...string)` : autant de titres qu'on veut, reçus comme une slice |
| Adresse d'une structure | `s := &Stock{}` : `s` est un pointeur (`*Stock`) |
| Fonction anonyme | `f := func(x int) int { return x * 2 }` |
| Closure | fonction anonyme qui garde les variables de son entourage (`journal`, `compteur`) |

## Erreurs

```go
var ErrIntrouvable = errors.New("introuvable")

return fmt.Errorf("jeu %d : %w", id, ErrIntrouvable) // envelopper

if errors.Is(err, ErrIntrouvable) { /* repère */ }
var q *ErreurQuota
if errors.As(err, &q) { /* type d'erreur */ }
```

`panic` est réservé aux erreurs de programmation, pas aux erreurs courantes.

## HTTP et JSON

```go
mux := http.NewServeMux()
mux.HandleFunc("GET /jeux", liste) // motifs avec méthode : Go 1.22 ou plus
json.NewDecoder(r.Body).Decode(&j) // lire
json.NewEncoder(w).Encode(j)       // écrire
```

| Code | Sens |
| --- | --- |
| `200` / `201` | Réussi / créé |
| `400` | Requête invalide |
| `401` / `403` | Non authentifié / interdit |
| `404` / `405` | Introuvable / méthode non permise |
| `500` | Panne du serveur |

Étiquette de champ : `` `json:"nom,omitempty"` ``. Plusieurs requêtes en parallèle : protéger les données partagées avec `sync.Mutex` (`Lock`, puis `defer Unlock`). Un middleware est une fonction `func(http.Handler) http.Handler` qui renvoie une closure `http.HandlerFunc(func(w, r) { … })`.

## WebSocket

- Un WebSocket garde une connexion ouverte : `Upgrader.Upgrade`, `ReadMessage`, `WriteMessage`.
- Le navigateur joint l'en-tête `Origin` (schéma, domaine, port du site de la page). Le CORS ne protège **pas** les WebSocket : écrire `CheckOrigin` et comparer `r.Header.Get("Origin")` à l'origine attendue.
- Une seule écriture à la fois par connexion, et une map partagée entre goroutines se protège avec un mutex.

## JWT

- Un JWT : `en-tête.contenu.signature`. Signé, **pas chiffré**. HS256 : une clé secrète partagée ; RS256 : clé privée pour signer, publique pour vérifier.
- `jwt.ParseWithClaims(brut, &claims, func(t *jwt.Token) (any, error) { return cle, nil }, options…)` : toujours imposer l'algorithme (`jwt.WithValidMethods`) et exiger l'expiration (`jwt.WithExpirationRequired()`).
- Structure embarquée : `jwt.RegisteredClaims` sans nom de champ donne accès à `ExpiresAt`, etc.
- Jamais de secret par défaut codé en dur : pas de `JWT_SECRET`, pas de démarrage.

## Base de données

- Cacher la base derrière une **interface** ; passer un `context.Context` en premier argument.
- Ent : schéma dans `ent/schema/`, code généré par `go generate`, requête `client.X.Query().Where(…).Only(ctx)`, `ent.IsNotFound(err)`.
- Secrets de connexion dans des variables d'environnement.

## Tests et image Docker

```bash
go test -v ./...
```

Fichier `xxx_test.go`, fonction `TestXxx(t *testing.T)`, `t.Run` pour les sous-tests, `httptest.NewRequest` et `httptest.NewRecorder` pour les handlers.

```dockerfile
FROM golang:1.25 AS build
RUN CGO_ENABLED=0 go build -o /app
FROM alpine:3.22
COPY --from=build /app /app
USER 1000
ENTRYPOINT ["/app"]
```

## Versions rencontrées dans l'équipe

`mgmt` : Go 1.16, Macaron 1.3.5 · `billetterie` : Go 1.14, Macaron 1.3.8, MongoDB, ancien outil `dep` (`Gopkg.*`) · `event-planner-api` : Go 1.23, Echo 4, Ent, PostgreSQL.
