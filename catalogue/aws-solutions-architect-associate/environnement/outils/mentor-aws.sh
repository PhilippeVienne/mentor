# Réglages du labo AWS, lus (« sourcés ») par les terminaux, par le lanceur et par demarrer-aws.
# Tout est FACTICE et LOCAL : le point d'accès est l'émulateur MiniStack de cet environnement, les identifiants
# n'ouvrent aucun accès réel. Un réglage déjà défini dans l'environnement est conservé.
#   - AWS_ENDPOINT_URL : l'AWS CLI et les SDK parlent à l'émulateur au lieu d'Amazon (inutile de taper --endpoint-url) ;
#   - la clé « test » est, pour MiniStack, l'utilisateur racine du compte fictif 000000000000 ;
#   - eu-west-3 (Paris) est la région par défaut du labo.
: "${AWS_ENDPOINT_URL:=http://127.0.0.1:4566}"
: "${AWS_DEFAULT_REGION:=eu-west-3}"
: "${AWS_ACCESS_KEY_ID:=test}"
: "${AWS_SECRET_ACCESS_KEY:=test}"
: "${AWS_PAGER:=}"
: "${AWS_EC2_METADATA_DISABLED:=true}"
export AWS_ENDPOINT_URL AWS_DEFAULT_REGION AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY AWS_PAGER AWS_EC2_METADATA_DISABLED
