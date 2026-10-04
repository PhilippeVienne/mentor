# Réglages du labo, lus (« sourcés ») par les terminaux et par le lanceur. Toutes les valeurs sont FACTICES :
# elles n'ouvrent aucun accès réel. Un réglage déjà défini dans l'environnement est conservé.
: "${URL:=http://127.0.0.1:9000}"
: "${ACCESSKEY:=labo-acces-factice}"
: "${SECRETKEY:=labo-secret-factice-0000}"
export URL ACCESSKEY SECRETKEY
# Les mêmes valeurs pour l'AWS CLI et boto3 (variables standard d'AWS), et l'alias « labo » de mc.
: "${AWS_ACCESS_KEY_ID:=$ACCESSKEY}"
: "${AWS_SECRET_ACCESS_KEY:=$SECRETKEY}"
: "${AWS_ENDPOINT_URL:=$URL}"
: "${AWS_DEFAULT_REGION:=us-east-1}"
: "${AWS_REQUEST_CHECKSUM_CALCULATION:=when_required}"
: "${AWS_RESPONSE_CHECKSUM_VALIDATION:=when_required}"
: "${AWS_PAGER:=}"
: "${MC_HOST_labo:=http://$ACCESSKEY:$SECRETKEY@127.0.0.1:9000}"
export AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY AWS_ENDPOINT_URL AWS_DEFAULT_REGION AWS_REQUEST_CHECKSUM_CALCULATION \
    AWS_RESPONSE_CHECKSUM_VALIDATION AWS_PAGER MC_HOST_labo
# PostgreSQL du labo : socket Unix dans /tmp/pgsock, base « asso ».
: "${PGHOST:=/tmp/pgsock}"
: "${PGDATABASE:=asso}"
export PGHOST PGDATABASE
