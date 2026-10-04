from importlib.metadata import PackageNotFoundError, version

ATTENDUES = {"Django": "5.2", "sqlparse": "0.5"}

for nom, prefixe in ATTENDUES.items():
    try:
        installee = version(nom)
    except PackageNotFoundError:
        print(f"{nom}: absent")
        continue
    etat = "ok" if installee.startswith(prefixe) else "À REVOIR"
    print(f"{nom}: {installee} (attendu {prefixe}.x) {etat}")
