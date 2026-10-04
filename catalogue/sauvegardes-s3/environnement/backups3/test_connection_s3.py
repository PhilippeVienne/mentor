#!/usr/bin/env python3
"""Liste les buckets accessibles pour tester le point d'accès et les clés (lit URL, ACCESSKEY et SECRETKEY).

Version SIMPLIFIÉE écrite pour le parcours de formation, dans l'esprit de test_connection_s3.py de backups3.
"""
import os

import boto3

s3 = boto3.client(
    "s3",
    endpoint_url=os.environ["URL"],
    aws_access_key_id=os.environ["ACCESSKEY"],
    aws_secret_access_key=os.environ["SECRETKEY"],
    region_name="us-east-1",
)
buckets = [b["Name"] for b in s3.list_buckets()["Buckets"]]
print(f"Connexion OK : {len(buckets)} bucket(s)")
for nom in buckets:
    print(f"- {nom}")
