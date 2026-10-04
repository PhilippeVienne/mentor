import { Component } from '@angular/core';

@Component({
  selector: 'app-titre',
  template: `<h1>{{ titre }}</h1>`,
})
export class TitreComponent {
  titre = 'Mes adhérents';
}
