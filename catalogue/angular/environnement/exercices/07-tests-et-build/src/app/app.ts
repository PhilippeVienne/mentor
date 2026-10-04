import { Component } from '@angular/core';
import { environment } from '../environments/environment';

@Component({
  selector: 'app-root',
  template: `
    <h1>Club photo</h1>
    <p>API : {{ api }}</p>
    <p>Connexion : {{ keycloak }} (realm {{ realm }}, client {{ client }})</p>
  `,
})
export class App {
  readonly api = environment.adhesion_api_url;
  readonly keycloak = environment.keycloak_url;
  readonly realm = environment.keycloak_realm;
  readonly client = environment.keycloak_client_id;
}
