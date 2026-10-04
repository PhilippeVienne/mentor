#!/usr/bin/env python3
"""Télécharge un objet d'un bucket S3, avec extraction facultative.

Version SIMPLIFIÉE écrite pour le parcours de formation, dans l'esprit de restore.py du dépôt
equipe/dev/backups3 (ce n'est pas une copie). Elle lit les variables URL, ACCESSKEY et SECRETKEY.

Usage : restore.py [-v] [-c] [--date "AAAA-MM-JJ HH:MM:SS"] BUCKET OBJET DESTINATION/
"""
import argparse
import os
import sys
import tarfile
from datetime import timezone

import boto3


def main():
    parser = argparse.ArgumentParser(description="Télécharge un objet d'un bucket S3.")
    parser.add_argument("-v", "--verbose", action="store_true", help="affiche les messages d'information")
    parser.add_argument("-c", "--compress", action="store_true", help="extrait l'archive puis supprime le fichier téléchargé")
    parser.add_argument("--date", help='date de la version voulue, en UTC : "AAAA-MM-JJ HH:MM:SS"')
    parser.add_argument("bucket", help="bucket source")
    parser.add_argument("objet", help="nom (clé) de l'objet")
    parser.add_argument("destination", help="dossier de destination, avec un / final")
    args = parser.parse_args()

    s3 = boto3.client(
        "s3",
        endpoint_url=os.environ["URL"],
        aws_access_key_id=os.environ["ACCESSKEY"],
        aws_secret_access_key=os.environ["SECRETKEY"],
        region_name="us-east-1",
    )

    extra = {}
    if args.date:
        # On cherche la version dont la date de dernière modification correspond EXACTEMENT, à la seconde près.
        versions = s3.list_object_versions(Bucket=args.bucket, Prefix=args.objet).get("Versions", [])
        trouvees = [
            v["VersionId"]
            for v in versions
            if v["Key"] == args.objet
            and v["LastModified"].astimezone(timezone.utc).strftime("%Y-%m-%d %H:%M:%S") == args.date
        ]
        if not trouvees:
            sys.exit(f"Erreur : aucune version de {args.objet} datée de {args.date} (UTC)")
        extra["VersionId"] = trouvees[0]

    os.makedirs(args.destination, exist_ok=True)
    chemin = args.destination + args.objet  # simple concaténation : d'où le / final obligatoire
    if args.verbose:
        print(f"Téléchargement de {args.bucket}/{args.objet} vers {chemin}")
    s3.download_file(args.bucket, args.objet, chemin, ExtraArgs=extra or None)
    if args.compress:
        with tarfile.open(chemin) as archive:
            archive.extractall(args.destination, filter="data")
        os.remove(chemin)
    if args.verbose:
        print("Restauration terminée")


if __name__ == "__main__":
    main()
