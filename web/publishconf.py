# Used only by `make publish` (production build) or when passed explicitly as -s.
import os
import sys

sys.path.append(os.curdir)
from pelicanconf import *  # noqa: F401,F403

SITEURL = "https://naba.ysapp.net"
RELATIVE_URLS = False

DELETE_OUTPUT_DIRECTORY = True
