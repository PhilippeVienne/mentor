## Cycle de base

| Commande | Rôle |
| --- | --- |
| `terraform init` | Télécharge les fournisseurs, se connecte au backend |
| `terraform workspace show` | Affiche le workspace actif |
| `terraform workspace select production` | Choisit le workspace de production |
| `terraform plan -out=plan.tfplan` | Calcule et enregistre le plan, ne modifie rien |
| `terraform show plan.tfplan` | Relit un plan enregistré |
| `terraform apply plan.tfplan` | Applique exactement ce plan |
| `terraform fmt` | Met en forme les fichiers |
| `terraform validate` | Vérifie la syntaxe |

## Lire un plan

| Symbole | Sens | Réflexe |
| --- | --- | --- |
| `+` | Créé | Vérifier le nom et le namespace |
| `~` | Modifié en place | Lire l'attribut qui change |
| `-` | **Détruit** | S'arrêter, comprendre pourquoi |
| `-/+` | **Détruit puis recréé** | Chercher `forces replacement` |
| `<=` | Donnée lue | Aucun effet |

La ligne `Plan: X to add, Y to change, Z to destroy.` : si **Z** n'est pas 0 et que tu ne l'avais pas prévu, n'applique pas.

## Éléments du langage

| Bloc | Rôle |
| --- | --- |
| `provider` | Plugin qui parle à une API |
| `resource "type" "nom"` | Objet géré, adresse `type.nom` |
| `variable` / `var.x` | Entrée |
| `locals` / `local.x` | Valeur de calcul interne |
| `output` | Valeur exposée (`sensitive = true` masque l'affichage) |
| `module` / `module.x.sortie` | Dossier réutilisable |
| `depends_on` | Ordre explicite entre ressources |
| `lifecycle { prevent_destroy = true }` | Refuse une destruction |

## Dépôt cluster-configuration

| Dossier | Contenu | Préfixe de workspace |
| --- | --- | --- |
| `ingress/` | HAProxy, cert-manager, External-DNS | `ingress-` |
| `kubedb/` | Opérateur de bases KubeDB | `kubedb-` |
| `minio/` | Stockage objet MinIO | `cluster-minio-` |
| `keycloak/` | Keycloak, sa base et son bucket | `keycloak-` |
| `helm-setup/` | Installation de Tiller (Helm 2) | `helm-` |
| `postgres/`, `minio-bucket/` | Modules appelés par d'autres dossiers | aucun |

Tout le travail de production se fait dans le workspace `production`.

## État

| Commande | Effet |
| --- | --- |
| `terraform state list` | Liste les ressources suivies (lecture) |
| `terraform state show <adresse>` | Détail d'une ressource (lecture) |
| `terraform state rm / mv`, `import` | **Modifient l'état** : sauvegarde et accord d'abord |

## Avant un apply en production

1. `terraform workspace show` indique `production`.
2. Lire tout le plan : `destroy`, `replace`, `forces replacement`.
3. Lire les `provisioner` (`when = "destroy"` surtout).
4. Volume, base ou secret touché : demander une relecture.
5. Sauvegarde récente vérifiée.
6. Appliquer le plan enregistré, puis contrôler le service.
