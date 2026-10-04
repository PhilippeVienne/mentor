---
# ── Front matter d'un parcours ───────────────────────────────────────────────
# Les valeurs texte sont TOUJOURS entre guillemets (un « : » dans le texte casse le YAML sinon).
titre: "Parcours modèle"                     # obligatoire : nom affiché
icone: "🧪"                                  # obligatoire : un emoji (badge de fin de parcours)
resume: "Une phrase qui donne envie : ce que l'on apprend, en une ligne."   # obligatoire
moteur: git                                  # git | docker | (absent = pas de terminal, parcours sans labo)
prerequis: []                                # slugs (noms de dossier) des parcours à terminer avant
publie: true                                 # false = affiché « bientôt disponible », sans leçon
couleur: "#6366F1"                           # accent du parcours (facultatif)
banniere: images/banniere.svg                # bannière 1200×400 (facultatif)
---

<!-- Le corps de parcours.md est la présentation du parcours (page du parcours). Markdown + directives. -->

Présente ici le **public visé**, les **objectifs** et la **durée estimée**. Reste bref : l'apprenant·e veut commencer.

- Public : débutant·e·s en…
- Tu sauras faire : …
- Durée : environ 1 h
