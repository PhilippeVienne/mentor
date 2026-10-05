---
# ── Front matter d'un parcours ───────────────────────────────────────────────
# Les valeurs texte sont TOUJOURS entre guillemets (un « : » dans le texte casse le YAML sinon).
title: "Parcours modèle"                     # obligatoire : nom affiché
icon: "🧪"                                   # obligatoire : un emoji (badge de fin de parcours)
summary: "Une phrase qui donne envie : ce que l'on apprend, en une ligne."  # obligatoire
engine: git                                  # git | docker | (absent = pas de terminal, parcours sans labo)
requires: []                                 # slugs (noms de dossier) des parcours à terminer avant
published: true                              # false = affiché « bientôt disponible », sans leçon
color: "#6366F1"                             # accent du parcours (facultatif)
banner: images/banniere.svg                  # bannière 1200×400 (facultatif)
---

<!-- Le corps de parcours.md est la présentation du parcours (page du parcours). Markdown + directives. -->

Présente ici le **public visé**, les **objectifs** et la **durée estimée**. Reste bref : l'apprenant·e veut commencer.

- Public : débutant·e·s en…
- Tu sauras faire : …
- Durée : environ 1 h
