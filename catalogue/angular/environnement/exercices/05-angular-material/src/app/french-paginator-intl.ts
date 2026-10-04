import { Injectable } from '@angular/core';
import { MatPaginatorIntl } from '@angular/material/paginator';

// Les textes du bas des tableaux paginés de Material sont en anglais par défaut.
@Injectable()
export class FrenchMatPaginatorIntl extends MatPaginatorIntl {
  // À FAIRE (étape 5) : traduis `itemsPerPageLabel` (« Éléments par page : ») et `nextPageLabel` (« Page suivante »).
}
