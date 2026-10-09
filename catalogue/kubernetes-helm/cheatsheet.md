## Vocabulaire

| Mot | Sens |
| --- | --- |
| Cluster / nœud | Ensemble de machines pilotées par Kubernetes / une de ces machines |
| Pod | Un ou plusieurs conteneurs qui partagent réseau et volumes ; jetable |
| Deployment | Décrit l'image, le nombre de copies, les sondes ; crée et remplace les pods |
| Service | Adresse et nom DNS stables devant des pods choisis par label |
| Namespace | « Dossier » du cluster qui range les objets par projet |
| ConfigMap / Secret | Configuration non sensible / valeurs sensibles (base64, pas chiffré) |
| PVC | Demande de volume ; `ReadWriteOnce` (un nœud) ou `ReadWriteMany` (plusieurs) |
| Ingress / contrôleur | Règle de routage HTTP / programme qui l'applique (HAProxy, Traefik) |
| Chart / release / révision | Paquet Helm / installation nommée / étape de son historique |

## kubectl

```bash
kubectl apply -f fichier.yaml               # crée ou met à jour
kubectl get pods -n NAMESPACE               # liste (aussi : deploy, svc, ingress, pvc, nodes)
kubectl describe pod NOM                    # détail, évènements en bas
kubectl logs NOM -c CONTENEUR --previous    # journaux (de l'exécution précédente)
kubectl exec -it NOM -c CONTENEUR -- sh     # terminal dans le conteneur
kubectl port-forward service/NOM 8080:80    # ton port 8080 vers le port 80 du service
kubectl rollout restart deployment/NOM      # relance les pods (après un ConfigMap modifié)
```

## Valider sans cluster (atelier du cours)

```bash
kubectl create deployment NOM --image=IMAGE --dry-run=client -o yaml > deployment.yaml   # fabrique le YAML, n'envoie rien
kubeconform fichier.yaml                    # le fichier respecte-t-il le schéma officiel de Kubernetes ?
yamllint fichier.yaml                       # la syntaxe YAML est-elle correcte ?
verifier-k8s selecteur deployment.yaml service.yaml   # le service trouve-t-il ses pods ? (outil de l'atelier)
```

Aucune de ces commandes ne prouve que l'application fonctionne : il n'y a pas de cluster dans l'atelier.

## Helm

```bash
helm lint ./chart                           # vérifie le chart
helm template ma-release ./chart -f v.yaml  # affiche le YAML rendu, sans toucher au cluster
helm package ./chart                        # fabrique l'archive chart-VERSION.tgz
helm install ma-release ./chart             # première installation
helm upgrade ma-release ./chart --set a.b=1 # mise à jour (nouvelle révision)
helm history ma-release                     # révisions
helm rollback ma-release 1                  # retour à la révision 1
helm uninstall ma-release                   # suppression
```

## Diagnostic

| État | Piste |
| --- | --- |
| `Pending` | `describe` : volume non prêt, ressources insuffisantes |
| `ImagePullBackOff` | nom, étiquette ou accès de l'image |
| `CrashLoopBackOff` | `logs --previous` : variable, secret ou base manquants |
| `Running` mais `0/1` | sonde de disponibilité en échec, ou démarrage encore en cours |

Ordre : `get pods`, puis `describe`, puis `logs`.

## Cluster de dev (infra-dev)

```bash
sh startme.sh                       # k3d cluster create --api-port 6550 -p "80:80@loadbalancer"
sh install-keycloak.sh              # namespace, PVC, Secret, helm install
k3d cluster delete k3s-default      # tout supprimer
```

Noms : `sso.172.17.0.1.nip.io`. Pas de HTTPS, pas de `ReadWriteMany`, un seul nœud.

## À retenir sur cluster-configuration

Dépôt ancien (2019 environ) : Tiller (Helm 2), charts `stable/` et `incubator/`, cert-manager 0.6 (`certmanager.k8s.io/v1alpha1`). À confirmer avec l'équipe Infra avant de copier quoi que ce soit.
