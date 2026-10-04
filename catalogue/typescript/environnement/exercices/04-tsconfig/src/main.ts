import { lieuEnMajuscules } from "@/lieu";
import { Association } from "./association";

const asso = new Association();
console.log(asso, lieuEnMajuscules({ titre: "Gala", lieu: "Amphi Chappe" }));
