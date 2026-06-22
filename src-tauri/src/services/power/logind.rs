//! systemd-logind power actions via zbus (Linux only).

#[cfg(target_os = "linux")]
mod inner {
    use super::super::super::config::PowerAction;
    use zbus::{proxy, Connection, Result as ZResult};

    #[proxy(
        interface = "org.freedesktop.login1.Manager",
        default_service = "org.freedesktop.login1",
        default_path = "/org/freedesktop/login1"
    )]
    trait Login1Manager {
        #[zbus(name = "PowerOff")]
        async fn power_off(&self, interactive: bool) -> ZResult<()>;

        #[zbus(name = "Reboot")]
        async fn reboot(&self, interactive: bool) -> ZResult<()>;

        #[zbus(name = "Suspend")]
        async fn suspend(&self, interactive: bool) -> ZResult<()>;

        #[zbus(name = "Hibernate")]
        async fn hibernate(&self, interactive: bool) -> ZResult<()>;

        #[zbus(name = "HybridSleep")]
        async fn hybrid_sleep(&self, interactive: bool) -> ZResult<()>;
    }

    #[proxy(
        interface = "org.freedesktop.login1.Session",
        default_service = "org.freedesktop.login1"
    )]
    trait Login1Session {
        #[zbus(name = "Lock")]
        async fn lock(&self) -> ZResult<()>;

        #[zbus(name = "Terminate")]
        async fn terminate(&self) -> ZResult<()>;
    }

    pub async fn execute(action: &PowerAction) -> anyhow::Result<()> {
        let conn = Connection::system().await?;

        match action {
            PowerAction::Poweroff => {
                Login1ManagerProxy::new(&conn)
                    .await?
                    .power_off(false)
                    .await?;
            }
            PowerAction::Reboot => {
                Login1ManagerProxy::new(&conn).await?.reboot(false).await?;
            }
            PowerAction::Suspend => {
                Login1ManagerProxy::new(&conn).await?.suspend(false).await?;
            }
            PowerAction::Hibernate => {
                Login1ManagerProxy::new(&conn)
                    .await?
                    .hibernate(false)
                    .await?;
            }
            PowerAction::HybridSleep => {
                Login1ManagerProxy::new(&conn)
                    .await?
                    .hybrid_sleep(false)
                    .await?;
            }
            PowerAction::Lock | PowerAction::Logout => {
                let session_path = session_object_path();
                let proxy = Login1SessionProxy::builder(&conn)
                    .path(session_path)?
                    .build()
                    .await?;
                if matches!(action, PowerAction::Lock) {
                    proxy.lock().await?;
                } else {
                    proxy.terminate().await?;
                }
            }
        }

        Ok(())
    }

    fn session_object_path() -> String {
        // $XDG_SESSION_ID is set by logind for the current session.
        // "auto" is a logind alias that resolves to the caller's session.
        let id = std::env::var("XDG_SESSION_ID").unwrap_or_else(|_| "auto".into());
        format!("/org/freedesktop/login1/session/{id}")
    }
}

/// Execute a power action. No-op on non-Linux platforms (compilation guard only).
pub async fn execute(action: &crate::config::PowerAction) -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    {
        inner::execute(action).await
    }

    #[cfg(not(target_os = "linux"))]
    {
        log::warn!("power action {:?} is a no-op on this platform", action);
        Ok(())
    }
}
