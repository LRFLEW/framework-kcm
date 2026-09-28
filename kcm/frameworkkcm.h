// SPDX-License-Identifier: GPL-2.0-or-later

#pragma once

#include <KQuickConfigModule>

#include <QDBusError>
#include <QDBusMessage>
#include <QTimer>
#include <QVariant>

#include <functional>

// Settings the user edits; applied through the daemon on save()
struct FrameworkSettings {
    // Defaults; defaults() and representsDefaults use these (see withDefaults())
    int chargeLimit = 100;
    double chargeRateLimit = 1.0;
    int chargeRateSoc = -1; // -1: limit applies at any battery level
    QString fanMode = QStringLiteral("auto"); // auto, duty, rpm
    int fanDuty = 50;
    int fanRpm = 3000;
    QString fpLedLevel;
    int hapticIntensity = -1; // -1: never set, the touchpad can't report it
    QString clickForce;

    bool operator==(const FrameworkSettings &) const = default;
};

class FrameworkKcm : public KQuickConfigModule
{
    Q_OBJECT

    Q_PROPERTY(bool daemonAvailable READ daemonAvailable NOTIFY daemonAvailableChanged)
    Q_PROPERTY(bool busy READ busy NOTIFY busyChanged)
    Q_PROPERTY(QString errorMessage READ errorMessage NOTIFY errorMessageChanged)

    // Which live readings the visible page needs: "thermal", "ports" or empty
    Q_PROPERTY(QString liveData READ liveData WRITE setLiveData NOTIFY liveDataChanged)

    // Live readings, refreshed every couple of seconds while shown
    Q_PROPERTY(QVariantMap systemInfo READ systemInfo NOTIFY systemInfoChanged)
    Q_PROPERTY(QVariantList sensors READ sensors NOTIFY thermalChanged)
    Q_PROPERTY(QVariantList fans READ fans NOTIFY thermalChanged)
    Q_PROPERTY(QVariantMap throttle READ throttle NOTIFY thermalChanged)
    Q_PROPERTY(QVariantList ports READ ports NOTIFY portsChanged)
    Q_PROPERTY(bool fpLedSupported READ fpLedSupported NOTIFY supportChanged)
    Q_PROPERTY(bool chargeLimitOverridden READ chargeLimitOverridden NOTIFY chargeOverrideChanged)

    // Editable settings
    Q_PROPERTY(int chargeLimit READ chargeLimit WRITE setChargeLimit NOTIFY settingsChanged)
    Q_PROPERTY(double chargeRateLimit READ chargeRateLimit WRITE setChargeRateLimit NOTIFY settingsChanged)
    Q_PROPERTY(int chargeRateSoc READ chargeRateSoc WRITE setChargeRateSoc NOTIFY settingsChanged)
    Q_PROPERTY(QString fanMode READ fanMode WRITE setFanMode NOTIFY settingsChanged)
    Q_PROPERTY(int fanDuty READ fanDuty WRITE setFanDuty NOTIFY settingsChanged)
    Q_PROPERTY(int fanRpm READ fanRpm WRITE setFanRpm NOTIFY settingsChanged)
    Q_PROPERTY(QString fpLedLevel READ fpLedLevel WRITE setFpLedLevel NOTIFY settingsChanged)
    Q_PROPERTY(int hapticIntensity READ hapticIntensity WRITE setHapticIntensity NOTIFY settingsChanged)
    Q_PROPERTY(QString clickForce READ clickForce WRITE setClickForce NOTIFY settingsChanged)

public:
    FrameworkKcm(QObject *parent, const KPluginMetaData &data);

    void load() override;
    void save() override;
    void defaults() override;

    bool daemonAvailable() const { return m_daemonAvailable; }
    bool busy() const { return m_busy; }
    QString errorMessage() const { return m_errorMessage; }
    QString liveData() const { return m_liveData; }
    void setLiveData(const QString &liveData);

    QVariantMap systemInfo() const { return m_systemInfo; }
    QVariantList sensors() const { return m_sensors; }
    QVariantList fans() const { return m_fans; }
    QVariantMap throttle() const { return m_throttle; }
    QVariantList ports() const { return m_ports; }
    bool fpLedSupported() const { return m_fpLedSupported; }
    bool chargeLimitOverridden() const { return m_chargeLimitOverridden; }

    int chargeLimit() const { return m_current.chargeLimit; }
    double chargeRateLimit() const { return m_current.chargeRateLimit; }
    int chargeRateSoc() const { return m_current.chargeRateSoc; }
    QString fanMode() const { return m_current.fanMode; }
    int fanDuty() const { return m_current.fanDuty; }
    int fanRpm() const { return m_current.fanRpm; }
    QString fpLedLevel() const { return m_current.fpLedLevel; }
    int hapticIntensity() const { return m_current.hapticIntensity; }
    QString clickForce() const { return m_current.clickForce; }

    void setChargeLimit(int value);
    void setChargeRateLimit(double value);
    void setChargeRateSoc(int value);
    void setFanMode(const QString &value);
    void setFanDuty(int value);
    void setFanRpm(int value);
    void setFpLedLevel(const QString &value);
    void setHapticIntensity(int value);
    void setClickForce(const QString &value);

    Q_INVOKABLE void clearError();
    // Charge to 100% until the next boot; applied immediately, not on Apply
    Q_INVOKABLE void overrideChargeLimit();
    Q_INVOKABLE void cancelChargeLimitOverride();

Q_SIGNALS:
    void daemonAvailableChanged();
    void busyChanged();
    void errorMessageChanged();
    void liveDataChanged();
    void systemInfoChanged();
    void thermalChanged();
    void portsChanged();
    void supportChanged();
    void settingsChanged();
    void chargeOverrideChanged();

private:
    using ReplyHandler = std::function<void(const QDBusMessage &)>;
    using ErrorHandler = std::function<void(const QDBusError &)>;

    void call(const QString &method,
              const QVariantList &args,
              ReplyHandler onReply,
              ErrorHandler onError = {},
              int timeout = -1);
    void refreshLive();
    void loadSettings();
    template<typename T>
    void syncSetting(T FrameworkSettings::*field, const T &value);
    void runNextWrite();
    void settingsEdited();
    void setError(const QString &message);
    void setWriteError(const QDBusError &error);
    void runAction(const QString &method);
    void setBusy(bool busy);
    void setDaemonAvailable(bool available);

    FrameworkSettings m_current;
    FrameworkSettings m_saved;

    struct PendingWrite {
        QString method;
        QVariantList args;

        bool operator==(const PendingWrite &) const = default;
    };
    static PendingWrite fanWrite(const FrameworkSettings &s);
    QList<PendingWrite> m_writeQueue;

    bool m_daemonAvailable = true;
    bool m_busy = false;
    bool m_liveInFlight = false;
    QString m_errorMessage;
    QString m_liveData;
    QTimer m_liveTimer;

    QVariantMap m_systemInfo;
    QVariantList m_sensors;
    QVariantList m_fans;
    QVariantMap m_throttle;
    QVariantList m_ports;
    bool m_fpLedSupported = false;
    bool m_chargeLimitOverridden = false;
};
