use nx::result::*;
use nx::ipc::sf;
use nx::fs;
use nx::ipc::server;
use nx::ipc::sf::sm;
use nx::diag::log::{lm::LmLogger, LogSeverity};
use nx::version;

// TODO: move this interface to nx libs (and finish it)...

ipc_sf_define_default_interface_client!(PrepoService);
ipc_sf_define_interface_trait! {
	trait PrepoService {
        save_report_old [10100, version::VersionInterval::all(), mut ]: (process_id: sf::ProcessId, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) => () ();
        save_report_with_user_old [10101, version::VersionInterval::all(), mut ]: (user_id: u128, process_id: sf::ProcessId, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) => () ();
        save_report_old_2 [10102, version::VersionInterval::all(), mut ]: (process_id: sf::ProcessId, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) => () ();
        save_report_with_user_old_2 [10103, version::VersionInterval::all(), mut ]: (user_id: u128, process_id: sf::ProcessId, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) => () ();
        save_report [10104, version::VersionInterval::all(), mut ]: (process_id: sf::ProcessId, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) => () ();
        save_report_with_user [10105, version::VersionInterval::all(), mut ]: (user_id: u128, process_id: sf::ProcessId, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) => () ();
        request_immediate_transmission [10200, version::VersionInterval::all(), mut ]: () => () ();
        get_transmission_status [10300, version::VersionInterval::all(), mut ]: () => (status: u32) (status: u32);
        get_system_session_id [10400, version::VersionInterval::all(), mut ]: () => (id: u64) (id: u64);
        save_system_report [20100, version::VersionInterval::all(), mut ]: (application_id: u64, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) => () ();
        save_system_report_with_user [20101, version::VersionInterval::all(), mut ]: (user_id: u128, application_id: u64, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) => () ();
    }
}

pub const SERVICE_TYPE_ADMIN: u32 = 1;
pub const SERVICE_TYPE_ADMIN2: u32 = 2;
pub const SERVICE_TYPE_MANAGER: u32 = 3;
pub const SERVICE_TYPE_USER: u32 = 4;
pub const SERVICE_TYPE_SYSTEM: u32 = 5;

#[inline]
fn get_service_name<const S: u32>() -> &'static str {
    match S {
        SERVICE_TYPE_ADMIN => nul!("prepo:a"),
        SERVICE_TYPE_ADMIN2 => nul!("prepo:a2"),
        SERVICE_TYPE_MANAGER => nul!("prepo:m"),
        SERVICE_TYPE_USER => nul!("prepo:u"),
        SERVICE_TYPE_SYSTEM => nul!("prepo:s"),
        _ => nul!("")
    }
}

#[inline]
fn get_non_null_service_name<const S: u32>() -> &'static str {
    get_service_name::<S>().trim_matches('\0')
}

#[derive(Debug)]
pub enum ReportKind {
    Normal,
    System
}

pub struct ReportContext {
    pub kind: ReportKind,
    pub process_id: Option<u64>,
    pub application_id: Option<u64>,
    pub room_str_buf: sf::InPointerBuffer<u8>,
    pub report_msgpack_buf: sf::InMapAliasBuffer<u8>,
    pub user_id: Option<u128>
}

pub struct PrepoServiceMitmServer<const S: u32> {
    _info: sm::mitm::MitmProcessInfo
}

impl<const S: u32> PrepoServiceMitmServer<S> {
    fn process_report(&self, ctx: ReportContext) {
        let mut idx = 1;
        let mut msgpack_path = format!("sdmc:/prepo/{:#X}-{:#X}-{:?}.msgpack", ctx.process_id.unwrap_or(0), ctx.application_id.unwrap_or(0), ctx.kind);
        while fs::get_entry_type(msgpack_path.as_str()).is_ok() {
            msgpack_path = format!("sdmc:/prepo/{:#X}-{:#X}-{:?}-{}.msgpack", ctx.process_id.unwrap_or(0), ctx.application_id.unwrap_or(0), ctx.kind, idx);
            idx += 1;
        }

        if let Ok(mut msgpack_file) = fs::open_file(msgpack_path.as_str(), fs::FileOpenOption::Create() | fs::FileOpenOption::Write() | fs::FileOpenOption::Append()) {
            msgpack_file.write_array(unsafe {ctx.report_msgpack_buf.get_slice()}).unwrap();
        }

        diag_log!(LmLogger { LogSeverity::Info, true } => "\nREPORT START\n");

        diag_log!(LmLogger { LogSeverity::Info, true } => "Kind: {:?}\n", ctx.kind);
        diag_log!(LmLogger { LogSeverity::Info, true } => "Room: {}\n", ctx.room_str_buf.get_string());
        diag_log!(LmLogger { LogSeverity::Info, true } => "Msgpack size: {}\n", ctx.report_msgpack_buf.get_size());
        
        if let Some(process_id) = ctx.process_id {
            diag_log!(LmLogger { LogSeverity::Info, true } => "Process (ID) sending the report: {:#X}\n", process_id);
        }
        if let Some(application_id) = ctx.application_id {
            diag_log!(LmLogger { LogSeverity::Info, true } => "Application (ID) sending the report: {:#X}\n", application_id);
        }
        if let Some(_user_id) = ctx.user_id {
            let user_name = "TODOUser";
            diag_log!(LmLogger { LogSeverity::Info, true } => "User sending the report: {}\n", user_name);
        }

        diag_log!(LmLogger { LogSeverity::Info, true } => "REPORT END\n");
    }

    fn save_report_impl(&self, process_id: sf::ProcessId, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) -> Result<()> {
        let ctx = ReportContext {
            kind: ReportKind::Normal,
            process_id: Some(process_id.process_id),
            application_id: None,
            room_str_buf: room_str_buf,
            report_msgpack_buf: report_msgpack_buf,
            user_id: None
        };
        self.process_report(ctx);
        Ok(())
    }

    fn save_report_with_user_impl(&self, user_id: u128, process_id: sf::ProcessId, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) -> Result<()> {
        let ctx = ReportContext {
            kind: ReportKind::Normal,
            process_id: Some(process_id.process_id),
            application_id: None,
            room_str_buf: room_str_buf,
            report_msgpack_buf: report_msgpack_buf,
            user_id: Some(user_id)
        };
        self.process_report(ctx);
        Ok(())
    }

    fn save_system_report_impl(&self, application_id: u64, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) -> Result<()> {
        let ctx = ReportContext {
            kind: ReportKind::System,
            process_id: None,
            application_id: Some(application_id),
            room_str_buf: room_str_buf,
            report_msgpack_buf: report_msgpack_buf,
            user_id: None
        };
        self.process_report(ctx);
        Ok(())
    }

    fn save_system_report_with_user_impl(&self, user_id: u128, application_id: u64, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) -> Result<()> {
        let ctx = ReportContext {
            kind: ReportKind::System,
            process_id: None,
            application_id: Some(application_id),
            room_str_buf: room_str_buf,
            report_msgpack_buf: report_msgpack_buf,
            user_id: Some(user_id)
        };
        self.process_report(ctx);
        Ok(())
    }
}

impl<const S: u32> IPrepoServiceServer for PrepoServiceMitmServer<S> {
    fn save_report_old(&mut self, process_id: sf::ProcessId, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) -> Result<()> {
        self.save_report_impl(process_id, room_str_buf, report_msgpack_buf)
    }

    fn save_report_with_user_old(&mut self, user_id: u128, process_id: sf::ProcessId, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) -> Result<()> {
        self.save_report_with_user_impl(user_id, process_id, room_str_buf, report_msgpack_buf)
    }

    fn save_report_old_2(&mut self, process_id: sf::ProcessId, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) -> Result<()> {
        self.save_report_impl(process_id, room_str_buf, report_msgpack_buf)
    }

    fn save_report_with_user_old_2(&mut self, user_id: u128, process_id: sf::ProcessId, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) -> Result<()> {
        self.save_report_with_user_impl(user_id, process_id, room_str_buf, report_msgpack_buf)
    }

    fn save_report(&mut self, process_id: sf::ProcessId, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) -> Result<()> {
        self.save_report_impl(process_id, room_str_buf, report_msgpack_buf)
    }

    fn save_report_with_user(&mut self, user_id: u128, process_id: sf::ProcessId, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) -> Result<()> {
        self.save_report_with_user_impl(user_id, process_id, room_str_buf, report_msgpack_buf)
    }

    fn request_immediate_transmission(&mut self) -> Result<()> {
        diag_log!(LmLogger { LogSeverity::Info, true } => "\nRequesting immediate transmission...\n");
        Ok(())
    }

    fn get_transmission_status(&mut self) -> Result<u32> {
        diag_log!(LmLogger { LogSeverity::Info, true } => "\nSending transmission status...\n");
        Ok(0)
    }

    fn get_system_session_id(&mut self) -> Result<u64> {
        diag_log!(LmLogger { LogSeverity::Info, true } => "\nSending session ID...\n");
        Ok(0xBABABEBE)
    }

    fn save_system_report(&mut self, application_id: u64, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) -> Result<()> {
        self.save_system_report_impl(application_id, room_str_buf, report_msgpack_buf)
    }

    fn save_system_report_with_user(&mut self, user_id: u128, application_id: u64, room_str_buf: sf::InPointerBuffer<u8>, report_msgpack_buf: sf::InMapAliasBuffer<u8>) -> Result<()> {
        self.save_system_report_with_user_impl(user_id, application_id, room_str_buf, report_msgpack_buf)
    }
}

impl<const S: u32> server::ISessionObject for PrepoServiceMitmServer<S> {
    fn try_handle_request_by_id(&mut self, req_id: u32, protocol: nx::ipc::CommandProtocol, server_ctx: &mut server::ServerContext) -> Option<Result<()>> {
        <Self as IPrepoServiceServer>::try_handle_request_by_id(self, req_id, protocol, server_ctx)
    }
}

impl<const S: u32> server::IMitmServerObject for PrepoServiceMitmServer<S> {
    fn new(info: sm::mitm::MitmProcessInfo) -> Self {
        diag_log!(LmLogger { LogSeverity::Info, true } => "Opened '{}' from program {:#X}\n", get_non_null_service_name::<S>(), info.program_id.0);
        Self {
            _info: info
        }
    }
}

impl<const S: u32> server::IMitmService for PrepoServiceMitmServer<S> {
    fn get_name() -> sm::ServiceName {
        let name = get_service_name::<S>();
        diag_log!(LmLogger { LogSeverity::Info, true } => "Registering mitm at service '{}'...\n", get_non_null_service_name::<S>());
        sm::ServiceName::new(name)
    }

    fn should_mitm(_info: sm::mitm::MitmProcessInfo) -> bool {
        true
    }
}