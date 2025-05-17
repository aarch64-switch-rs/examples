use crate::logger;
use nx::diag::log;
use nx::ipc::server;
use nx::ipc::server::ISessionObject;
use nx::ipc::sf;
use nx::ipc::sf::lm;
use nx::result::*;
use nx::service;
use nx::service::lm::ILoggerServer;
use nx::service::lm::ILoggingServer;
use nx::service::pm;
use nx::service::pm::IInformationInterfaceClient;
use nx::service::sm;

pub struct BinaryFileLogger {
    log_destination: lm::LogDestination,
    program_id: u64,
}

impl BinaryFileLogger {
    pub fn new(program_id: u64) -> Self {
        Self {
            log_destination: lm::LogDestination::Tma(),
            program_id,
        }
    }
}

impl ILoggerServer for BinaryFileLogger {
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

impl ISessionObject for BinaryFileLogger {
    fn try_handle_request_by_id(
        &mut self,
        req_id: u32,
        protocol: nx::ipc::CommandProtocol,
        server_ctx: &mut server::ServerContext,
    ) -> Option<Result<()>> {
        <Self as ILoggerServer>::try_handle_request_by_id(self, req_id, protocol, server_ctx)
    }
}

pub struct LogService;

impl ILoggingServer for LogService {
    fn open_logger(
        &mut self,
        process_id: u64,
    ) -> Result<impl ILoggerServer + 'static + ISessionObject> {
        let pminfo = service::new_service_object::<pm::InformationInterface>()?;
        let program_id = pminfo.get_program_id(process_id)?;
        diag_log!(logger::SelfLogger { log::LogSeverity::Trace, false } => "Opening logger for program ID 0x{:016X}", program_id.0);

        Ok(BinaryFileLogger::new(program_id.0))
    }
}

impl server::ISessionObject for LogService {
    fn try_handle_request_by_id(
        &mut self,
        req_id: u32,
        protocol: nx::ipc::CommandProtocol,
        server_ctx: &mut server::ServerContext,
    ) -> Option<Result<()>> {
        <Self as ILoggingServer>::try_handle_request_by_id(self, req_id, protocol, server_ctx)
    }
}

impl server::IServerObject for LogService {
    fn new() -> Self {
        Self
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
