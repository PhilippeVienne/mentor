import { renderToStaticMarkup } from "react-dom/server";
import { App } from "./App";

// Transforme l'application en texte HTML, comme le ferait le serveur, et l'affiche dans le terminal.
console.log(renderToStaticMarkup(<App />));
