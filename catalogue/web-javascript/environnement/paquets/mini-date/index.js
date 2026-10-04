// Formate une date au format français jj/mm/aaaa (fuseau UTC, pour un résultat prévisible).
exports.dateFr = function dateFr(date) {
  const jj = String(date.getUTCDate()).padStart(2, '0');
  const mm = String(date.getUTCMonth() + 1).padStart(2, '0');
  return `${jj}/${mm}/${date.getUTCFullYear()}`;
};
