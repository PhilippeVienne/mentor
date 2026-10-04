import { Injectable } from '@angular/core';

@Injectable({ providedIn: 'root' })
export class Notifications {
  prevenir(message: string) {
    console.log(message);
  }
}

@Injectable({ providedIn: 'root' })
export class NotificationsSilencieuses extends Notifications {
  override prevenir(_message: string) {
    /* ne fait rien */
  }
}
