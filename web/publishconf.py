# Used only by `make publish` (production build) or when passed explicitly as -s.
import os
import sys

sys.path.append(os.curdir)
from pelicanconf import *  # noqa: F401,F403

SITEURL = "https://naba.ysapp.net"
RELATIVE_URLS = False

DELETE_OUTPUT_DIRECTORY = True

# Google Analytics (GA4) — PRODUCTION ONLY. The measurement id is account-specific and is
# NEVER committed: it is read from the environment (local .envrc + GitHub repo secret
# NABA_GA_MEASUREMENT_ID). Set here (not in pelicanconf.py) so dev/`make devserver` and PR
# builds never load analytics. When unset, base.html renders no gtag snippet.
GA_MEASUREMENT_ID = os.environ.get("NABA_GA_MEASUREMENT_ID")
