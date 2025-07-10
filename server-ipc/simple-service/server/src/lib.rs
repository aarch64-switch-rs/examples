#![no_std]

use nx::ipc::sf;
use nx::result::Result;
use nx::service::{self, sm};
use nx::version;
use nx::{ipc_sf_define_default_client_for_interface, ipc_sf_define_interface_trait};

ipc_sf_define_default_client_for_interface!(DemoService);
ipc_sf_define_interface_trait! {
    trait DemoService {
        sample_command [999, version::VersionInterval::all(), mut ]: (a: u32, b: u64, c: sf::InAutoSelectBuffer<'_, u8>, d: sf::InOutAutoSelectBuffer<'_, u8>) => () ();
    }
}

impl service::IService for DemoService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("dmo-srv")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}
