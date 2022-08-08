use nx::result::*;
use nx::mem;
use nx::ipc::sf;
use nx::ipc::server;
use nx::ipc::sf::lm;
use nx::ipc::sf::lm::ILogger;
use nx::ipc::sf::lm::ILogService;
use nx::diag::log;
use nx::service;
use nx::service::sm;
use nx::service::pm;
use nx::service::pm::IInformationInterface;
use crate::logger;

pub struct Logger {
    log_destination: lm::LogDestination,
    program_id: u64,
    dummy_session: sf::Session
}

impl Logger {
    pub fn new(program_id: u64) -> Self {
        Self { log_destination: lm::LogDestination::Tma(), program_id, dummy_session: sf::Session::new() }
    }
}

impl sf::IObject for Logger {
    ipc_sf_object_impl_default_command_metadata!();

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.dummy_session
    }
}

impl ILogger for Logger {
    fn log(&mut self, log_buf: sf::InAutoSelectBuffer<u8>) -> Result<()> {
        diag_log!(logger::SelfLogger { log::LogSeverity::Trace, false } => "Logging with buffer ({:p}, 0x{:X})", log_buf.get_address(), log_buf.get_size());

        logger::log_packet_buf(log_buf.get_address(), log_buf.get_size(), self.program_id);
        Ok(())
    }

    fn set_destination(&mut self, log_destination: lm::LogDestination) -> Result<()> {
        // Note: in official code, log destination is a global variable (not stored in the logger interface like here)
        // TODO: make use of log destination?
        diag_log!(logger::SelfLogger { log::LogSeverity::Trace, false } => "Setting destination 0x{:X}", log_destination.get());
        self.log_destination = log_destination;

        Ok(())
    }
}

pub struct LogService {
    dummy_session: sf::Session
}

impl sf::IObject for LogService {
    ipc_sf_object_impl_default_command_metadata!();

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.dummy_session
    }
}

impl ILogService for LogService {
    fn open_logger(&mut self, process_id: sf::ProcessId) -> Result<mem::Shared<dyn ILogger>> {
        let pminfo = service::new_service_object::<pm::InformationInterface>()?;
        let program_id = pminfo.get().get_program_id(process_id.process_id)?;
        diag_log!(logger::SelfLogger { log::LogSeverity::Trace, false } => "Opening logger for program ID 0x{:016X}", program_id);

        Ok(mem::Shared::new(Logger::new(program_id)))
    }
}

impl server::ISessionObject for LogService {}

impl server::IServerObject for LogService {
    fn new() -> Self {
        Self {
            dummy_session: sf::Session::new()
        }
    }
}

impl server::IService for LogService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("lm")
    }

    fn get_max_sesssions() -> i32 {
        42
    }
}