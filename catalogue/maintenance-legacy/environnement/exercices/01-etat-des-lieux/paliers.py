"""Calcule les paliers d'une montée de version Django : les LTS intermédiaires, puis la cible."""
import sys

VERSIONS = ["3.0", "3.1", "3.2", "4.0", "4.1", "4.2", "5.0", "5.1", "5.2"]
LTS = {"3.2", "4.2", "5.2"}


def paliers(depart, arrivee):
    """Versions à traverser : les LTS intermédiaires, puis la cible."""
    debut = VERSIONS.index(depart) + 1
    fin = VERSIONS.index(arrivee) + 1
    return [v for v in VERSIONS[debut:fin] if v in LTS or v == arrivee]


if __name__ == "__main__":
    depart, arrivee = sys.argv[1], sys.argv[2]
    print(" -> ".join([depart, *paliers(depart, arrivee)]))
