/* Catalogue : recherche et filtres côté navigateur (amélioration progressive : sans JavaScript, tout est affiché). */
(function (root) {
    'use strict';

    /* Minuscules sans accents, pour comparer « Sécurité » et « securite ». */
    function normalize(text) {
        return String(text || '').toLowerCase().normalize('NFD').replace(/[̀-ͯ]/g, '');
    }

    /* Une carte correspond si elle contient tous les mots de la recherche et respecte le filtre actif. */
    function matches(card, query, filter) {
        const words = normalize(query).split(/\s+/).filter(Boolean);
        const text = normalize(card.search);
        if (!words.every((w) => text.includes(w))) return false;
        if (filter === 'sans-prerequis') return card.state !== 'bientot' && !card.prereq;
        return !filter || filter === 'tous' || card.state === filter;
    }

    function countLabel(shown, total) {
        if (shown === total) return `${total} parcours`;
        return `${shown} parcours sur ${total}`;
    }

    function init(doc) {
        const container = doc.getElementById('catalogue-root');
        if (!container) return;
        const tools = container.querySelector('.cat-tools');
        const input = doc.getElementById('cat-search');
        const count = doc.getElementById('cat-count');
        const empty = doc.getElementById('cat-empty');
        const buttons = Array.from(container.querySelectorAll('.cat-filter'));
        const cards = Array.from(container.querySelectorAll('[data-course]'));
        let filter = 'tous';

        function apply() {
            let shown = 0;
            cards.forEach((el) => {
                const ok = matches({ search: el.dataset.search, state: el.dataset.state, prereq: el.dataset.prereq === '1' }, input.value, filter);
                el.hidden = !ok;
                if (ok) shown++;
            });
            container.querySelectorAll('[data-cat-section]').forEach((section) => {
                section.hidden = !section.querySelector('[data-course]:not([hidden])');
            });
            empty.hidden = shown > 0;
            count.textContent = countLabel(shown, cards.length);
        }

        function setFilter(name) {
            filter = name;
            buttons.forEach((b) => b.setAttribute('aria-pressed', b.dataset.filter === name ? 'true' : 'false'));
            apply();
        }

        buttons.forEach((b) => b.addEventListener('click', () => setFilter(b.dataset.filter)));
        input.addEventListener('input', apply);
        doc.getElementById('cat-reset').addEventListener('click', () => {
            input.value = '';
            setFilter('tous');
            input.focus();
        });
        tools.hidden = false;
        apply();
    }

    const api = { normalize, matches, countLabel };
    if (typeof module !== 'undefined' && module.exports) module.exports = api;
    else root.CatalogueUI = api;
    if (typeof document !== 'undefined') init(document);
}(typeof window !== 'undefined' ? window : globalThis));
