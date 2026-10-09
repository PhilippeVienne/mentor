#!/usr/bin/env python3
"""Envoie un fichier ou un dossier vers un bucket S3.

Version SIMPLIFIÉE écrite pour la formation, dans l'esprit de backup.py du dépôt
equipe/dev/backups3 (ce n'est pas une copie). Elle lit les variables URL, ACCESSKEY et SECRETKEY.

Usage : backup.py [-v] [-c] BUCKET SOURCE OBJET
"""
import argparse
import os
import sys
import tarfile
import tempfile

import boto3


def main():
    parser = argparse.ArgumentParser(description="Envoie un fichier ou un dossier vers un bucket S3.")
    parser.add_argument("-v", "--verbose", action="store_true", help="affiche les messages d'information")
    parser.add_argument("-c", "--compress", action="store_true", help="compresse en archive tar.gz avant l'envoi")
    parser.add_argument("bucket", help="bucket de destination")
    parser.add_argument("source", help="fichier ou dossier à sauvegarder")
    parser.add_argument("objet", help="nom (clé) de l'objet dans le bucket")
    args = parser.parse_args()

    if os.path.isdir(args.source) and not args.compress:
        sys.exit("Erreur : un dossier doit être compressé avec -c (folder backup without -c is not supported)")

    s3 = boto3.client(
        "s3",
        endpoint_url=os.environ["URL"],
        aws_access_key_id=os.environ["ACCESSKEY"],
        aws_secret_access_key=os.environ["SECRETKEY"],
        region_name="us-east-1",
    )

    # La copie de travail est faite dans /tmp : prévois autant de place que les données.
    with tempfile.TemporaryDirectory() as tmp:
        envoi = args.source
        if args.compress:
            envoi = os.path.join(tmp, "archive.tgz")
            with tarfile.open(envoi, "w:gz") as archive:
                archive.add(args.source, arcname=os.path.basename(os.path.normpath(args.source)))
        if args.verbose:
            print(f"Envoi de {args.source} vers {args.bucket}/{args.objet}")
        s3.upload_file(envoi, args.bucket, args.objet)
    if args.verbose:
        print("Sauvegarde terminée")


if __name__ == "__main__":
    main()
