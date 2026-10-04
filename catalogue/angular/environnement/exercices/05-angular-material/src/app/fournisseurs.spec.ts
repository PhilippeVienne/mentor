import { TestBed } from '@angular/core/testing';
import { MatPaginatorIntl } from '@angular/material/paginator';
import { FrenchMatPaginatorIntl } from './french-paginator-intl';
import { fournisseurs } from './fournisseurs';

describe('Paginateur en français', () => {
  beforeEach(() => TestBed.configureTestingModule({ providers: fournisseurs }));

  it('remplace MatPaginatorIntl par FrenchMatPaginatorIntl', () => {
    expect(TestBed.inject(MatPaginatorIntl)).toBeInstanceOf(FrenchMatPaginatorIntl);
  });

  it('a des libellés français', () => {
    const intl = TestBed.inject(MatPaginatorIntl);
    expect(intl.itemsPerPageLabel).toBe('Éléments par page :');
    expect(intl.nextPageLabel).toBe('Page suivante');
  });
});
