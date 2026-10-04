'use strict';
// Mini serveur de fichiers statiques : « servir [dossier] » publie le dossier sur le port 8000.
// Il ne sert qu'à voir une page dans TON navigateur via la redirection de ports de VS Code.
const http = require('node:http');
const fs = require('node:fs');
const path = require('node:path');

const racine = path.resolve(process.argv[2] || '.');
const port = Number(process.env.PORT || 8000);
const types = {
  '.html': 'text/html; charset=utf-8', '.css': 'text/css; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8', '.json': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml', '.png': 'image/png', '.jpg': 'image/jpeg', '.txt': 'text/plain; charset=utf-8',
};

http.createServer((req, res) => {
  const url = new URL(req.url, 'http://localhost');
  let fichier = path.join(racine, decodeURIComponent(url.pathname));
  if (!fichier.startsWith(racine)) { res.writeHead(403); return res.end('Interdit'); }
  if (fs.existsSync(fichier) && fs.statSync(fichier).isDirectory()) fichier = path.join(fichier, 'index.html');
  fs.readFile(fichier, (err, contenu) => {
    if (err) { res.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' }); return res.end('Introuvable'); }
    res.writeHead(200, { 'Content-Type': types[path.extname(fichier)] || 'application/octet-stream' });
    res.end(contenu);
  });
}).listen(port, '127.0.0.1', () => console.log(`Dossier ${racine} publié sur http://localhost:${port} (Ctrl+C pour arrêter)`));
