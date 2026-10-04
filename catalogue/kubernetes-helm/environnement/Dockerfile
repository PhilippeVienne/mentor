# Environnement réel du parcours « Kubernetes et Helm » : helm, kubectl (client seul), kubeconform avec les schémas
# Kubernetes embarqués, yamllint, nano et un serveur SSH pour VS Code. IL N'Y A PAS DE CLUSTER dans ce conteneur.
# Règles (vérifiées à la compilation du catalogue, voir catalogue/README.md §15) :
#   - image de base de la liste blanche, avec un tag explicite et épinglé ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le réseau n'existe QU'AU BUILD : helm, kubectl, kubeconform et les schémas sont donc téléchargés ici (sommes SHA-256
# vérifiées) ; à l'exécution il n'y a ni réseau, ni cluster, ni démon Docker.
# python3 + python3-yaml : outils de contrôle des leçons (outils/verifier-k8s) et couche VS Code web du portail.
FROM debian:trixie-slim

ARG HELM_VERSION=3.19.0
ARG HELM_SHA256_AMD64=a7f81ce08007091b86d8bd696eb4d86b8d0f2e1b9f6c714be62f82f96a594496
ARG HELM_SHA256_ARM64=440cf7add0aee27ebc93fada965523c1dc2e0ab340d4348da2215737fc0d76ad
ARG KUBECTL_VERSION=1.37.1
ARG KUBECTL_SHA256_AMD64=65691ff77eb6fa44c908b77a1082c9f092c3b9733b5cefabec0d1104890e21a8
ARG KUBECTL_SHA256_ARM64=ff749f4b78d9c4f1ec87307df9b50119ed819e2094aa9810cb9acffc3286c8c7
ARG KUBECONFORM_VERSION=0.8.0
ARG KUBECONFORM_SHA256_AMD64=9bc2bffbf71f261128533edaf912153948b7ff238f9a531ae6d34466ec287883
ARG KUBECONFORM_SHA256_ARM64=1f53fc8e81258197a35e8603054162a5af1de8c5af13746c71ab680d9534ed87
# Schémas JSON des objets Kubernetes (dépôt yannh/kubernetes-json-schema), commit épinglé ; version du dossier : v1.37.1.
ARG SCHEMAS_COMMIT=8df8a883b68a24a104b4a9e43c1288090ae60b3b

RUN apt-get update \
    && apt-get install -y --no-install-recommends bash ca-certificates curl git jq less nano openssh-server procps \
       python3 python3-yaml yamllint \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant

# helm, kubectl et kubeconform : binaires officiels, sommes SHA-256 vérifiées (publiées avec chaque version).
RUN set -eux; \
    arch="$(dpkg --print-architecture)"; \
    case "$arch" in \
      amd64) helm_sha="$HELM_SHA256_AMD64"; kubectl_sha="$KUBECTL_SHA256_AMD64"; kc_sha="$KUBECONFORM_SHA256_AMD64" ;; \
      arm64) helm_sha="$HELM_SHA256_ARM64"; kubectl_sha="$KUBECTL_SHA256_ARM64"; kc_sha="$KUBECONFORM_SHA256_ARM64" ;; \
      *) echo "architecture non prévue : $arch" >&2; exit 1 ;; \
    esac; \
    curl -fsSL -o /tmp/helm.tgz "https://get.helm.sh/helm-v${HELM_VERSION}-linux-${arch}.tar.gz"; \
    echo "${helm_sha}  /tmp/helm.tgz" | sha256sum -c -; \
    tar -C /tmp -xzf /tmp/helm.tgz "linux-${arch}/helm"; \
    install -m 0755 "/tmp/linux-${arch}/helm" /usr/local/bin/helm; \
    curl -fsSL -o /usr/local/bin/kubectl "https://dl.k8s.io/release/v${KUBECTL_VERSION}/bin/linux/${arch}/kubectl"; \
    echo "${kubectl_sha}  /usr/local/bin/kubectl" | sha256sum -c -; \
    chmod 0755 /usr/local/bin/kubectl; \
    curl -fsSL -o /tmp/kubeconform.tgz "https://github.com/yannh/kubeconform/releases/download/v${KUBECONFORM_VERSION}/kubeconform-linux-${arch}.tar.gz"; \
    echo "${kc_sha}  /tmp/kubeconform.tgz" | sha256sum -c -; \
    mkdir -p /opt/outils; \
    tar -C /opt/outils -xzf /tmp/kubeconform.tgz kubeconform; \
    rm -rf /tmp/helm.tgz /tmp/linux-* /tmp/kubeconform.tgz

# Schémas Kubernetes embarqués : kubeconform les lit sur le disque (pas de réseau à l'exécution).
RUN set -eux; \
    mkdir /tmp/schemas && cd /tmp/schemas; \
    git init -q .; \
    git remote add origin https://github.com/yannh/kubernetes-json-schema.git; \
    git config core.sparseCheckout true; \
    echo "/v${KUBECTL_VERSION}-standalone-strict/" > .git/info/sparse-checkout; \
    git fetch -q --depth 1 --filter=blob:none origin "$SCHEMAS_COMMIT"; \
    git checkout -q FETCH_HEAD; \
    mkdir -p /opt/schemas; \
    mv "v${KUBECTL_VERSION}-standalone-strict" /opt/schemas/; \
    chmod -R a+rX /opt/schemas; \
    cd /; rm -rf /tmp/schemas

# `kubeconform` (dans le PATH) est un petit script qui ajoute les schémas locaux ; `verifier-k8s` contrôle la cohérence
# entre objets (ce que ferait le cluster : un service trouve-t-il ses pods ? les références existent-elles ?).
COPY outils/kubeconform /usr/local/bin/kubeconform
COPY outils/verifier-k8s /usr/local/bin/verifier-k8s
RUN chmod 0755 /usr/local/bin/kubeconform /usr/local/bin/verifier-k8s

ENV PATH=/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

# Les exercices sont lisibles par tout le monde ; chaque labo en copie un dans /workspace (qui est effacé à l'arrêt).
COPY exercices /opt/exercices
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh

WORKDIR /workspace
USER apprenant
