#!/bin/sh
# KDE-style string extraction; run through po/update.sh (or KDE's scripty)
$EXTRACTRC `find kcm -name '*.ui'` >> rc.cpp 2>/dev/null
$XGETTEXT `find kcm -name '*.cpp' -o -name '*.qml' | sort` -o $podir/kcm_framework.pot
rm -f rc.cpp
