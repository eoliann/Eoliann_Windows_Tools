import subprocess
import re
from pathlib import Path

# === CONFIG ===
INSTALL_RS = Path("src/tabs/install.rs")  # ajustează calea dacă e nevoie

# 1. Extrage toate winget_id din install.rs
with open(INSTALL_RS, "r", encoding="utf-8") as f:
    content = f.read()

ids = re.findall(r'winget_id:\s*"([^"]+)"', content)
print(f"Am găsit {len(ids)} winget_id în APP_CATALOG")

invalid_ids = []

# 2. Verifică fiecare ID cu winget
for wid in ids:
    try:
        result = subprocess.run(
            ["winget", "search", "--id", wid, "--exact"],
            capture_output=True,
            text=True,
            check=False
        )
        output = result.stdout.strip()
        if wid.lower() not in output.lower():
            invalid_ids.append(wid)
            print(f"❌ Invalid: {wid}")
        else:
            print(f"✅ Valid:   {wid}")
    except Exception as e:
        print(f"⚠️ Eroare la {wid}: {e}")
        invalid_ids.append(wid)

# 3. Rezumat
print("\n=== Rezumat ===")
if invalid_ids:
    print("ID-uri invalide găsite:")
    for wid in invalid_ids:
        print(" -", wid)
else:
    print("Toate ID-urile sunt valide ✅")
