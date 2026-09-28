// SPDX-License-Identifier: GPL-2.0-or-later

import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts

import org.kde.kirigami as Kirigami

// Small explanatory text under a form field
QQC2.Label {
    Layout.maximumWidth: Kirigami.Units.gridUnit * 20
    wrapMode: Text.Wrap
    font: Kirigami.Theme.smallFont
}
