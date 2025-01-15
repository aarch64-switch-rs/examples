use nx::result::*;
use nx::arm;
use nx::sync;
use nx::diag::log;
use nx::fs;
use nx::thread;
use alloc::string::String;

const BASE_LOG_DIR: &'static str = "sdmc:/lm-binlogs";

static mut G_ENABLED: sync::Mutex<bool> = sync::Mutex::new(true);

pub fn set_log_enabled(enabled: bool) {
    unsafe {
        G_ENABLED.set(enabled);
    }
}

const LOG_BINARY_HEADER_MAGIC: u32 = 0x70687068;
const CURRENT_VERSION: u32 = 1;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LogBinaryHeader {
    magic: u32,
    version: u32
}

impl LogBinaryHeader {
    pub const fn new(magic: u32, version: u32) -> Self {
        Self { magic: magic, version: version }
    }
}

pub fn initialize() -> Result<()> {
    let _ = fs::remove_dir(BASE_LOG_DIR);
    fs::create_directory(BASE_LOG_DIR)
}

fn log_packet_buf_impl(packet_buf: *const u8, packet_buf_size: usize, bin_header: LogBinaryHeader, log_dir: String, log_buf_file: String) -> Result<()> {
    unsafe {
        if G_ENABLED.get_val() {
            let _ = fs::create_directory(BASE_LOG_DIR);
            let _ = fs::create_directory(log_dir.as_str());

            let _ = fs::remove_file(log_buf_file.as_str());

            let mut log_file = fs::open_file(log_buf_file.as_str(), fs::FileOpenOption::Create() | fs::FileOpenOption::Write() | fs::FileOpenOption::Append())?;
            log_file.write_val(&bin_header)?;
            log_file.write_array(core::slice::from_raw_parts(packet_buf, packet_buf_size))?;
        }
    }
    Ok(())
}

fn log_self_impl(self_msg: String, log_dir: String, log_buf_file: String) -> Result<()> {
    unsafe {
        if G_ENABLED.get_val() {
            let _ = fs::create_directory(BASE_LOG_DIR);
            let _ = fs::create_directory(log_dir.as_str());

            let mut log_file = fs::open_file(log_buf_file.as_str(), fs::FileOpenOption::Create() | fs::FileOpenOption::Write() | fs::FileOpenOption::Append())?;
            log_file.write_array(self_msg.as_bytes())?;
        }
    }
    Ok(())
}

pub fn log_packet_buf(packet_buf: *const u8, packet_buf_size: usize, program_id: u64) {
    let log_timestamp = arm::get_system_tick();
    let process_log_dir = format!("{}/0x{:016X}", BASE_LOG_DIR, program_id);
    let log_buf_path = format!("{}/0x{:016X}.nxbinlog", process_log_dir, log_timestamp);

    let _ = log_packet_buf_impl(packet_buf, packet_buf_size, LogBinaryHeader::new(LOG_BINARY_HEADER_MAGIC, CURRENT_VERSION), process_log_dir, log_buf_path);
}

pub fn log_self(self_msg: String) {
    let process_log_dir = format!("{}/self-logs", BASE_LOG_DIR);
    let log_buf_path = format!("{}/self.log", process_log_dir);

    let _ = log_self_impl(self_msg, process_log_dir, log_buf_path);
}

// System for LogManager to be able to log stuff itself (even if it gets saved in a different way)

pub struct SelfLogger;

impl log::Logger for SelfLogger {
    fn new() -> Self {
        Self {}
    }

    fn log(&mut self, metadata: &log::LogMetadata) {
        let severity_str = match metadata.severity {
            log::LogSeverity::Trace => "Trace",
            log::LogSeverity::Info => "Info",
            log::LogSeverity::Warn => "Warn",
            log::LogSeverity::Error => "Error",
            log::LogSeverity::Fatal => "Fatal",
        };
        let msg = format!("[ SelfLog (severity: {}, verbosity: {}) from {} in thread {}, at {}:{} ] {}\n", severity_str, metadata.verbosity, metadata.fn_name, thread::get_current_thread_name(), metadata.file_name, metadata.line_number, metadata.msg);
        log_self(msg);
    }
}