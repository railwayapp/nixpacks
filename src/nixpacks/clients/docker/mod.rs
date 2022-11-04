use bollard::Docker as BollardDocker;

pub struct Docker {
    pub client: BollardDocker,
}

impl Docker {
    pub fn new() -> Docker {
        Docker {
            // Todo: Consume flag somehow. For now, chaos.
            client: BollardDocker::connect_with_local_defaults().unwrap(),
        }
    }

    pub async fn healthy(&self) -> bool {
        self.client.version().await.is_ok()
    }
}

impl Default for Docker {
    fn default() -> Self {
        Self::new()
    }
}
