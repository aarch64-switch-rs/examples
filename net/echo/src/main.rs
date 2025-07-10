#![no_std]
#![no_main]

extern crate alloc;

use core::net::Ipv4Addr;
use core::panic;

use core::fmt::Write;
use nx::diag::abort;
use nx::diag::log::lm::LmLogger;
use nx::fs::{self, FileOpenOption};

use nx::service::hid;
use nx::socket::net::{TcpListener, traits::SocketCommon};
use nx::{input, svc, thread, util};

nx::rrt0_define_module_name!("echo-server");

#[unsafe(no_mangle)]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    if hbl_heap.is_valid() {
        hbl_heap
    } else {
        let heap_size: usize = 0x10000000;
        let heap_address = svc::set_heap_size(heap_size).unwrap();
        util::PointerAndSize::new(heap_address, heap_size)
    }
}

#[unsafe(no_mangle)]
fn main() {
    fs::initialize_fspsrv_session().expect("Error starting filesystem services");
    fs::mount_sd_card("sdmc").expect("Failed to mount sd card");
    fs::mount_sd_card("sdmc").unwrap();
    let mut log_file = fs::open_file(
        "sdmc:/echo-server.log",
        FileOpenOption::Append() | FileOpenOption::Create() | FileOpenOption::Write(),
    )
    .unwrap();

    let supported_style_tags = hid::NpadStyleTag::Handheld()
        | hid::NpadStyleTag::FullKey()
        | hid::NpadStyleTag::JoyDual()
        | hid::NpadStyleTag::JoyLeft()
        | hid::NpadStyleTag::JoyRight();
    let input_ctx = match input::Context::new(supported_style_tags, 1) {
        Ok(ok) => ok,
        Err(e) => {
            let _ = write!(log_file, "Error getting input context: {:#X}\n", e.get_value());
            return;
        }
    };

    if let Err(e) = nx::socket::initialize(
        nx::service::bsd::BsdSrvkind::System,
        Default::default(),
        None,
    ) {
        let _ = write!(log_file, 
                "Error initializing socket service: {}-{}\n",
                e.get_module(),
                e.get_description()
            );
        return;
    }

    let listener = match TcpListener::bind(Ipv4Addr::UNSPECIFIED, 4660) {
        Ok(l) => l,
        Err(e) => {
            let _ = write!(log_file, 
                    "Error creating listener: {}-{}\n",
                    e.get_module(),
                    e.get_description()
                );
            return;
        }
    };

    match listener.local_addr() {
        Ok(socket) => {
            let _ = write!(log_file, "Listening for TCP connections. Local Address: {:?}\n", socket);
        },
        Err(e) => {
            let _ = write!(log_file, 
                    "Error getting socket name: {}-{}\n",
                    e.get_module(),
                    e.get_description()
            );
            return;
        }
    }

    let (stream, remote_addr) = match listener.accept() {
        Ok(s) => s,
        Err(e) => {
            let _ = write!(log_file, 
                    "Error accepting connection: {}-{}\n",
                    e.get_module(),
                    e.get_description()
                );
            return;
        }
    };

    let _ = write!(log_file, 
            "received connection: IP - {}\n",
            Ipv4Addr::from_bits(u32::from_be_bytes(remote_addr.addr))
        );

    let mut read_buf = [0u8; 0x200];
    loop {
        for controller in [hid::NpadIdType::Handheld, hid::NpadIdType::No1]
            .iter()
            .cloned()
        {
            if input_ctx
                .get_player(controller)
                .get_buttons_down()
                .contains(hid::NpadButton::Plus())
            {
                // Exit if Plus/+ is pressed.
                return;
            }
        }

        match stream.recv_non_blocking(&mut read_buf) {
            Ok(Some(read_len))  if read_len > 0 => {
                let _ = write!(log_file, "read data from the network: {:?}\n", &read_buf[..read_len]);
                let _ = stream.send_non_blocking(&read_buf[..read_len]);
            },
            Ok(_) => {
                let _  = thread::sleep(10_000);
            },
            Err(e) => {
                let _ = write!(log_file, 
                        "Error accepting connection: {}-{}\n",
                        e.get_module(),
                        e.get_description()
                    );
                break;
            }
        }
    }
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}
