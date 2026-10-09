---
title: "OIDC avec Keycloak"
icon: "🔐"
summary: "Authentification OpenID Connect avec le SSO de l'équipe : flux authorization code, clients, realms. Disponible prochainement."
requires: [django]
published: false
color: "#4D8FCC"
banner: images/banniere.svg
---

Pour se connecter aux applications des associations, les étudiant·e·s utilisent un compte unique : le **SSO de l'équipe**, propulsé par [Keycloak](https://www.keycloak.org/) et accessible sur `sso.example.org`. Ce cours explique comment ça marche et comment brancher une application Django dessus avec **OpenID Connect (OIDC)**. Ce portail de formation utilise lui-même ce mécanisme.

:::info Cours en préparation
Ce cours n'est pas encore publié. Il sera débloqué une fois le cours *Django* terminé.
:::

## À qui s'adresse-t-il ?

Aux développeur·se·s qui connaissent Django et doivent protéger une application ou une API avec le SSO de l'équipe.

## Plan prévisionnel

1. Authentification, autorisation, SSO : le vocabulaire
2. OAuth 2.0 et OpenID Connect en un schéma (flux *authorization code*)
3. Keycloak : realm, client, utilisateurs, rôles
4. Brancher Django avec [`mozilla-django-oidc`](https://github.com/mozilla/mozilla-django-oidc)
5. Comprendre les jetons (ID token, access token, JWT)
6. Protéger une API et gérer la déconnexion
7. Déboguer : redirections, `redirect_uri`, claims manquants

## Ressources utilisées par l'équipe

- [`django-keycloak`](https://gitlab.example.org/equipe/dev/django-keycloak) : application Django maintenue par l'équipe
- [`mozilla-django-oidc`](https://github.com/mozilla/mozilla-django-oidc) : client OIDC utilisé par l'API d'Adhésion
- [`keycloak-theme`](https://gitlab.example.org/equipe/utils/keycloak-theme) : thème du SSO de l'équipe

**Prérequis :** cours *Django*. **Durée estimée :** environ 3 h.
