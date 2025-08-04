#![no_std]
#![no_main]

extern crate alloc;

use core::sync::atomic::Ordering;
use core::net::Ipv4Addr;
use core::panic;
use core::sync::atomic::AtomicBool;

use alloc::string::String;
use alloc::sync::Arc;
use alloc::{format, vec};
use nx::thread::sleep;
use core::fmt::Write;
use nx::diag::abort;
use nx::diag::log::lm::LmLogger;
use nx::fs::{self, FileOpenOption};

use nx::service::hid;
use nx::socket::net::TcpStream;
use nx::socket::net::{TcpListener, traits::SocketCommon};
use nx::sync::RwLock;
use nx::{input, svc, thread, util};

nx::rrt0_define_module_name!("chat-server");

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

    let mut log_file = fs::open_file(
        "sdmc:/broadcast-server.log",
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
            let _ = write!(
                log_file,
                "Error getting input context: {:#X}\n",
                e.get_value()
            );
            return;
        }
    };

    if let Err(e) = nx::socket::initialize(
        nx::service::bsd::BsdSrvkind::System,
        Default::default(),
        None,
        nx::socket::Paralellism::Eight,
    ) {
        let _ = write!(
            log_file,
            "Error initializing socket service: {}-{}\n",
            e.get_module(),
            e.get_description()
        );
        return;
    }

    let listener = match TcpListener::bind(Ipv4Addr::UNSPECIFIED, 4660) {
        Ok(l) => l,
        Err(e) => {
            let _ = write!(
                log_file,
                "Error creating listener: {}-{}\n",
                e.get_module(),
                e.get_description()
            );
            return;
        }
    };

    let _ = listener.set_nonblocking(true);

    match listener.local_addr() {
        Ok(socket) => {
            let _ = write!(
                log_file,
                "Listening for TCP connections. Local Address: {:?}\n",
                socket
            );
        }
        Err(e) => {
            let _ = write!(
                log_file,
                "Error getting socket name: {}-{}\n",
                e.get_module(),
                e.get_description()
            );
            return;
        }
    }

    let connection_map: Arc<RwLock<hashbrown::HashMap<i32, Arc<TcpStream>>>> =
    Arc::new(RwLock::new(hashbrown::HashMap::new()));
    let stop = Arc::new(AtomicBool::new(false));

    let mut thread_list = vec![];
    
    'main_loop: loop {
        match listener.accept() {
            Ok((stream, remote_addr)) =>  {
                let fd = stream.as_raw_fd();
                let stream = Arc::new(stream);
                let remote_addr = Ipv4Addr::from_bits(u32::from_be_bytes(remote_addr.addr));

                connection_map.write().insert(fd, stream.clone());

                {
                    let stop = stop.clone();
                    let connections = connection_map.clone();
                
                    thread_list.push(nx::thread::spawn(move || {
                        let mut read_buf = [0u8; 0x200];

                        while !stop.load(Ordering::Relaxed) {

                        match stream.recv_non_blocking(&mut read_buf) {
                            Ok(Some(read_len)) if read_len > 0 => {
                                let guard = connections.read();
                                for (_, connection) in guard.iter().filter(|(id,_)| **id != stream.as_raw_fd()) {
                                    let _res = connection.send(format!("[{}] {}", remote_addr, String::from_utf8_lossy(&read_buf[..read_len])).as_bytes());
                                }
                                
                            },
                            Ok(_) => {
                                let _ = thread::sleep(10_000);
                            },
                            Err(_e) => {
                                // we will just silently leave for now
                                break;
                            }
                        }
                    }
                    connections.write().remove(&stream.as_raw_fd());
                    }));
                }

            },
            Err(e) if e.get_module() == nx::socket::rc::RESULT_MODULE && e.get_description() == 1011 /*EAGAIN */ =>  {
                let _ = sleep(10_000);
            }
            Err(e) => {
                let _ = write!(
                    log_file,
                    "Error accepting connection: {}-{}\n",
                    e.get_module(),
                    e.get_description()
                );
                break 'main_loop;
            }
        };

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
                    break 'main_loop;
                }
            }
    }
    
    stop.store(true, Ordering::Relaxed);

    for joiner in thread_list {
        let _  = joiner.join();
    }
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}
