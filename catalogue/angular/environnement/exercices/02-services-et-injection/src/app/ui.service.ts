import { Injectable } from '@angular/core';
import { Observable, Subject } from 'rxjs';

// Le service d'Adhésion, simplifié : un écran envoie le titre de la page, l'écran racine l'écoute.
// `@Injectable()` tout seul : Angular sait qu'il PEUT fabriquer cette classe, mais pas où ni pour qui.
@Injectable()
export class UiService {
  private subject = new Subject<string>();

  setTitle(message: string) {
    this.subject.next(message);
  }

  getTitle(): Observable<string> {
    return this.subject.asObservable();
  }
}
