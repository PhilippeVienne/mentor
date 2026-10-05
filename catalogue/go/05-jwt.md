---
id: jwt-authentification
title: "Authentification par JWT"
summary: "Vérifier qui se connecte grâce à un jeton signé (JWT) : signature, expiration et algorithme imposé, avec golang-jwt."
minutes: 45
objectives:
  - Décrire les trois parties d'un JWT et ce que sa signature garantit (et ne garantit pas)
  - Lire une structure qui en embarque une autre (`jwt.RegisteredClaims`)
  - Vérifier un jeton avec `golang-jwt` en imposant l'algorithme et l'expiration
  - Expliquer pourquoi le jeton d'un WebSocket arrive dans le premier message
---

Dans la leçon précédente, n'importe quelle personne dont le navigateur vient du bon site peut rejoindre le salon. Mais comment savoir *qui* est là, et si elle a le droit d'y être ? Quand tu te connectes dans l'équipe, un serveur d'identité (Keycloak) te remet un petit texte qui prouve qui tu es : un **jeton**. Cette leçon t'apprend à le vérifier.

## Qu'est-ce qu'un JWT ?

Un **JWT** (*JSON Web Token*, prononcé « jote ») est un jeton qui se compose de trois parties séparées par des points :

```text
eyJhbGciOi…  .  eyJuYW1lIjoi…  .  kuurgRmtty6q…
 en-tête         contenu            signature
```

- l'**en-tête** indique l'algorithme de signature (`HS256`, `RS256`…) ;
- le **contenu** (les *claims*, « revendications ») est un objet JSON qui contient des informations : le nom, les rôles, la date d'expiration `exp`…
- la **signature** est calculée à partir de l'en-tête, du contenu et d'une clé secrète. Si quelqu'un modifie le contenu, la signature ne correspond plus : le serveur le détecte.

:::warning Signé n'est pas chiffré
Le contenu d'un JWT est simplement encodé : n'importe qui peut le lire. La signature garantit qu'il n'a **pas été modifié** et qu'il vient de celui qui détient la clé, pas qu'il est secret. N'y mets jamais de mot de passe.
:::

Deux familles d'algorithmes existent. Avec **HS256**, la même clé secrète signe et vérifie : c'est le cas d'`event-planner-api`. Avec **RS256**, une clé privée signe et une clé publique vérifie : c'est le cas de `mgmt`, qui télécharge la clé publique de Keycloak au démarrage (fonction `InitAuth`) et refuse tout jeton dont l'algorithme n'est pas `RS256`. Dans cette leçon, on utilise HS256, plus simple à essayer.

## Décrire le contenu du jeton : une structure embarquée

On utilise la bibliothèque `github.com/golang-jwt/jwt/v5`. Commençons par décrire le contenu attendu :

```go
// Claims décrit le contenu du jeton : nos champs et les champs standards (exp, iss…).
type Claims struct {
	Nom   string   `json:"name"`
	Roles []string `json:"roles"`
	jwt.RegisteredClaims
}

func (c Claims) ARole(role string) bool {
	for _, r := range c.Roles {
		if r == role {
			return true
		}
	}
	return false
}
```

- `Nom` et `Roles` sont deux champs ordinaires. Leurs étiquettes `json:"name"` et `json:"roles"` (leçon 3) font le lien avec les noms utilisés dans le jeton. `[]string` est une slice de textes : la liste des rôles.
- La ligne `jwt.RegisteredClaims` n'a **pas de nom de champ**, seulement un type : on dit que la structure est **embarquée**. Les champs de `RegisteredClaims` (`ExpiresAt` pour la date d'expiration, `Issuer`, `Audience`…) deviennent alors utilisables directement sur `Claims`, comme s'ils étaient écrits dedans : on peut écrire `c.ExpiresAt`. C'est le moyen qu'a Go de « réutiliser » une structure sans héritage. `mgmt` fait pareil avec sa structure `KeycloakClaims`, qui embarque `jwt.StandardClaims` (l'ancien nom, dans l'ancienne version de la bibliothèque).
- `ARole` parcourt la liste des rôles avec une boucle `for` (leçon 1) et répond `true` dès qu'elle trouve le rôle cherché ; si la boucle se termine sans rien trouver, elle répond `false`.

## La clé secrète

```go
var secret []byte

func Initialise(cle []byte) error {
	if len(cle) == 0 {
		return errors.New("la clé est vide")
	}
	secret = cle
	return nil
}
```

`[]byte` est une suite d'octets : c'est la forme que la bibliothèque attend pour une clé. Le programme principal lit la clé dans une **variable d'environnement** (une valeur fournie au programme par le système au moment où on le lance) : `Initialise([]byte(os.Getenv("JWT_SECRET")))`. Si elle est vide, `Initialise` renvoie une erreur (leçon 2) et le programme **refuse de démarrer**.

:::danger Jamais de secret par défaut
Dans `event-planner-api`, `JwtMiddleware` et `CreateTokens` utilisent `utils.GetEnv("JWT_SECRET", "secret123456")` : si la variable est absente, le serveur signe avec un secret connu de tous (il est écrit dans le code). Notre exemple fait le choix inverse : pas de secret, pas de démarrage. Dans ce parcours, les clés des exercices sont factices et ne servent qu'aux tests.
:::

## Vérifier un jeton

```go
func Verifie(brut string) (*Claims, error) {
	claims := &Claims{}
	_, err := jwt.ParseWithClaims(brut, claims, func(t *jwt.Token) (any, error) {
		return secret, nil
	}, jwt.WithValidMethods([]string{"HS256"}), jwt.WithExpirationRequired())
	if err != nil {
		return nil, err
	}
	return claims, nil
}
```

Cette fonction reçoit le jeton « brut » (le texte à trois parties) et renvoie son contenu, ou une erreur. Ligne à ligne :

- `claims := &Claims{}` fabrique une structure vide et en garde l'adresse (le `&` de la leçon 2) : la bibliothèque va la **remplir** avec le contenu du jeton.
- `jwt.ParseWithClaims(brut, claims, …)` décode le jeton, **recalcule la signature** et la compare. Elle renvoie deux résultats : le jeton décodé (dont on n'a pas besoin ici, d'où le `_`) et une erreur.
- Le troisième argument, `func(t *jwt.Token) (any, error) { return secret, nil }`, est une **fonction anonyme** (leçon 3) que la bibliothèque **appelle elle-même** pour savoir quelle clé utiliser pour ce jeton (`any` veut dire « n'importe quel type »). Ici, elle renvoie toujours `secret` ; c'est une closure qui utilise la variable `secret` de l'entourage. Avec RS256, elle choisirait la clé publique qui correspond au jeton.
- Les arguments suivants sont des **options**, que `ParseWithClaims` accepte en nombre variable (le paramètre variadique `...` de la leçon 2). `jwt.WithValidMethods([]string{"HS256"})` impose l'algorithme. C'est capital : sans cette option, un attaquant pourrait fabriquer un jeton annonçant un autre algorithme. `mgmt` le fait à la main avec `token.Method.(*jwt.SigningMethodRSA)`, puis compare `token.Header["alg"]` à `"RS256"`.
- `jwt.WithExpirationRequired()` refuse un jeton qui n'a pas de date d'expiration. Un jeton expiré, lui, est toujours refusé.
- Enfin, si `err` n'est pas `nil`, on renvoie `nil` (pas de contenu) et l'erreur ; sinon on renvoie le contenu rempli.

Pour tester, il faut aussi pouvoir **fabriquer** un jeton. Le labo fournit la fonction `Signe(nom, roles, duree)` qui fait l'inverse : `jwt.NewWithClaims(jwt.SigningMethodHS256, claims).SignedString(secret)` assemble les trois parties et signe.

## Brancher la vérification sur le WebSocket

Un navigateur **ne peut pas ajouter d'en-tête** `Authorization` à un WebSocket. La solution de `mgmt` (fonction `handleWs`) est donc de lire le **premier message** de la connexion comme un jeton. Dans le `Ws` de la leçon 4, cela donnerait, juste après l'`Upgrade` :

```go
_, premier, err := conn.ReadMessage()
if err != nil {
	return
}
claims, err := Verifie(strings.TrimSpace(string(premier)))
if err != nil {
	conn.WriteJSON(map[string]string{"type": "error", "data": "jeton refusé"})
	return
}
s.Ajoute(conn, claims.Nom)
```

`string(premier)` convertit les octets en texte, `strings.TrimSpace` retire les espaces et retours à la ligne autour. Si le jeton est refusé, on prévient le client en JSON et on ferme (le `return` déclenche les `defer`). Sinon, le nom vient désormais **du jeton**, et non plus de l'adresse : on ne peut plus se faire passer pour quelqu'un d'autre.

## Entraîne-toi

:::lab
engine: real
intro: |
  Tu écris le module de vérification des jetons. `claims.go` (la structure `Claims`) et `signe.go` (la fabrication d'un jeton) sont déjà écrits. À toi d'écrire `Initialise` (`secret.go`), `ARole` (`role.go`) et `Verifie` (`verifie.go`), en trois temps pour `Verifie`. Les tests sont dans `jetons_test.go` et utilisent une clé factice. La bibliothèque `golang-jwt` est déjà installée dans l'environnement. Les tests fournis sont vérifiés tels quels.
commands:
  - cp -R /opt/exercices/05-jwt/. .
  - go vet ./... >/dev/null 2>&1 || true
steps:
  - text: 'Dans `secret.go`, écris `Initialise` : elle refuse une clé vide (erreur) et enregistre sinon la clé dans `secret`. `TestInitialise` doit passer'
    hint: 'Teste `len(cle) == 0` et renvoie `errors.New("la clé est vide")` (import `errors`) ; sinon `secret = cle` puis `return nil`.'
    checks:
      - command-succeeds: "verifier-go tests 05-jwt . TestInitialise"
    solution:
      - |
        cat > secret.go <<'EOF'
        package main

        import "errors"

        // secret est la clé qui signe et vérifie les jetons (algorithme HS256).
        var secret []byte

        // Initialise enregistre la clé. Elle refuse une clé vide.
        func Initialise(cle []byte) error {
        	if len(cle) == 0 {
        		return errors.New("la clé est vide")
        	}
        	secret = cle
        	return nil
        }
        EOF
  - text: 'Dans `role.go`, écris `ARole` : elle renvoie `true` si le rôle est dans `c.Roles`, `false` sinon. `TestARole` doit passer'
    hint: 'Boucle `for _, r := range c.Roles` ; `if r == role { return true }` ; `return false` après la boucle.'
    checks:
      - command-succeeds: "verifier-go tests 05-jwt . TestARole"
    solution:
      - |
        cat > role.go <<'EOF'
        package main

        // ARole dit si la liste Roles du jeton contient ce rôle.
        func (c Claims) ARole(role string) bool {
        	for _, r := range c.Roles {
        		if r == role {
        			return true
        		}
        	}
        	return false
        }
        EOF
  - text: 'Dans `verifie.go`, écris une première version de `Verifie` avec `jwt.ParseWithClaims` et une fonction anonyme qui renvoie `secret`. `TestVerifieValide` doit passer : un jeton valide est accepté, un jeton signé avec une autre clé est refusé'
    hint: 'Reprends le code de la leçon sans les deux options à la fin. Imports : `github.com/golang-jwt/jwt/v5`.'
    after: [1]
    checks:
      - command-succeeds: "verifier-go tests 05-jwt . TestVerifieValide"
    solution:
      - |
        cat > verifie.go <<'EOF'
        package main

        import "github.com/golang-jwt/jwt/v5"

        // Verifie décode un jeton brut et renvoie son contenu (première version : signature seulement).
        func Verifie(brut string) (*Claims, error) {
        	claims := &Claims{}
        	_, err := jwt.ParseWithClaims(brut, claims, func(t *jwt.Token) (any, error) {
        		return secret, nil
        	})
        	if err != nil {
        		return nil, err
        	}
        	return claims, nil
        }
        EOF
  - text: 'Exige une date d''expiration : ajoute l''option `jwt.WithExpirationRequired()` à `ParseWithClaims`. `TestVerifieSansExpiration` et `TestVerifieExpire` doivent passer'
    hint: 'Ajoute-la après la fonction anonyme, séparée par une virgule : `…}, jwt.WithExpirationRequired())`.'
    after: [3]
    checks:
      - command-succeeds: "verifier-go tests 05-jwt . TestVerifieSansExpiration TestVerifieExpire"
    solution:
      - |
        cat > verifie.go <<'EOF'
        package main

        import "github.com/golang-jwt/jwt/v5"

        // Verifie décode un jeton brut et renvoie son contenu (signature et expiration).
        func Verifie(brut string) (*Claims, error) {
        	claims := &Claims{}
        	_, err := jwt.ParseWithClaims(brut, claims, func(t *jwt.Token) (any, error) {
        		return secret, nil
        	}, jwt.WithExpirationRequired())
        	if err != nil {
        		return nil, err
        	}
        	return claims, nil
        }
        EOF
  - text: 'Impose l''algorithme : ajoute `jwt.WithValidMethods([]string{"HS256"})`. Un jeton signé avec la même clé mais en HS512 doit être refusé. `TestVerifieAlgorithme` doit passer'
    hint: 'Une deuxième option, après la première, séparée par une virgule.'
    after: [4]
    checks:
      - command-succeeds: "verifier-go tests 05-jwt . TestVerifieAlgorithme"
    solution:
      - |
        cat > verifie.go <<'EOF'
        package main

        import "github.com/golang-jwt/jwt/v5"

        // Verifie décode un jeton brut et renvoie son contenu (signature, expiration et algorithme).
        func Verifie(brut string) (*Claims, error) {
        	claims := &Claims{}
        	_, err := jwt.ParseWithClaims(brut, claims, func(t *jwt.Token) (any, error) {
        		return secret, nil
        	}, jwt.WithValidMethods([]string{"HS256"}), jwt.WithExpirationRequired())
        	if err != nil {
        		return nil, err
        	}
        	return claims, nil
        }
        EOF
  - text: 'Compile le programme avec `go build -o jetons .`, puis lance-le avec une clé factice : `JWT_SECRET=cle-factice ./jetons` affiche « jeton accepté pour Camille », alors que `./jetons` sans la variable échoue. `go vet ./...` et `go test ./...` passent aussi'
    hint: 'Tout est déjà écrit dans `main.go`. Essaie `go build -o jetons .`, puis `JWT_SECRET=cle-factice ./jetons`, puis `./jetons`, et enfin `go vet ./... && go test ./...`.'
    after: [1, 2, 3, 4, 5]
    checks:
      - command-succeeds: 'verifier-go binaire 05-jwt jetons'
      - command-succeeds: "verifier-go lancer 05-jwt 'jeton accepté pour Camille - bureau : true' JWT_SECRET=cle-factice"
      - command-succeeds: 'verifier-go lancer-echoue 05-jwt'
      - command-succeeds: 'verifier-go tout 05-jwt'
    solution:
      - go build -o jetons .
      - JWT_SECRET=cle-factice ./jetons
:::

## Vérifie tes acquis

:::quiz
Que garantit la signature d'un JWT ?

- [ ] Que son contenu est confidentiel
- [x] Que son contenu n'a pas été modifié et qu'il a été signé par le détenteur de la clé
- [ ] Que la personne est majeure
- [ ] Que le jeton n'expire jamais

> Le contenu d'un JWT est lisible par tous ; la signature permet seulement de détecter une modification.
:::

:::quiz
Pourquoi écrit-on `jwt.WithValidMethods([]string{"HS256"})` ?

- [ ] Pour accélérer la lecture du jeton
- [ ] Pour que le jeton soit chiffré
- [ ] Pour autoriser plusieurs secrets
- [x] Pour refuser un jeton signé avec un autre algorithme que celui qu'on attend

> Il faut imposer l'algorithme côté serveur au lieu de faire confiance à celui annoncé dans le jeton.
:::

:::quiz
Dans `type Claims struct { Nom string; jwt.RegisteredClaims }`, que signifie la ligne `jwt.RegisteredClaims` sans nom de champ ?

- [ ] Que le champ est privé
- [ ] Que le champ est facultatif dans le jeton
- [x] Que la structure est embarquée : ses champs (comme `ExpiresAt`) s'utilisent directement sur `Claims`
- [ ] Que `Claims` est une interface

> L'embarquement permet de réutiliser une structure sans héritage : `c.ExpiresAt` fonctionne comme si le champ était écrit dans `Claims`.
:::

:::quiz
Pourquoi le jeton est-il envoyé dans le premier message du WebSocket, dans le code de `mgmt` ?

- [ ] Parce que le protocole WebSocket interdit les en-têtes
- [ ] Parce que Keycloak l'exige
- [x] Parce qu'un navigateur ne permet pas d'ajouter un en-tête `Authorization` à un WebSocket
- [ ] Parce que Go ne sait pas lire les en-têtes d'une requête upgrade

> L'API WebSocket des navigateurs n'offre pas cette possibilité ; on passe donc le jeton dans un message (ou l'URL).
:::
