#!/usr/bin/env node
'use strict';
/*
 * verifier-page : ce qu'un navigateur ferait d'une page, sans navigateur.
 *
 * Le conteneur du labo n'a ni navigateur ni réseau. Cet outil charge la page avec jsdom (HTML + feuilles de style
 * + scripts locaux), affiche un diagnostic, puis joue dans l'ordre les actions et les contrôles demandés.
 * Il vérifie le RÉSULTAT (le DOM obtenu), jamais le texte du code.
 *
 * Usage : verifier-page page.html [options]
 *
 * Actions (jouées dans l'ordre, la page a ~50 ms pour réagir après chacune) :
 *   --clic SEL            clique sur l'élément
 *   --clics SEL N         clique N fois
 *   --saisie SEL TEXTE    écrit TEXTE dans un champ
 *   --envoyer SEL         envoie le formulaire (échoue si la page ne l'intercepte pas : elle se rechargerait)
 *   --attendre MS         attend MS millisecondes
 * Réglages (valables pour tout l'examen de la page) :
 *   --largeur PX          simule une fenêtre de PX pixels de large (pour les media queries min/max-width)
 *   --erreur CODE         le faux serveur répond CODE (500, 404…) à toute requête
 *   --panne               le faux serveur est injoignable (erreur réseau)
 *   --api DOSSIER         dossier des réponses du faux serveur (défaut : api-fictive)
 * Contrôles (code de sortie 1 si l'un échoue) :
 *   --existe SEL | --absent SEL | --vide SEL
 *   --contient SEL TEXTE  le texte de l'élément contient TEXTE (sans tenir compte des majuscules ni des espaces en trop)
 *   --egal SEL TEXTE      le texte de l'élément est exactement TEXTE
 *   --style SEL PROP VAL  valeur calculée de la propriété CSS
 *   --appelle METHODE CHEMIN   la page a envoyé cette requête au faux serveur
 *   --corps REGEX         le corps de la dernière requête envoyée correspond à REGEX
 *   --entete NOM VALEUR   la dernière requête envoyée porte cet en-tête (sans tenir compte de la casse du nom)
 *   --accessible          contrôles d'accessibilité de base (langue, titres, images, libellés)
 *
 * Le faux « fetch » : fetch("https://example.org/api/evenements") lit api-fictive/evenements.json ;
 * un fichier absent donne 404 ; un POST donne 201 et renvoie le corps reçu.
 */
const fs = require('node:fs');
const path = require('node:path');
const { JSDOM, VirtualConsole } = require('jsdom');

const sommeil = (ms) => new Promise((r) => setTimeout(r, ms));
const norme = (t) => String(t).replace(/[’‘]/g, "'").replace(/\s+/g, ' ').trim().toLowerCase();

// ── Lecture des arguments ───────────────────────────────────────────────────
const ARITE = {
  '--clic': 1, '--clics': 2, '--saisie': 2, '--envoyer': 1, '--attendre': 1,
  '--existe': 1, '--absent': 1, '--vide': 1, '--contient': 2, '--egal': 2, '--style': 3,
  '--appelle': 2, '--corps': 1, '--entete': 2, '--accessible': 0,
};
const REGLAGES = { '--largeur': 1, '--erreur': 1, '--panne': 0, '--api': 1 };

function lireArguments(argv) {
  const cfg = { fichier: null, ops: [], largeur: 1280, erreur: null, panne: false, api: 'api-fictive' };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a in ARITE) {
      const args = argv.slice(i + 1, i + 1 + ARITE[a]);
      if (args.length < ARITE[a]) fin(2, `Il manque un argument après ${a}.`);
      cfg.ops.push({ op: a.slice(2), args });
      i += ARITE[a];
    } else if (a in REGLAGES) {
      const v = REGLAGES[a] ? argv[++i] : true;
      if (v === undefined) fin(2, `Il manque un argument après ${a}.`);
      if (a === '--largeur') cfg.largeur = Number(v);
      if (a === '--erreur') cfg.erreur = Number(v);
      if (a === '--panne') cfg.panne = true;
      if (a === '--api') cfg.api = v;
    } else if (!a.startsWith('--') && !cfg.fichier) {
      cfg.fichier = a;
    } else {
      fin(2, `Option inconnue : ${a}`);
    }
  }
  if (!cfg.fichier) fin(2, 'Usage : verifier-page page.html [options] (voir l\'en-tête de /opt/outils/verifier-page.js)');
  return cfg;
}

function fin(code, message) {
  if (message) (code ? console.error : console.log)(message);
  process.exit(code);
}

// ── Faux serveur : le « fetch » de la page ──────────────────────────────────
function fabriquerFetch(cfg, requetes) {
  return async (url, init = {}) => {
    const methode = String(init.method || 'GET').toUpperCase();
    const adresse = new URL(String(url), 'https://example.org/');
    const entetes0 = {};
    const h = init.headers;
    if (h) {
      const paires = Array.isArray(h) ? h : typeof h.entries === 'function' ? [...h.entries()] : Object.entries(h);
      for (const [k, v] of paires) entetes0[String(k).toLowerCase()] = String(v);
    }
    requetes.push({ methode, chemin: adresse.pathname, corps: init.body === undefined ? '' : String(init.body), entetes: entetes0 });
    await sleepCourt();
    if (cfg.panne) throw new TypeError('Failed to fetch (le faux serveur est injoignable)');
    const entetes = { 'Content-Type': 'application/json' };
    if (cfg.erreur) return new Response(JSON.stringify({ detail: 'Erreur simulée' }), { status: cfg.erreur, headers: entetes });
    if (methode === 'POST') return new Response(init.body === undefined ? '{}' : String(init.body), { status: 201, headers: entetes });
    const fichier = path.resolve(path.dirname(cfg.fichier), cfg.api, adresse.pathname.replace(/^\/api\//, '').replace(/\/$/, '') + '.json');
    if (!fichier.includes('..') && fs.existsSync(fichier)) return new Response(fs.readFileSync(fichier, 'utf8'), { status: 200, headers: entetes });
    return new Response(JSON.stringify({ detail: 'Introuvable' }), { status: 404, headers: entetes });
  };
}
const sleepCourt = () => sommeil(15);

// ── Media queries : jsdom ignore @media ; on applique celles qui correspondent à --largeur ─────────────────────────
function appliquerMediaQueries(document, largeur) {
  let css = '';
  for (const feuille of Array.from(document.styleSheets)) {
    let regles;
    try { regles = Array.from(feuille.cssRules); } catch { continue; }
    for (const regle of regles) {
      if (!regle.media) continue;
      const texte = String(regle.media.mediaText || '');
      const conditions = [...texte.matchAll(/\(\s*(min|max)-width\s*:\s*([\d.]+)px\s*\)/g)];
      if (!conditions.length) continue;
      const ok = conditions.every(([, sens, px]) => (sens === 'min' ? largeur >= Number(px) : largeur <= Number(px)));
      if (ok) css += Array.from(regle.cssRules).map((r) => r.cssText).join('\n') + '\n';
    }
  }
  if (css) {
    const style = document.createElement('style');
    style.textContent = css;
    document.head.append(style);
  }
}

// ── Diagnostic ──────────────────────────────────────────────────────────────
function decrire(el) {
  let t = el.localName;
  if (el.id) t += '#' + el.id;
  if (el.classList.length) t += '.' + Array.from(el.classList).join('.');
  return t;
}

function arbre(el, niveau, lignes) {
  if (niveau > 6 || lignes.length > 60) return;
  const enfants = Array.from(el.children);
  const texteDirect = Array.from(el.childNodes).filter((n) => n.nodeType === 3).map((n) => n.textContent).join(' ').replace(/\s+/g, ' ').trim();
  let ligne = '  '.repeat(niveau) + decrire(el);
  if (texteDirect) ligne += ' « ' + (texteDirect.length > 50 ? texteDirect.slice(0, 47) + '…' : texteDirect) + ' »';
  lignes.push(ligne);
  for (const e of enfants) arbre(e, niveau + 1, lignes);
}

function diagnostic(fenetre, cfg, journal) {
  const d = fenetre.document;
  console.log(`Page : ${cfg.fichier}`);
  console.log(`Titre de l'onglet : ${d.title || '(aucun <title>)'}`);
  console.log(`Langue : ${d.documentElement.getAttribute('lang') || '(non précisée)'}`);
  console.log('Structure de la page (le DOM obtenu) :');
  const lignes = [];
  if (d.body) arbre(d.body, 1, lignes);
  console.log(lignes.join('\n') || '  (corps vide)');
  const feuilles = d.querySelectorAll('link[rel~=stylesheet]').length;
  const scripts = d.querySelectorAll('script[src]').length;
  console.log(`Feuilles de style liées : ${feuilles} ; scripts liés : ${scripts}`);
  for (const m of journal.console) console.log(`[console] ${m}`);
  for (const m of journal.erreurs) console.log(`[ERREUR dans la page] ${m}`);
}

// ── Contrôles ───────────────────────────────────────────────────────────────
function controleAccessible(d) {
  const p = [];
  if (!d.documentElement.getAttribute('lang')) p.push('la balise <html> n\'a pas d\'attribut lang');
  if (!d.title.trim()) p.push('la page n\'a pas de <title>');
  const h1 = d.querySelectorAll('h1').length;
  if (h1 !== 1) p.push(`il faut exactement un <h1> (il y en a ${h1})`);
  let precedent = 0;
  for (const t of d.querySelectorAll('h1,h2,h3,h4,h5,h6')) {
    const niveau = Number(t.localName[1]);
    if (precedent && niveau > precedent + 1) p.push(`saut de niveau de titre : <h${precedent}> puis <${t.localName}> ("${t.textContent.trim()}")`);
    precedent = niveau;
  }
  for (const img of d.querySelectorAll('img')) {
    if (!(img.getAttribute('alt') || '').trim()) p.push(`image sans texte alternatif (alt) : ${img.getAttribute('src') || '?'}`);
  }
  for (const l of d.querySelectorAll('label[for]')) {
    if (!d.getElementById(l.getAttribute('for'))) p.push(`le libellé « ${l.textContent.trim()} » vise un id qui n'existe pas : ${l.getAttribute('for')}`);
  }
  for (const c of d.querySelectorAll('input, select, textarea')) {
    const type = (c.getAttribute('type') || 'text').toLowerCase();
    if (['hidden', 'submit', 'button', 'reset', 'image'].includes(type)) continue;
    const libelle = (c.id && d.querySelector(`label[for="${c.id}"]`)) || c.closest('label');
    if (!libelle) p.push(`champ sans libellé : <${c.localName} name="${c.getAttribute('name') || ''}">`);
  }
  return p;
}

// ── Programme principal ─────────────────────────────────────────────────────
async function main() {
  const cfg = lireArguments(process.argv.slice(2));
  if (!fs.existsSync(cfg.fichier)) fin(2, `Fichier introuvable : ${cfg.fichier}`);

  const journal = { console: [], erreurs: [] };
  const requetes = [];
  const virtuelle = new VirtualConsole();
  virtuelle.on('log', (...a) => journal.console.push(a.map(String).join(' ')));
  virtuelle.on('info', (...a) => journal.console.push(a.map(String).join(' ')));
  virtuelle.on('warn', (...a) => journal.console.push('(avertissement) ' + a.map(String).join(' ')));
  virtuelle.on('error', (...a) => journal.console.push('(erreur) ' + a.map(String).join(' ')));
  virtuelle.on('jsdomError', (e) => journal.erreurs.push(e.detail && e.detail.message ? `${e.message} : ${e.detail.message}` : e.message));
  process.on('unhandledRejection', (e) => journal.erreurs.push('promesse rejetée sans catch : ' + (e && e.message ? e.message : e)));

  const dom = await JSDOM.fromFile(cfg.fichier, {
    runScripts: 'dangerously',
    resources: 'usable',
    pretendToBeVisual: true,
    virtualConsole: virtuelle,
    beforeParse(fenetre) {
      fenetre.innerWidth = cfg.largeur;
      fenetre.fetch = fabriquerFetch(cfg, requetes);
    },
  });
  const fenetre = dom.window;
  const d = fenetre.document;
  await new Promise((r) => (d.readyState === 'complete' ? r() : fenetre.addEventListener('load', r)));
  await sommeil(150);
  appliquerMediaQueries(d, cfg.largeur);

  diagnostic(fenetre, cfg, journal);

  let echecs = 0;
  const ok = (msg) => console.log('OK     ' + msg);
  const ko = (msg) => { echecs++; console.log('ÉCHEC  ' + msg); };
  const trouver = (sel) => { try { return Array.from(d.querySelectorAll(sel)); } catch { ko(`sélecteur CSS invalide : ${sel}`); return []; } };
  if (cfg.ops.length) console.log('Contrôles :');

  for (const { op, args } of cfg.ops) {
    const [a, b, c] = args;
    switch (op) {
      case 'clic': case 'clics': {
        const n = op === 'clics' ? Number(b) : 1;
        const el = trouver(a)[0];
        if (!el) { ko(`clic impossible : aucun élément ${a}`); break; }
        for (let i = 0; i < n; i++) el.dispatchEvent(new fenetre.MouseEvent('click', { bubbles: true, cancelable: true }));
        await sommeil(50);
        ok(`clic${n > 1 ? 's (' + n + ')' : ''} sur ${a}`);
        break;
      }
      case 'saisie': {
        const el = trouver(a)[0];
        if (!el) { ko(`saisie impossible : aucun champ ${a}`); break; }
        el.value = b;
        el.dispatchEvent(new fenetre.Event('input', { bubbles: true }));
        el.dispatchEvent(new fenetre.Event('change', { bubbles: true }));
        ok(`saisie de « ${b} » dans ${a}`);
        break;
      }
      case 'envoyer': {
        const el = trouver(a)[0];
        if (!el) { ko(`envoi impossible : aucun formulaire ${a}`); break; }
        const evenement = new fenetre.Event('submit', { bubbles: true, cancelable: true });
        el.dispatchEvent(evenement);
        await sommeil(150);
        if (evenement.defaultPrevented) ok(`envoi de ${a} intercepté par la page`);
        else ko(`envoi de ${a} : la page ne l'intercepte pas (preventDefault manquant ?), le navigateur la rechargerait`);
        break;
      }
      case 'attendre': await sommeil(Number(a)); break;
      case 'existe': {
        const n = trouver(a).length;
        n ? ok(`${a} existe (${n} élément${n > 1 ? 's' : ''})`) : ko(`aucun élément ne correspond à ${a}`);
        break;
      }
      case 'absent': trouver(a).length ? ko(`${a} ne devrait pas exister`) : ok(`${a} est absent`); break;
      case 'vide': {
        const el = trouver(a)[0];
        if (!el) ko(`aucun élément ${a}`);
        else if (el.textContent.trim() === '') ok(`${a} est vide`);
        else ko(`${a} devrait être vide, il contient « ${el.textContent.trim()} »`);
        break;
      }
      case 'contient': case 'egal': {
        const el = trouver(a)[0];
        if (!el) { ko(`aucun élément ${a}`); break; }
        const texte = el.textContent.replace(/\s+/g, ' ').trim();
        const bon = op === 'egal' ? norme(texte) === norme(b) : norme(texte).includes(norme(b));
        bon ? ok(`${a} ${op === 'egal' ? 'vaut' : 'contient'} « ${b} »`) : ko(`${a} contient « ${texte} » au lieu de « ${b} »`);
        break;
      }
      case 'style': {
        const el = trouver(a)[0];
        if (!el) { ko(`aucun élément ${a}`); break; }
        const valeur = fenetre.getComputedStyle(el).getPropertyValue(b).trim();
        norme(valeur) === norme(c) ? ok(`${a} : ${b} = ${valeur}`) : ko(`${a} : ${b} vaut « ${valeur || '(vide)'} » au lieu de « ${c} »`);
        break;
      }
      case 'appelle': {
        const trouve = requetes.some((r) => r.methode === a.toUpperCase() && r.chemin === b);
        trouve ? ok(`la page a envoyé ${a.toUpperCase()} ${b}`) : ko(`la page n'a pas envoyé ${a.toUpperCase()} ${b} (requêtes vues : ${requetes.map((r) => r.methode + ' ' + r.chemin).join(', ') || 'aucune'})`);
        break;
      }
      case 'corps': {
        const derniere = requetes[requetes.length - 1];
        derniere && new RegExp(a).test(derniere.corps) ? ok(`le corps de la dernière requête correspond à ${a}`) : ko(`le corps de la dernière requête (« ${derniere ? derniere.corps : 'aucune requête'} ») ne correspond pas à ${a}`);
        break;
      }
      case 'entete': {
        const derniere = requetes[requetes.length - 1];
        const val = derniere && derniere.entetes[a.toLowerCase()];
        val !== undefined && norme(val) === norme(b) ? ok(`l'en-tête ${a} vaut ${b}`) : ko(`l'en-tête ${a} de la dernière requête vaut « ${val === undefined ? 'absent' : val} » au lieu de « ${b} »`);
        break;
      }
      case 'accessible': {
        const problemes = controleAccessible(d);
        if (!problemes.length) ok('accessibilité de base : aucun problème');
        for (const p of problemes) ko(p);
        break;
      }
    }
  }

  if (journal.erreurs.length) echecs++;
  console.log(echecs ? `\nRésultat : ${echecs} problème(s).` : '\nRésultat : tout est bon.');
  fenetre.close();
  process.exit(echecs ? 1 : 0);
}

main().catch((e) => fin(2, 'Erreur de l\'outil : ' + (e && e.stack ? e.stack : e)));
