#!/bin/sh
# Version simplifiée, reconstituée d'après la leçon, du script startme.sh d'infra-dev.
# (Le script réel vérifie aussi que docker, kubectl, helm et k3d sont installés.)
k3d cluster create --api-port 6550 -p "80:80@loadbalancer"
export KUBECONFIG="$(k3d kubeconfig get k3s-default)"
echo "Le cluster est prêt : ouvre http://localhost:8081"
