#!/bin/sh
# Re-extract translatable strings into po/kcm_framework.pot and merge them
# into every po/<lang>/kcm_framework.po. Needs gettext (xgettext, msgmerge).
set -eu
cd "$(dirname "$0")/.."

export podir="$(pwd)/po"
export EXTRACTRC=true
# Same keywords KDE's scripty uses for KI18n
export XGETTEXT="xgettext --from-code=UTF-8 -C --kde --add-comments=i18n \
    -ci18n -ki18n:1 -ki18nc:1c,2 -ki18np:1,2 -ki18ncp:1c,2,3 \
    -kki18n:1 -kki18nc:1c,2 -kki18np:1,2 -kki18ncp:1c,2,3 \
    -kI18N_NOOP:1 -kI18NC_NOOP:1c,2 \
    --package-name=framework-kcm --msgid-bugs-address=github-viral8565@pxdmail.com"
sh Messages.sh

for po in po/*/kcm_framework.po; do
    [ -e "$po" ] || continue
    msgmerge --quiet --update --backup=none --no-fuzzy-matching "$po" po/kcm_framework.pot
done
