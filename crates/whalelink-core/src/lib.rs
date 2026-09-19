//! Core configuration, state persistence and EasyTier process ownership.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Stdio,
};
use thiserror::Error;
use tokio::process::{Child, Command};
use whalelink_protocol::DataPlaneEnrollment;

pub const CURRENT_DAEMON_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("EasyTier is already running")]
    AlreadyRunning,
    #[error("EasyTier is not running")]
    NotRunning,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DaemonConfig {
    pub schema_version: u32,
    pub easytier: EasyTierConfig,
    #[serde(default)]
    pub diagnostics: DiagnosticsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EasyTierConfig {
    /// Absolute path to the user-provided, checksum-verified executable.
    pub executable: PathBuf,
    #[serde(default)]
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiagnosticsConfig {
    #[serde(default = "default_redaction")]
    pub redact_sensitive_values: bool,
}

fn default_redaction() -> bool {
    true
}
impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self {
            redact_sensitive_values: true,
        }
    }
}

impl DaemonConfig {
    pub fn parse(contents: &str) -> Result<Self, CoreError> {
        let mut value: toml::Value =
            toml::from_str(contents).map_err(|error| CoreError::Config(error.to_string()))?;
        let version = value
            .get("schema_version")
            .and_then(toml::Value::as_integer)
            .ok_or_else(|| CoreError::Config("schema_version is required".to_owned()))?;
        match version {
            0 => {
                // v0 already used the nested `[easytier]` table. v1 makes
                // diagnostics explicit but retains a serde default, so this
                // migration is lossless and does not touch user credentials.
                value["schema_version"] =
                    toml::Value::Integer(CURRENT_DAEMON_SCHEMA_VERSION.into());
            }
            current if current == i64::from(CURRENT_DAEMON_SCHEMA_VERSION) => {}
            unsupported => {
                return Err(CoreError::Config(format!(
                    "unsupported schema_version {unsupported}"
                )))
            }
        }
        let config: Self = value
            .try_into()
            .map_err(|error: toml::de::Error| CoreError::Config(error.to_string()))?;
        if !config.easytier.executable.is_absolute() {
            return Err(CoreError::Config(
                "easytier.executable must be an absolute path".to_owned(),
            ));
        }
        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RuntimeState {
    pub schema_version: u32,
    pub last_error: Option<String>,
    pub easytier_running: bool,
}

pub fn write_state_atomically(path: &Path, state: &RuntimeState) -> Result<(), CoreError> {
    let parent = path
        .parent()
        .ok_or_else(|| CoreError::Config("state path has no parent".to_owned()))?;
    fs::create_dir_all(parent)?;
    let temp = path.with_extension("json.tmp");
    fs::write(
        &temp,
        serde_json::to_vec_pretty(state).expect("state is serializable"),
    )?;
    fs::rename(temp, path)?;
    Ok(())
}

pub struct EasyTierProcess {
    config: EasyTierConfig,
    child: Option<Child>,
    #[cfg(windows)]
    job: Option<KillOnDropJob>,
}

impl EasyTierProcess {
    pub fn new(config: EasyTierConfig) -> Self {
        Self {
            config,
            child: None,
            #[cfg(windows)]
            job: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), CoreError> {
        if self.child.is_some() {
            return Err(CoreError::AlreadyRunning);
        }
        let working_directory = self
            .config
            .executable
            .parent()
            .filter(|path| !path.as_os_str().is_empty());
        let mut command = Command::new(&self.config.executable);
        if let Some(directory) = working_directory {
            // EasyTier ships native helper DLLs beside easytier-core. Running
            // from that directory makes the portable package self-contained.
            command.current_dir(directory);
        }
        let child = command
            .args(&self.config.arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()?;
        #[cfg(windows)]
        {
            let job = KillOnDropJob::new()?;
            job.assign(&child)?;
            self.job = Some(job);
        }
        self.child = Some(child);
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), CoreError> {
        if self.child.is_none() {
            return Err(CoreError::NotRunning);
        }
        if self.observe_exit()? {
            return Ok(());
        }
        let child = self.child.as_mut().expect("child was checked above");
        child.kill().await?;
        self.child = None;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }

    /// Reaps an exited child and reports whether it ended since the previous
    /// observation. Supervisors call this before reporting health or deciding
    /// whether a restart is necessary.
    pub fn observe_exit(&mut self) -> Result<bool, CoreError> {
        let exited = match self.child.as_mut() {
            Some(child) => child.try_wait()?.is_some(),
            None => false,
        };
        if exited {
            self.child = None;
        }
        Ok(exited)
    }
}

/// A Windows Job Object makes a force-closed daemon take its EasyTier child
/// with it. This prevents a stale virtual adapter process from surviving an
/// updater, terminal close, or desktop crash.
#[cfg(windows)]
struct KillOnDropJob(windows_sys::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl KillOnDropJob {
    fn new() -> Result<Self, CoreError> {
        use std::{mem::size_of, ptr};
        use windows_sys::Win32::{
            Foundation::CloseHandle,
            System::JobObjects::{
                CreateJobObjectW, SetInformationJobObject, JobObjectExtendedLimitInformation,
                JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            },
        };

        let handle = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
        if handle.is_null() {
            return Err(CoreError::Io(std::io::Error::last_os_error()));
        }
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = unsafe {
            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if configured == 0 {
            unsafe { CloseHandle(handle) };
            return Err(CoreError::Io(std::io::Error::last_os_error()));
        }
        Ok(Self(handle))
    }

    fn assign(&self, child: &Child) -> Result<(), CoreError> {
        use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;

        let process_handle = child
            .raw_handle()
            .ok_or_else(|| CoreError::Config("EasyTier child has no Windows process handle".into()))?;
        let assigned = unsafe { AssignProcessToJobObject(self.0, process_handle.cast()) };
        if assigned == 0 {
            return Err(CoreError::Io(std::io::Error::last_os_error()));
        }
        Ok(())
    }
}

#[cfg(windows)]
impl Drop for KillOnDropJob {
    fn drop(&mut self) {
        unsafe { windows_sys::Win32::Foundation::CloseHandle(self.0) };
    }
}

pub fn redact_text(value: &str) -> String {
    const SENSITIVE_KEYS: [&str; 4] = ["token", "password", "secret", "invite"];
    value
        .lines()
        .map(|line| {
            let lower = line.to_ascii_lowercase();
            if SENSITIVE_KEYS.iter().any(|key| lower.contains(key)) {
                "[REDACTED]"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Current-user encrypted enrollment storage. The Windows implementation uses
/// DPAPI; other platforms intentionally do not expose this client-only store.
#[cfg(windows)]
pub struct EnrollmentStore {
    directory: PathBuf,
}

#[cfg(windows)]
impl EnrollmentStore {
    pub fn current_user() -> Result<Self, CoreError> {
        let local_app_data = std::env::var_os("LOCALAPPDATA").ok_or_else(|| {
            CoreError::Config("LOCALAPPDATA is required for current-user credentials".into())
        })?;
        Ok(Self {
            directory: PathBuf::from(local_app_data)
                .join("WhaleLink")
                .join("credentials"),
        })
    }

    pub fn save(&self, room_id: &str, enrollment: &DataPlaneEnrollment) -> Result<(), CoreError> {
        if room_id.trim().is_empty() {
            return Err(CoreError::Config("room_id cannot be empty".into()));
        }
        fs::create_dir_all(&self.directory)?;
        protect_current_user_directory(&self.directory)?;
        let path = self.path_for(room_id);
        let plaintext = serde_json::to_vec(enrollment).map_err(|error| {
            CoreError::Config(format!("enrollment serialization failed: {error}"))
        })?;
        let protected = protect_current_user(&plaintext)?;
        let temporary = path.with_extension("bin.tmp");
        fs::write(&temporary, protected)?;
        fs::rename(temporary, path)?;
        Ok(())
    }

    pub fn load(&self, room_id: &str) -> Result<DataPlaneEnrollment, CoreError> {
        let protected = fs::read(self.path_for(room_id))?;
        let plaintext = unprotect_current_user(&protected)?;
        serde_json::from_slice(&plaintext)
            .map_err(|error| CoreError::Config(format!("invalid protected enrollment: {error}")))
    }

    fn path_for(&self, room_id: &str) -> PathBuf {
        let digest = format!("{:x}", Sha256::digest(room_id.as_bytes()));
        self.directory.join(format!("{digest}.bin"))
    }
}

/// Replaces inherited directory permissions with an owner-only DACL. The
/// encrypted blob remains protected by DPAPI as a second, independent layer.
#[cfg(windows)]
fn protect_current_user_directory(directory: &Path) -> Result<(), CoreError> {
    use std::{ffi::c_void, os::windows::ffi::OsStrExt, ptr};
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::{
            Authorization::{
                ConvertStringSecurityDescriptorToSecurityDescriptorW, SetNamedSecurityInfoW,
                SDDL_REVISION_1, SE_FILE_OBJECT,
            },
            GetSecurityDescriptorDacl, DACL_SECURITY_INFORMATION,
            PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
        },
    };

    let path: Vec<u16> = directory.as_os_str().encode_wide().chain(Some(0)).collect();
    // OW resolves to the security descriptor's owner, which is the current
    // user for the LocalAppData directory created by this process.
    let sddl: Vec<u16> = "D:P(A;;GA;;;OW)".encode_utf16().chain(Some(0)).collect();
    let mut descriptor: PSECURITY_DESCRIPTOR = ptr::null_mut();
    let converted = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            ptr::null_mut(),
        )
    };
    if converted == 0 {
        return Err(CoreError::Io(std::io::Error::last_os_error()));
    }
    let mut present = 0;
    let mut defaulted = 0;
    let mut dacl = ptr::null_mut();
    let got_dacl =
        unsafe { GetSecurityDescriptorDacl(descriptor, &mut present, &mut dacl, &mut defaulted) };
    if got_dacl == 0 || present == 0 || dacl.is_null() {
        unsafe { LocalFree(descriptor.cast::<c_void>()) };
        return Err(CoreError::Io(std::io::Error::last_os_error()));
    }
    let result = unsafe {
        SetNamedSecurityInfoW(
            path.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            dacl,
            ptr::null(),
        )
    };
    unsafe { LocalFree(descriptor.cast::<c_void>()) };
    if result != 0 {
        return Err(CoreError::Io(std::io::Error::from_raw_os_error(
            result as i32,
        )));
    }
    Ok(())
}

#[cfg(windows)]
fn protect_current_user(plaintext: &[u8]) -> Result<Vec<u8>, CoreError> {
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::Cryptography::{CryptProtectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB},
    };

    let mut input = plaintext.to_vec();
    let input_blob = CRYPT_INTEGER_BLOB {
        cbData: input
            .len()
            .try_into()
            .map_err(|_| CoreError::Config("enrollment is too large".into()))?,
        pbData: input.as_mut_ptr(),
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    let protected = unsafe {
        CryptProtectData(
            &input_blob,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if protected == 0 {
        return Err(CoreError::Io(std::io::Error::last_os_error()));
    }
    let bytes =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe { LocalFree(output.pbData.cast()) };
    Ok(bytes)
}

#[cfg(windows)]
fn unprotect_current_user(protected: &[u8]) -> Result<Vec<u8>, CoreError> {
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::Cryptography::{
            CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
        },
    };

    let mut input = protected.to_vec();
    let input_blob = CRYPT_INTEGER_BLOB {
        cbData: input
            .len()
            .try_into()
            .map_err(|_| CoreError::Config("protected enrollment is too large".into()))?,
        pbData: input.as_mut_ptr(),
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    let unprotected = unsafe {
        CryptUnprotectData(
            &input_blob,
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if unprotected == 0 {
        return Err(CoreError::Io(std::io::Error::last_os_error()));
    }
    let bytes =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe { LocalFree(output.pbData.cast()) };
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_relative_binary_path() {
        let result =
            DaemonConfig::parse("schema_version = 1\n[easytier]\nexecutable = 'easytier-core'\n");
        assert!(matches!(result, Err(CoreError::Config(_))));
    }

    #[test]
    fn redacts_sensitive_lines() {
        assert_eq!(redact_text("mode=ok\ntoken=secret"), "mode=ok\n[REDACTED]");
    }

    #[test]
    fn state_can_be_serialized() {
        let state = RuntimeState {
            schema_version: CURRENT_DAEMON_SCHEMA_VERSION,
            last_error: None,
            easytier_running: false,
        };
        assert!(serde_json::to_string(&state).is_ok());
    }

    #[test]
    fn migrates_schema_zero_to_current_schema() {
        let config = DaemonConfig::parse(
            "schema_version = 0\n[easytier]\nexecutable = 'C:/Program Files/WhaleLink/easytier-core.exe'\n",
        )
        .unwrap();
        assert_eq!(config.schema_version, CURRENT_DAEMON_SCHEMA_VERSION);
        assert!(config.diagnostics.redact_sensitive_values);
    }

    #[test]
    fn state_atomic_write_replaces_existing_state() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("whalelink-runtime-{suffix}.json"));
        write_state_atomically(
            &path,
            &RuntimeState {
                schema_version: CURRENT_DAEMON_SCHEMA_VERSION,
                last_error: Some("first".into()),
                easytier_running: false,
            },
        )
        .unwrap();
        write_state_atomically(
            &path,
            &RuntimeState {
                schema_version: CURRENT_DAEMON_SCHEMA_VERSION,
                last_error: None,
                easytier_running: true,
            },
        )
        .unwrap();
        let state: RuntimeState = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert!(state.easytier_running);
        assert_eq!(state.last_error, None);
        fs::remove_file(path).unwrap();
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn exited_child_is_reaped_before_supervisor_restart() {
        let mut process = EasyTierProcess::new(EasyTierConfig {
            executable: PathBuf::from(r"C:\Windows\System32\cmd.exe"),
            arguments: vec!["/C".into(), "exit 0".into()],
        });
        process.start().await.unwrap();
        for _ in 0..20 {
            if process.observe_exit().unwrap() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        assert!(!process.is_running());
        assert!(matches!(process.stop().await, Err(CoreError::NotRunning)));
    }

    #[cfg(windows)]
    #[test]
    fn dpapi_round_trip_preserves_enrollment_without_plaintext_storage() {
        let protected = protect_current_user(b"whalelink-test-secret").unwrap();
        assert!(!String::from_utf8_lossy(&protected).contains("whalelink-test-secret"));
        assert_eq!(
            unprotect_current_user(&protected).unwrap(),
            b"whalelink-test-secret"
        );
    }
}
