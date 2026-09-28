// SPDX-License-Identifier: GPL-2.0-or-later

import QtQuick
import QtQuick.Controls as QQC2

// ComboBox bound to the kcm property named by `setting`; model entries are { text, value }
QQC2.ComboBox {
    required property string setting
    // Shown when the setting's value isn't in the model
    property string unsetText

    textRole: "text"
    valueRole: "value"
    // Bind once the model is set, or indexOfValue() runs against an empty model
    Component.onCompleted: currentIndex = Qt.binding(() => indexOfValue(kcm[setting]))
    displayText: currentIndex < 0 ? unsetText : currentText
    onActivated: kcm[setting] = currentValue
}
