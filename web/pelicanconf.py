from datetime import datetime

AUTHOR = "James Dixson"
CURRENT_YEAR = datetime.now().year
SITENAME = "naba"
SITESUBTITLE = "AI image generation from your terminal"
# Full-sentence description for meta/OG (crawlers want >=100 chars).
SITE_DESCRIPTION = (
    "naba is a single-binary Rust CLI for AI image generation across multiple providers "
    "(Google Gemini, OpenRouter, and AWS Bedrock). Generate, edit, restore, and compose "
    "images — icons, patterns, stories, diagrams — straight from the command line."
)
SITEURL = ""

# The GitHub project — canonical for binaries + self-update.
GITHUB_URL = "https://github.com/dixson3/naba"
GITHUB_RELEASES_URL = "https://github.com/dixson3/naba/releases"
# The short, memorable bootstrap install command surfaced on the site.
INSTALL_URL = "https://naba.ysapp.net/install.sh"

PATH = "content"
OUTPUT_PATH = "output"

TIMEZONE = "America/Los_Angeles"
DEFAULT_LANG = "en"

# Feed generation is not desired for a docs site.
FEED_ALL_ATOM = None
CATEGORY_FEED_ATOM = None
TRANSLATION_FEED_ATOM = None
AUTHOR_FEED_ATOM = None
AUTHOR_FEED_RSS = None

# Sitemap only — a small static docs site.
PLUGINS = ["pelican.plugins.sitemap"]
SITEMAP = {
    "format": "xml",
    "priorities": {"articles": 0.5, "indexes": 0.5, "pages": 0.8},
    "changefreqs": {"articles": "monthly", "indexes": "monthly", "pages": "monthly"},
}

# Pretty, directory-style, extension-less URLs (pins the CloudFront index-rewrite
# Function in Issue 3.1: /install/ -> install/index.html). A private-bucket + OAC
# origin does NOT append index.html to subdirectory requests, so this URL style is a
# first-class hosting requirement, not a cosmetic choice.
PAGE_URL = "{slug}/"
PAGE_SAVE_AS = "{slug}/index.html"
# No articles/blog on this site, but pin the article scheme for consistency.
ARTICLE_URL = "posts/{slug}/"
ARTICLE_SAVE_AS = "posts/{slug}/index.html"

# Pages ARE the site — surface them, drive nav explicitly via MENUITEMS below.
DIRECT_TEMPLATES = ["index"]
DISPLAY_PAGES_ON_MENU = False
DISPLAY_CATEGORIES_ON_MENU = False

# Bespoke dark terminal/technical theme.
THEME = "themes/naba-terminal"

# Header nav (title, url). Pretty directory-style URLs.
MENUITEMS = (
    ("home", "/"),
    ("install", "/install/"),
    ("usage", "/usage/"),
    ("config", "/config/"),
    ("skills", "/skills/"),
    ("mcp", "/mcp/"),
    ("github", GITHUB_URL),
)

MARKDOWN = {
    "extension_configs": {
        "markdown.extensions.codehilite": {"css_class": "highlight"},
        "markdown.extensions.extra": {},
        "markdown.extensions.meta": {},
        "markdown.extensions.toc": {"permalink": False, "toc_depth": "2-3"},
    },
    "output_format": "html5",
}

# Static assets. `extra/` files are copied to the site root (favicon, robots, 404,
# install.sh) via EXTRA_PATH_METADATA below.
STATIC_PATHS = ["images", "extra"]
EXTRA_PATH_METADATA = {
    "extra/robots.txt": {"path": "robots.txt"},
    # install.sh is staged into extra/ by web/scripts/sync_installer.sh; it lands at the
    # site root in the normal build/output (and tree-wide `s3 sync`). The Makefile
    # `sync_installer` target re-uploads this one key with an explicit short Cache-Control
    # and invalidates it — the tree-wide sync does NOT set per-key cache headers.
    "extra/install.sh": {"path": "install.sh"},
}

# Uncomment for document-relative URLs when developing.
# RELATIVE_URLS = True
