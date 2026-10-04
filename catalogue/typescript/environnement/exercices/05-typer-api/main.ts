import { chargerEvenements } from "./charger";
import { reponseCassee } from "./serveur";

const evenements = await chargerEvenements(reponseCassee);
console.log(`Places du premier événement : ${evenements[0].places.toFixed(0)}`);
