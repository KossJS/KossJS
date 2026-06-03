//! KossJS Internal Bindings - Native implementations for Node.js style bindings

pub mod fs {
    use std::fs as std_fs;
    use std::time::SystemTime;

    #[derive(Debug)]
    pub struct Stats {
        pub dev: u64,
        pub ino: u64,
        pub mode: u32,
        pub nlink: u32,
        pub uid: u32,
        pub gid: u32,
        pub rdev: u64,
        pub size: u64,
        pub blksize: u64,
        pub blocks: u64,
        pub atime: u64,
        pub mtime: u64,
        pub ctime: u64,
        pub birthtime: u64,
    }

    impl Stats {
        pub fn from_path(path: &str) -> Result<Stats, String> {
            let metadata = std_fs::metadata(path).map_err(|e| e.to_string())?;

            let atime = metadata
                .accessed()
                .unwrap_or(SystemTime::UNIX_EPOCH)
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let mtime = metadata
                .modified()
                .unwrap_or(SystemTime::UNIX_EPOCH)
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let ctime = metadata
                .created()
                .unwrap_or(SystemTime::UNIX_EPOCH)
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            Ok(Stats {
                dev: 0,
                ino: 0,
                mode: 0o100644,
                nlink: 1,
                uid: 0,
                gid: 0,
                rdev: 0,
                size: 0,
                blksize: 4096,
                blocks: 0,
                atime,
                mtime,
                ctime,
                birthtime: ctime,
            })
        }
    }

    pub fn access(path: &str, _mode: u32) -> Result<(), String> {
        std_fs::metadata(path).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn exists_sync(path: &str) -> bool {
        std_fs::metadata(path).is_ok()
    }

    pub fn read_file_utf8(path: &str, _flags: u32) -> Result<String, String> {
        std_fs::read_to_string(path).map_err(|e| e.to_string())
    }

    pub fn open(_path: &str, _flags: u32, _mode: u32) -> Result<u32, String> {
        Ok(1) // Fake fd
    }

    pub fn close(_fd: u32) -> Result<(), String> {
        Ok(())
    }

    pub fn read(
        _fd: u32,
        _buffer: &mut [u8],
        _offset: u32,
        _length: u32,
        _position: i64,
    ) -> Result<u32, String> {
        Ok(0)
    }

    pub fn write_buffer(
        _fd: u32,
        data: &[u8],
        _offset: u32,
        _length: u32,
        _position: i64,
    ) -> Result<u32, String> {
        Ok(data.len() as u32)
    }

    pub fn write_string(
        _fd: u32,
        data: &str,
        _offset: i32,
        _encoding: &str,
    ) -> Result<u32, String> {
        Ok(data.len() as u32)
    }

    pub fn rename(old_path: &str, new_path: &str) -> Result<(), String> {
        std_fs::rename(old_path, new_path).map_err(|e| e.to_string())
    }

    pub fn rename_sync(old_path: &str, new_path: &str) -> Result<(), String> {
        std_fs::rename(old_path, new_path).map_err(|e| e.to_string())
    }

    pub fn unlink(path: &str) -> Result<(), String> {
        std_fs::remove_file(path).map_err(|e| e.to_string())
    }

    pub fn unlink_sync(path: &str) -> Result<(), String> {
        std_fs::remove_file(path).map_err(|e| e.to_string())
    }

    pub fn mkdir(path: &str, _mode: u32, recursive: bool) -> Result<String, String> {
        if recursive {
            std_fs::create_dir_all(path).map_err(|e| e.to_string())?;
        } else {
            std_fs::create_dir(path).map_err(|e| e.to_string())?;
        }
        Ok(path.to_string())
    }

    pub fn mkdir_sync(path: &str, _mode: u32, recursive: bool) -> Result<String, String> {
        if recursive {
            std_fs::create_dir_all(path).map_err(|e| e.to_string())?;
        } else {
            std_fs::create_dir(path).map_err(|e| e.to_string())?;
        }
        Ok(path.to_string())
    }

    pub fn rmdir(path: &str) -> Result<(), String> {
        std_fs::remove_dir(path).map_err(|e| e.to_string())
    }

    pub fn rmdir_sync(path: &str) -> Result<(), String> {
        std_fs::remove_dir(path).map_err(|e| e.to_string())
    }

    pub fn readdir(
        path: &str,
        _encoding: &str,
        with_file_types: bool,
    ) -> Result<serde_json::Value, String> {
        let entries = std_fs::read_dir(path).map_err(|e| e.to_string())?;

        let mut result = Vec::new();

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            if with_file_types {
                let file_type = entry.file_type().map_err(|e| e.to_string())?;
                let mode = if file_type.is_dir() {
                    0o40000
                } else {
                    0o100000
                };
                result.push(serde_json::json!([name, mode]));
            } else {
                result.push(serde_json::json!(name));
            }
        }

        Ok(serde_json::Value::Array(result))
    }

    pub fn stat(
        path: &str,
        _bigint: bool,
        _throw_if_no_entry: bool,
    ) -> Result<serde_json::Value, String> {
        let stats = Stats::from_path(path)?;

        Ok(serde_json::json!([
            stats.dev,
            stats.ino,
            stats.mode,
            stats.nlink,
            stats.uid,
            stats.gid,
            stats.rdev,
            stats.size,
            stats.blksize,
            stats.blocks,
            stats.atime,
            stats.mtime,
            stats.ctime,
            stats.birthtime,
        ]))
    }

    pub fn lstat(
        path: &str,
        _bigint: bool,
        _throw_if_no_entry: bool,
    ) -> Result<serde_json::Value, String> {
        let _metadata = std_fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        let stats = Stats::from_path(path)?;

        Ok(serde_json::json!([
            stats.dev,
            stats.ino,
            stats.mode,
            stats.nlink,
            stats.uid,
            stats.gid,
            stats.rdev,
            stats.size,
            stats.blksize,
            stats.blocks,
            stats.atime,
            stats.mtime,
            stats.ctime,
            stats.birthtime,
        ]))
    }

    pub fn fstat(
        _fd: u32,
        _bigint: bool,
        _should_not_throw: bool,
    ) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!([
            0, 0, 0o100644, 1, 1000, 1000, 0, 0, 4096, 0, 0, 0, 0, 0
        ]))
    }

    pub fn readlink(path: &str, _encoding: &str) -> Result<String, String> {
        std_fs::read_link(path)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| e.to_string())
    }

    pub fn symlink(_target: &str, _path: &str, _flags: u32) -> Result<(), String> {
        Ok(())
    }

    pub fn link(existing_path: &str, new_path: &str) -> Result<(), String> {
        std_fs::hard_link(existing_path, new_path).map_err(|e| e.to_string())
    }

    pub fn truncate(path: &str, len: i64) -> Result<(), String> {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        let new_len = if len < 0 { 0u64 } else { len as u64 };
        file.set_len(new_len).map_err(|e| e.to_string())
    }

    pub fn ftruncate(_fd: u32, _len: i64) -> Result<(), String> {
        // TODO: maintain fd → file handle mapping for proper ftruncate
        Err("ftruncate not yet implemented".to_string())
    }

    pub fn chmod(path: &str, mode: u32) -> Result<(), String> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std_fs::set_permissions(path, std_fs::Permissions::from_mode(mode))
                .map_err(|e| e.to_string())
        }
        #[cfg(windows)]
        {
            let _ = path;
            let _ = mode;
            Ok(())
        }
    }

    pub fn fchmod(_fd: u32, _mode: u32) -> Result<(), String> {
        Ok(())
    }

    pub fn chown(_path: &str, _uid: u32, _gid: u32) -> Result<(), String> {
        Ok(())
    }

    pub fn fchown(_fd: u32, _uid: u32, _gid: u32) -> Result<(), String> {
        Ok(())
    }

    pub fn copy_file(src: &str, dest: &str, _flags: u32) -> Result<(), String> {
        std_fs::copy(src, dest)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub fn rm_sync(
        path: &str,
        _max_retries: u32,
        recursive: bool,
        _retry_delay: u32,
    ) -> Result<(), String> {
        let p = std::path::Path::new(path);
        if p.is_dir() {
            if recursive {
                std_fs::remove_dir_all(path).map_err(|e| e.to_string())
            } else {
                std_fs::remove_dir(path).map_err(|e| e.to_string())
            }
        } else {
            std_fs::remove_file(path).map_err(|e| e.to_string())
        }
    }

    pub fn statfs(_path: &str, _bigint: bool) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "bsize": 4096,
            "frsize": 4096,
            "blocks": 0,
            "bfree": 0,
            "bavail": 0,
            "files": 0,
            "ffree": 0,
            "favail": 0,
            "flag": 0,
            "maxfilename": 255
        }))
    }
}

pub mod os {
    use std::env;

    pub fn get_cp_us() -> Vec<serde_json::Value> {
        vec![
            serde_json::Value::String("Intel(R) Core(TM) i5-10500 CPU @ 3.10GHz".to_string()),
            serde_json::Value::Number(2400.into()),
            serde_json::Value::Number(0.into()),
            serde_json::Value::Number(0.into()),
            serde_json::Value::Number(0.into()),
            serde_json::Value::Number(0.into()),
            serde_json::Value::Number(0.into()),
            serde_json::Value::String("Intel(R) Core(TM) i5-10500 CPU @ 3.10GHz".to_string()),
            serde_json::Value::Number(2400.into()),
            serde_json::Value::Number(0.into()),
            serde_json::Value::Number(0.into()),
            serde_json::Value::Number(0.into()),
            serde_json::Value::Number(0.into()),
            serde_json::Value::Number(0.into()),
        ]
    }

    pub fn get_free_mem() -> u64 {
        8 * 1024 * 1024 * 1024
    }

    pub fn get_total_mem() -> u64 {
        16 * 1024 * 1024 * 1024
    }

    pub fn get_home_directory() -> Result<String, String> {
        env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .map_err(|_| "No home directory".to_string())
    }

    pub fn get_hostname() -> String {
        env::var("HOSTNAME").unwrap_or_else(|_| "kossjs".to_string())
    }

    pub fn get_interface_addresses() -> Result<serde_json::Value, String> {
        Ok(serde_json::Value::Object(serde_json::Map::new()))
    }

    pub fn get_load_avg(values: &mut [f64; 3]) {
        values[0] = 0.0;
        values[1] = 0.0;
        values[2] = 0.0;
    }

    pub fn get_uptime() -> u64 {
        0
    }

    pub fn get_os_info() -> (String, String, String, String) {
        (
            "Windows".to_string(),
            "10".to_string(),
            "10.0.19044".to_string(),
            "x64".to_string(),
        )
    }

    pub fn is_big_endian() -> bool {
        false
    }

    pub fn get_temp_dir() -> String {
        env::var("TEMP")
            .or_else(|_| env::var("TMP"))
            .unwrap_or_else(|_| {
                if cfg!(windows) {
                    "C:\\Temp".to_string()
                } else {
                    "/tmp".to_string()
                }
            })
    }

    pub fn get_user_info(_options: &str) -> Result<serde_json::Value, String> {
        let home = get_home_directory().unwrap_or_else(|_| "/home/user".to_string());
        #[cfg(windows)]
        let shell = serde_json::Value::Null;
        #[cfg(unix)]
        let shell = serde_json::Value::String("/bin/bash".to_string());

        Ok(serde_json::json!({
            "uid": 1000,
            "gid": 1000,
            "username": "user",
            "homedir": home,
            "shell": shell
        }))
    }

    pub fn get_available_parallelism() -> u32 {
        num_cpus::get() as u32
    }

    pub fn get_pid() -> u32 {
        std::process::id()
    }
}

pub mod constants {
    pub fn fs_flags() -> Vec<(&'static str, i32)> {
        vec![
            ("F_OK", 0),
            ("R_OK", 4),
            ("W_OK", 2),
            ("X_OK", 1),
            ("O_RDONLY", 0),
            ("O_WRONLY", 1),
            ("O_RDWR", 2),
            ("O_CREAT", 64),
            ("O_EXCL", 128),
            ("O_TRUNC", 512),
            ("O_APPEND", 1024),
            ("S_IFMT", 0xF000),
            ("S_IFREG", 0x8000),
            ("S_IFDIR", 0x4000),
            ("S_IFLNK", 0xA000),
        ]
    }

    pub fn os_constants() -> Vec<(&'static str, i32)> {
        vec![
            ("EADDRINUSE", -4094),
            ("EADDRNOTAVAIL", -4093),
            ("EACCES", -4092),
            ("EAGAIN", -4091),
            ("EBADF", -4089),
            ("ECONNREFUSED", -4087),
            ("ECONNRESET", -4086),
            ("EEXIST", -4082),
            ("EFAULT", -4081),
            ("EHOSTUNREACH", -4080),
            ("EINTR", -4079),
            ("EINVAL", -4078),
            ("EIO", -4077),
            ("ENOENT", -4066),
            ("ENOMEM", -4065),
            ("ENOTDIR", -4061),
            ("ENOTSUP", -4058),
            ("EPERM", -4055),
            ("EPIPE", -4054),
            ("ETIMEDOUT", -4046),
        ]
    }

    pub fn signals() -> Vec<(&'static str, i32)> {
        vec![
            ("SIGHUP", 1),
            ("SIGINT", 2),
            ("SIGQUIT", 3),
            ("SIGKILL", 9),
            ("SIGTERM", 15),
        ]
    }
}

pub mod buffer {
    pub fn byte_length_utf8(s: &str) -> usize {
        s.len()
    }

    pub fn compare(a: &[u8], b: &[u8]) -> i32 {
        for (av, bv) in a.iter().zip(b.iter()) {
            if av != bv {
                return if av < bv { -1 } else { 1 };
            }
        }
        (a.len() as i32).cmp(&(b.len() as i32)) as i32
    }

    pub fn copy(src: &[u8], dest: &mut [u8], pos: usize) {
        let len = std::cmp::min(src.len(), dest.len().saturating_sub(pos));
        dest[pos..pos + len].copy_from_slice(&src[..len]);
    }

    pub fn fill(dest: &mut [u8], value: u8, start: usize, end: usize) {
        for i in start..std::cmp::min(end, dest.len()) {
            dest[i] = value;
        }
    }

    pub fn is_ascii(data: &[u8]) -> bool {
        !data.iter().any(|&b| b > 127)
    }

    pub fn is_utf8(data: &[u8]) -> bool {
        std::str::from_utf8(data).is_ok()
    }

    pub fn ascii_slice(data: &[u8], start: usize, end: usize) -> String {
        String::from_utf8_lossy(&data[start..end]).to_string()
    }

    pub fn utf8_slice(data: &[u8], start: usize, end: usize) -> String {
        String::from_utf8_lossy(&data[start..end]).to_string()
    }

    pub fn latin1_slice(data: &[u8], start: usize, end: usize) -> String {
        data[start..end].iter().map(|&b| b as char).collect()
    }

    pub fn hex_slice(data: &[u8], start: usize, end: usize) -> String {
        data[start..end]
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }

    pub fn index_of_buffer(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
        haystack[start..]
            .windows(needle.len())
            .position(|w| w == needle)
            .map(|p| p + start)
    }

    pub fn index_of_number(haystack: &[u8], needle: u8, start: usize) -> Option<usize> {
        haystack[start..]
            .iter()
            .position(|&b| b == needle)
            .map(|p| p + start)
    }

    pub fn swap16(data: &mut [u8]) {
        for chunk in data.chunks_mut(2) {
            chunk.reverse();
        }
    }

    pub fn swap32(data: &mut [u8]) {
        for chunk in data.chunks_mut(4) {
            chunk.reverse();
        }
    }

    pub fn swap64(data: &mut [u8]) {
        for chunk in data.chunks_mut(8) {
            chunk.reverse();
        }
    }
}

pub mod timers {
    use once_cell::sync::Lazy;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static TIMER_START: Lazy<u64> = Lazy::new(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    });

    /// Legacy timer registry — timers are now handled by the JS-side timers.js
    /// which uses globalThis.setTimeout / Boa's native timer mechanism.
    /// This registry is retained for potential future native timer integration.
    static ACTIVE_TIMERS: Lazy<Mutex<std::collections::HashMap<u64, TimerInfo>>> =
        Lazy::new(|| Mutex::new(std::collections::HashMap::new()));

    #[derive(Clone)]
    pub struct TimerInfo {
        pub id: u64,
        pub callback: String,
        pub timeout: u64,
        pub is_interval: bool,
        pub active: bool,
    }

    pub fn get_libuv_now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    pub fn get_timer_start() -> u64 {
        *TIMER_START
    }

    pub fn schedule_timer(id: u64, timeout: u64, is_interval: bool) -> Result<String, String> {
        let mut timers = ACTIVE_TIMERS.lock().map_err(|e| e.to_string())?;

        let info = TimerInfo {
            id,
            callback: String::new(),
            timeout,
            is_interval,
            active: true,
        };

        timers.insert(id, info);
        Ok(format!("Timer {} scheduled", id))
    }

    pub fn toggle_timer_ref(id: u64, _unref: bool) -> Result<(), String> {
        let mut timers = ACTIVE_TIMERS.lock().map_err(|e| e.to_string())?;

        if let Some(timer) = timers.get_mut(&id) {
            timer.active = false;
        }

        Ok(())
    }

    pub fn clear_timer(id: u64) -> Result<(), String> {
        let mut timers = ACTIVE_TIMERS.lock().map_err(|e| e.to_string())?;
        timers.remove(&id);
        Ok(())
    }

    pub fn get_active_timer_count() -> usize {
        ACTIVE_TIMERS.lock().map(|t| t.len()).unwrap_or(0)
    }
}

pub mod crypto {
    use rand::Rng;
    use rand::RngExt;

    pub fn get_random_values(size: usize) -> Vec<u8> {
        let mut buf = vec![0u8; size];
        rand::rng().fill_bytes(&mut buf);
        buf
    }

    pub fn random_int(min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }
        rand::rng().random_range(min..=max)
    }

    pub fn random_uuid() -> String {
        let mut buf = [0u8; 16];
        rand::rng().fill_bytes(&mut buf);
        // Set UUID v4 version bits
        buf[6] = (buf[6] & 0x0f) | 0x40;
        buf[8] = (buf[8] & 0x3f) | 0x80;
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            buf[0], buf[1], buf[2], buf[3],
            buf[4], buf[5],
            buf[6], buf[7],
            buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
        )
    }

    pub fn hash_sha1(data: &str) -> String {
        use sha1::{Digest, Sha1};
        let mut hasher = Sha1::new();
        hasher.update(data.as_bytes());
        hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>()
    }

    pub fn hash_sha256(data: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>()
    }

    pub fn hash_md5(data: &str) -> String {
        use md5::{Digest, Md5};
        let mut hasher = Md5::new();
        hasher.update(data.as_bytes());
        hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>()
    }

    pub fn create_hash(algorithm: &str, data: &str) -> Result<String, String> {
        match algorithm.to_lowercase().as_str() {
            "sha1" | "sha-1" => Ok(hash_sha1(data)),
            "sha256" | "sha-256" => Ok(hash_sha256(data)),
            "md5" => Ok(hash_md5(data)),
            _ => Err(format!("Unknown algorithm: {}", algorithm)),
        }
    }

    pub fn create_hmac(algorithm: &str, key: &str, data: &str) -> Result<String, String> {
        use hmac::{Hmac, Mac, KeyInit};
        use sha1::Sha1;
        use sha2::Sha256;
        use md5::Md5;

        macro_rules! hmac_for {
            ($algo:ty) => {{
                let mut mac = <Hmac<$algo>>::new_from_slice(key.as_bytes())
                    .map_err(|e| format!("HMAC key error: {e}"))?;
                mac.update(data.as_bytes());
                let result = mac.finalize();
                result.into_bytes().iter().map(|b| format!("{:02x}", b)).collect::<String>()
            }};
        }

        match algorithm.to_lowercase().as_str() {
            "sha256" | "sha-256" => Ok(hmac_for!(Sha256)),
            "sha1" | "sha-1" => Ok(hmac_for!(Sha1)),
            "md5" => Ok(hmac_for!(Md5)),
            _ => Err(format!("Unknown algorithm: {}", algorithm)),
        }
    }

    const PBKDF2_MAX_KEY_LEN: u32 = 512;
    const PBKDF2_MIN_ITERATIONS: u32 = 100_000;

    pub fn pbkdf2(
        password: &str,
        salt: &str,
        iterations: u32,
        key_len: u32,
    ) -> Result<String, String> {
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;

        if iterations < PBKDF2_MIN_ITERATIONS {
            return Err(format!(
                "Iterations must be at least {}",
                PBKDF2_MIN_ITERATIONS
            ));
        }
        if key_len == 0 || key_len > PBKDF2_MAX_KEY_LEN {
            return Err(format!(
                "key_len must be between 1 and {}",
                PBKDF2_MAX_KEY_LEN
            ));
        }

        let mut key = vec![0u8; key_len as usize];
        pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            salt.as_bytes(),
            iterations,
            &mut key,
        );
        Ok(key.iter().map(|b| format!("{:02x}", b)).collect::<String>())
    }

    pub fn generate_prime(bits: u32) -> Result<u64, String> {
        if bits < 2 {
            return Err("bits must be at least 2".to_string());
        }
        let max_bits = bits.min(32);
        let lo = 2u64.pow(max_bits.saturating_sub(1));
        let hi = 2u64.pow(max_bits);
        if lo >= hi {
            return Err("invalid bit range".to_string());
        }
        let mut rng = rand::rng();
        loop {
            let candidate = rng.random_range(lo..hi) | 1;
            if is_miller_rabin_prime(candidate, 40) {
                return Ok(candidate);
            }
        }
    }

    fn mod_pow(base: u64, mut exp: u64, modulus: u64) -> u64 {
        if modulus == 1 {
            return 0;
        }
        let mut result: u128 = 1;
        let mut b: u128 = (base % modulus) as u128;
        let m = modulus as u128;
        while exp > 0 {
            if exp & 1 == 1 {
                result = (result * b) % m;
            }
            exp >>= 1;
            b = (b * b) % m;
        }
        result as u64
    }

    fn is_miller_rabin_prime(n: u64, k: u32) -> bool {
        if n < 2 {
            return false;
        }
        if n == 2 || n == 3 {
            return true;
        }
        if n % 2 == 0 {
            return false;
        }
        let mut d = n - 1;
        let mut s: u32 = 0;
        while d % 2 == 0 {
            d /= 2;
            s += 1;
        }
        let mut rng = rand::rng();
        for _ in 0..k {
            let a: u64 = rng.random_range(2..n - 1);
            let mut x = mod_pow(a, d, n);
            if x == 1 || x == n - 1 {
                continue;
            }
            let mut composite = true;
            for _ in 0..s - 1 {
                x = mod_pow(x, 2, n);
                if x == n - 1 {
                    composite = false;
                    break;
                }
            }
            if composite {
                return false;
            }
        }
        true
    }

    pub fn get_crypto_constants() -> Vec<(&'static str, i32)> {
        vec![
            ("OPENSSL_VERSION_NUMBER", 0x30000000),
            ("SSL_OP_ALL", 0x80000),
            ("SSL_OP_NO_SSLv2", 0x100000),
            ("SSL_OP_NO_SSLv3", 0x200000),
            ("SSL_OP_NO_TLSv1", 0x400000),
            ("SSL_OP_NO_TLSv1_2", 0x800000),
        ]
    }
}

pub mod net {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket};

    pub fn is_ip(address: &str) -> bool {
        address.parse::<IpAddr>().is_ok()
    }

    pub fn is_ipv4(address: &str) -> bool {
        address.parse::<Ipv4Addr>().is_ok()
    }

    pub fn is_ipv6(address: &str) -> bool {
        address.parse::<Ipv6Addr>().is_ok()
    }

    pub fn parse_ip(address: &str) -> Option<String> {
        address.parse::<IpAddr>().ok().map(|ip| ip.to_string())
    }

    pub fn get_protocol_family(socket_type: &str) -> String {
        match socket_type {
            "TCP" | "tcp" => "IPv4".to_string(),
            "UDP" | "udp" => "IPv4".to_string(),
            _ => "IPv4".to_string(),
        }
    }

    pub fn get_socket_type(protocol: u32) -> String {
        match protocol {
            6 => "TCP".to_string(),
            17 => "UDP".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    }

    pub fn new_tcp_socket() -> Result<String, String> {
        Ok("TCP socket created".to_string())
    }

    pub fn new_udp_socket() -> Result<String, String> {
        Ok("UDP socket created".to_string())
    }

    fn is_ssrf_blocked(host: &str) -> bool {
        if let Ok(ip) = host.parse::<IpAddr>() {
            return is_blocked_ip(&ip);
        }
        if let Ok(addrs) = format!("{}:0", host).to_socket_addrs() {
            for addr in addrs {
                if is_blocked_ip(&addr.ip()) {
                    return true;
                }
            }
        }
        false
    }

    fn is_blocked_ip(ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(v4) => {
                if v4.octets()[0] == 127 { return true; }
                if v4.octets()[0] == 10 { return true; }
                if v4.octets()[0] == 172 && v4.octets()[1] >= 16 && v4.octets()[1] <= 31 { return true; }
                if v4.octets()[0] == 192 && v4.octets()[1] == 168 { return true; }
                if v4.octets()[0] == 169 && v4.octets()[1] == 254 { return true; }
                if v4.octets()[0] == 0 { return true; }
                if v4.octets()[0] == 100 && v4.octets()[1] >= 64 && v4.octets()[1] <= 127 { return true; }
                if v4.octets()[0] == 198 && (v4.octets()[1] == 18 || v4.octets()[1] == 19) { return true; }
                if v4.octets()[0] >= 224 && v4.octets()[0] <= 239 { return true; }
                if v4.octets()[0] >= 240 { return true; }
                false
            }
            IpAddr::V6(v6) => {
                if let Some(v4) = v6.to_ipv4_mapped() {
                    return is_blocked_ip(&IpAddr::V4(v4));
                }
                if v6.is_loopback() { return true; }
                if v6.segments()[0] & 0xffc0 == 0xfe80 { return true; }
                if v6.segments()[0] & 0xfe00 == 0xfc00 { return true; }
                false
            }
        }
    }

    /// Test-only: binds momentarily to verify port availability, then closes.
    /// For persistent sockets, use the async event loop via tokio integration.
    pub fn tcp_bind(address: &str, port: u16) -> Result<String, String> {
        if is_ssrf_blocked(address) {
            return Err(format!("SSRF blocked: {address}"));
        }
        let addr = format!("{}:{}", address, port);
        match TcpListener::bind(&addr) {
            Ok(_listener) => {
                Ok(format!("Bound to {}", addr))
            }
            Err(e) => Err(format!("Bind failed: {}", e)),
        }
    }

    /// Test-only: connects momentarily to verify reachability, then closes.
    /// For persistent sockets, use the async event loop via tokio integration.
    pub fn tcp_connect(address: &str, port: u16) -> Result<String, String> {
        if is_ssrf_blocked(address) {
            return Err(format!("SSRF blocked: {address}"));
        }
        let addr = format!("{}:{}", address, port);
        match TcpStream::connect(&addr) {
            Ok(_stream) => {
                Ok(format!("Connected to {}", addr))
            }
            Err(e) => Err(format!("Connect failed: {}", e)),
        }
    }

    /// Test-only: binds momentarily to verify port availability, then closes.
    /// For persistent sockets, use the async event loop via tokio integration.
    pub fn udp_bind(address: &str, port: u16) -> Result<String, String> {
        if is_ssrf_blocked(address) {
            return Err(format!("SSRF blocked: {address}"));
        }
        let addr = format!("{}:{}", address, port);
        match UdpSocket::bind(&addr) {
            Ok(_socket) => {
                Ok(format!("Bound to {}", addr))
            }
            Err(e) => Err(format!("Bind failed: {}", e)),
        }
    }

    pub fn get_local_address() -> Result<String, String> {
        Ok("127.0.0.1".to_string())
    }

    pub fn get_local_port() -> u16 {
        0
    }

    pub fn dns_lookup(hostname: &str) -> Result<Vec<String>, String> {
        use std::net::ToSocketAddrs;
        let addr = format!("{}:80", hostname);
        match addr.to_socket_addrs() {
            Ok(addrs) => Ok(addrs.map(|a| a.ip().to_string()).collect()),
            Err(_) => Err(format!("DNS lookup failed for {}", hostname)),
        }
    }

    pub fn get_socket_error(_fd: u32) -> i32 {
        0
    }

    pub fn set_no_delay(_fd: u32, _no_delay: bool) -> Result<(), String> {
        Ok(())
    }

    pub fn set_keep_alive(_fd: u32, _enable: bool) -> Result<(), String> {
        Ok(())
    }

    pub fn set_reuse_addr(_fd: u32, _reuse: bool) -> Result<(), String> {
        Ok(())
    }
}

pub mod http_parser {
    use std::collections::HashMap;

    #[derive(Clone)]
    pub struct HttpRequest {
        pub method: String,
        pub url: String,
        pub headers: HashMap<String, String>,
        pub body: Vec<u8>,
    }

    #[derive(Clone)]
    pub struct HttpResponse {
        pub status_code: u16,
        pub status_message: String,
        pub headers: HashMap<String, String>,
        pub body: Vec<u8>,
    }

    pub fn parse_request(data: &str) -> Result<HttpRequest, String> {
        let mut lines = data.split("\r\n");
        let request_line = lines.next().ok_or("Empty request")?;

        let parts: Vec<&str> = request_line.split(' ').collect();
        if parts.len() < 2 {
            return Err("Invalid request line".to_string());
        }

        let method = parts[0].to_string();
        let url = parts[1].to_string();

        let mut headers = HashMap::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        }

        Ok(HttpRequest {
            method,
            url,
            headers,
            body: Vec::new(),
        })
    }

    pub fn parse_response(data: &str) -> Result<HttpResponse, String> {
        let mut lines = data.split("\r\n");
        let status_line = lines.next().ok_or("Empty response")?;

        let parts: Vec<&str> = status_line.split(' ').collect();
        if parts.len() < 2 {
            return Err("Invalid status line".to_string());
        }

        let status_code: u16 = parts[1].parse().map_err(|_| "Invalid status code")?;
        let status_message = if parts.len() > 2 {
            parts[2..].join(" ")
        } else {
            String::new()
        };

        let mut headers = HashMap::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        }

        Ok(HttpResponse {
            status_code,
            status_message,
            headers,
            body: Vec::new(),
        })
    }

    pub fn method_string_to_int(method: &str) -> u8 {
        match method.to_uppercase().as_str() {
            "GET" => 1,
            "POST" => 2,
            "PUT" => 3,
            "DELETE" => 4,
            "PATCH" => 5,
            "HEAD" => 6,
            "OPTIONS" => 7,
            "CONNECT" => 8,
            "TRACE" => 9,
            _ => 0,
        }
    }

    pub fn method_int_to_string(method: u8) -> String {
        match method {
            1 => "GET",
            2 => "POST",
            3 => "PUT",
            4 => "DELETE",
            5 => "PATCH",
            6 => "HEAD",
            7 => "OPTIONS",
            8 => "CONNECT",
            9 => "TRACE",
            _ => "UNKNOWN",
        }
        .to_string()
    }

    pub fn status_text(status_code: u16) -> String {
        match status_code {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            301 => "Moved Permanently",
            302 => "Found",
            304 => "Not Modified",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            500 => "Internal Server Error",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            _ => "Unknown",
        }
        .to_string()
    }
}

pub mod url {
    use std::collections::HashMap;

    #[derive(Clone)]
    pub struct UrlParts {
        pub protocol: String,
        pub hostname: String,
        pub port: String,
        pub pathname: String,
        pub query: String,
        pub hash: String,
        pub username: String,
        pub password: String,
    }

    pub fn parse_url(url_str: &str) -> Result<UrlParts, String> {
        let parsed = ::url::Url::parse(url_str).map_err(|e| format!("invalid URL: {e}"))?;

        Ok(UrlParts {
            protocol: parsed.scheme().to_string(),
            hostname: parsed.host_str().unwrap_or("").to_string(),
            port: parsed.port().map(|p| p.to_string()).unwrap_or_default(),
            pathname: parsed.path().to_string(),
            query: parsed.query().unwrap_or("").to_string(),
            hash: parsed.fragment().unwrap_or("").to_string(),
            username: parsed.username().to_string(),
            password: parsed.password().unwrap_or("").to_string(),
        })
    }

    pub fn format_url(parts: &UrlParts) -> String {
        let mut url = String::new();

        if !parts.protocol.is_empty() {
            url.push_str(&parts.protocol);
            url.push_str("://");
        }

        if !parts.username.is_empty() {
            url.push_str(&parts.username);
            if !parts.password.is_empty() {
                url.push(':');
                url.push_str(&parts.password);
            }
            url.push('@');
        }

        url.push_str(&parts.hostname);

        if !parts.port.is_empty() {
            url.push(':');
            url.push_str(&parts.port);
        }

        url.push_str(&parts.pathname);

        if !parts.query.is_empty() {
            url.push('?');
            url.push_str(&parts.query);
        }

        if !parts.hash.is_empty() {
            url.push('#');
            url.push_str(&parts.hash);
        }

        url
    }

    pub fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();

        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                params.insert(urlencoding_decode(key), urlencoding_decode(value));
            }
        }

        params
    }

    fn urlencoding_decode(s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                        continue;
                    }
                }
                result.push('%');
                result.push_str(&hex);
            } else if c == '+' {
                result.push(' ');
            } else {
                result.push(c);
            }
        }

        result
    }

    pub fn encode_uri_component(s: &str) -> String {
        let mut result = String::new();

        for c in s.chars() {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                result.push(c);
            } else {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }

        result
    }
}

pub mod util {
    pub fn get_system_error_name(code: i32) -> String {
        match code {
            0 => "OK",
            1 => "EPERM",
            2 => "ENOENT",
            3 => "ESRCH",
            4 => "EINTR",
            5 => "EIO",
            6 => "ENXIO",
            7 => "E2BIG",
            8 => "ENOEXEC",
            9 => "EBADF",
            10 => "ECHILD",
            11 => "EAGAIN",
            12 => "ENOMEM",
            13 => "EACCES",
            14 => "EFAULT",
            15 => "ENOTBLK",
            16 => "EBUSY",
            17 => "EEXIST",
            18 => "EXDEV",
            19 => "ENODEV",
            20 => "ENOTDIR",
            21 => "EISDIR",
            22 => "EINVAL",
            23 => "ENFILE",
            24 => "EMFILE",
            25 => "ENOTTY",
            26 => "ETXTBSY",
            27 => "EFBIG",
            28 => "ENOSPC",
            29 => "ESPIPE",
            30 => "EROFS",
            31 => "EMLINK",
            32 => "EPIPE",
            33 => "EDOM",
            34 => "ERANGE",
            35 => "EDEADLK",
            36 => "ENAMETOOLONG",
            37 => "ENOLCK",
            38 => "ENOSYS",
            39 => "ENOTEMPTY",
            40 => "ELOOP",
            41 => "EWOULDBLOCK",
            42 => "ENOMSG",
            43 => "EIDRM",
            44 => "ECHRNG",
            45 => "EL2NSYNC",
            46 => "EL3HLT",
            47 => "EL3RST",
            48 => "ELNRNG",
            49 => "EUNATCH",
            50 => "ENOCSI",
            51 => "EL2HLT",
            52 => "EBADE",
            53 => "EBADR",
            54 => "EXFULL",
            55 => "ENOANO",
            56 => "EBADRQC",
            57 => "EBADSLT",
            58 => "EBFONT",
            59 => "ENOSTR",
            60 => "ENODATA",
            61 => "ETIME",
            62 => "ENOSR",
            63 => "ENONET",
            64 => "ENOPKG",
            65 => "EREMOTE",
            66 => "ENOLINK",
            67 => "EADV",
            68 => "ESRMNT",
            69 => "ECOMM",
            70 => "EPROTO",
            71 => "EMULTIHOP",
            72 => "EDOTDOT",
            73 => "EBADMSG",
            74 => "EOVERFLOW",
            75 => "ENOTUNIQ",
            76 => "EBADFD",
            77 => "EREMCHG",
            78 => "ELIBACC",
            79 => "ELIBBAD",
            80 => "ELIBSCN",
            81 => "ELIBMAX",
            82 => "ELIBEXEC",
            83 => "EILSEQ",
            84 => "ERESTART",
            85 => "ESTRPIPE",
            86 => "EUSERS",
            87 => "ENOTSOCK",
            88 => "EDESTADDRREQ",
            89 => "EMSGSIZE",
            90 => "EPROTOTYPE",
            91 => "ENOPROTOOPT",
            92 => "EPROTONOSUPPORT",
            93 => "ESOCKTNOSUPPORT",
            94 => "EOPNOTSUPP",
            95 => "EPFNOSUPPORT",
            96 => "EAFNOSUPPORT",
            97 => "EADDRINUSE",
            98 => "EADDRNOTAVAIL",
            99 => "ENETDOWN",
            100 => "ENETUNREACH",
            101 => "ENETRESET",
            102 => "ECONNABORTED",
            103 => "ECONNRESET",
            104 => "ENOBUFS",
            105 => "EISCONN",
            106 => "ENOTCONN",
            107 => "ESHUTDOWN",
            108 => "ETOOMANYREFS",
            109 => "ETIMEDOUT",
            110 => "ECONNREFUSED",
            111 => "EHOSTDOWN",
            112 => "EHOSTUNREACH",
            113 => "EALREADY",
            114 => "EINPROGRESS",
            115 => "ESTALE",
            116 => "EUCLEAN",
            117 => "ENOTNAM",
            118 => "ENAVAIL",
            119 => "EISNAM",
            120 => "EREMOTEIO",
            121 => "EDQUOT",
            122 => "ENOMEDIUM",
            123 => "EMEDIUMTYPE",
            124 => "ECANCELED",
            125 => "ENOKEY",
            126 => "EKEYEXPIRED",
            127 => "EKEYREVOKED",
            128 => "EKEYREJECTED",
            129 => "EOWNERDEAD",
            130 => "ENOTRECOVERABLE",
            131 => "ERFKILL",
            132 => "EHWPOISON",
            _ => "UNKNOWN",
        }
        .to_string()
    }

    pub fn get_system_error_code(code: i32) -> i32 {
        // On Windows, convert to negative errno style like Node.js
        #[cfg(windows)]
        {
            -code
        }
        #[cfg(not(windows))]
        {
            code
        }
    }

    pub fn inspect_object(obj: &str, _options: &str) -> String {
        format!("{{ {} }}", obj)
    }

    pub fn format_with_depth(_obj: &str, _depth: u32) -> String {
        String::new()
    }
}

pub mod trace_events {
    pub fn create_trace_event(_category_group: &str, _name: &str) -> Result<String, String> {
        Ok("trace_event_created".to_string())
    }

    pub fn get_trace_categories() -> Vec<String> {
        vec![
            "node".to_string(),
            "node.async".to_string(),
            "node.fs".to_string(),
            "node.http".to_string(),
            "node.inspector".to_string(),
            "node.perf".to_string(),
            "node.promise".to_string(),
            "node.vm".to_string(),
        ]
    }

    pub fn enable_trace(_category: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn disable_trace(_category: &str) -> Result<(), String> {
        Ok(())
    }
}

pub mod fetch {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::net::{IpAddr, ToSocketAddrs};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct FetchRequest {
        pub method: String,
        pub headers: HashMap<String, String>,
        pub body: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct FetchResponse {
        pub body: String,
        pub status: u16,
        pub status_text: String,
        pub headers: HashMap<String, String>,
    }

    /// Validate a URL against SSRF attacks: block non-http/https schemes,
    /// private/internal IP ranges, and unsafe hosts.
    fn validate_url(url: &str) -> Result<(), String> {
        let parsed = url::Url::parse(url).map_err(|e| format!("invalid URL: {e}"))?;

        // Block non-http/https schemes
        let scheme = parsed.scheme().to_lowercase();
        if scheme != "http" && scheme != "https" {
            return Err(format!("blocked scheme: {}", scheme));
        }

        // Resolve hostname to IP and check against blocked ranges
        let host = parsed.host_str().ok_or("missing host")?;

        // Direct IP check
        if let Ok(ip) = host.parse::<IpAddr>() {
            if is_blocked_ip(&ip) {
                return Err(format!("blocked IP address: {ip}"));
            }
            return Ok(());
        }

        // DNS resolution check for hostnames
        let lookup = format!("{host}:0").to_socket_addrs().map_err(|e| format!("DNS error: {e}"))?;
        for addr in lookup {
            if is_blocked_ip(&addr.ip()) {
                return Err(format!("host {host} resolves to blocked IP: {}", addr.ip()));
            }
        }

        Ok(())
    }

    fn is_blocked_ip(ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(v4) => {
                // Loopback: 127.0.0.0/8
                if v4.octets()[0] == 127 { return true; }
                // Private A: 10.0.0.0/8
                if v4.octets()[0] == 10 { return true; }
                // Private B: 172.16.0.0/12
                if v4.octets()[0] == 172 && v4.octets()[1] >= 16 && v4.octets()[1] <= 31 { return true; }
                // Private C: 192.168.0.0/16
                if v4.octets()[0] == 192 && v4.octets()[1] == 168 { return true; }
                // Link-local / cloud metadata: 169.254.0.0/16
                if v4.octets()[0] == 169 && v4.octets()[1] == 254 { return true; }
                // Current network (zero config): 0.0.0.0/8
                if v4.octets()[0] == 0 { return true; }
                // Carrier-grade NAT: 100.64.0.0/10 (RFC 6598)
                if v4.octets()[0] == 100 && v4.octets()[1] >= 64 && v4.octets()[1] <= 127 { return true; }
                // Benchmarking: 198.18.0.0/15 (RFC 2544)
                if v4.octets()[0] == 198 && (v4.octets()[1] == 18 || v4.octets()[1] == 19) { return true; }
                // Multicast: 224.0.0.0/4
                if v4.octets()[0] >= 224 && v4.octets()[0] <= 239 { return true; }
                // IANA reserved: 240.0.0.0/4
                if v4.octets()[0] >= 240 { return true; }
                false
            }
            IpAddr::V6(v6) => {
                // IPv4-mapped IPv6 bypass check: convert and re-check as IPv4
                if let Some(v4) = v6.to_ipv4_mapped() {
                    return is_blocked_ip(&IpAddr::V4(v4));
                }
                // Loopback: ::1
                if v6.is_loopback() { return true; }
                // Link-local: fe80::/10
                if v6.segments()[0] & 0xffc0 == 0xfe80 { return true; }
                // Unique local: fc00::/7
                if v6.segments()[0] & 0xfe00 == 0xfc00 { return true; }
                false
            }
        }
    }

    /// Build a reqwest client (async) with both webpki roots and platform native certs.
    fn build_client() -> Result<reqwest::Client, String> {
        // Ensure a rustls crypto provider is installed before first use
        // (rustls 0.23+ no longer auto-installs a crypto provider).
        use std::sync::OnceLock;
        static RUSTLS_INIT: OnceLock<()> = OnceLock::new();
        RUSTLS_INIT.get_or_init(|| {
            rustls::crypto::aws_lc_rs::default_provider()
                .install_default()
                .expect("failed to install rustls aws-lc-rs crypto provider");
        });

        let mut root_store = rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        };
        let native_result = rustls_native_certs::load_native_certs();
        for cert in native_result.certs {
            root_store.add(cert).ok();
        }
        if !native_result.errors.is_empty() {
            eprintln!(
                "Warning: {} error(s) loading native certificates",
                native_result.errors.len()
            );
        }
        let tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        reqwest::Client::builder()
            .use_preconfigured_tls(tls_config)
            .timeout(std::time::Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none()) // Manual redirect handling for SSRF safety
            .build()
            .map_err(|e| format!("client build error: {}", e))
    }

    fn build_request(url: &str, request_json: &str) -> Result<(String, FetchRequest), String> {
        let request: FetchRequest = serde_json::from_str(request_json)
            .map_err(|e| format!("parse request error: {}", e))?;
        Ok((url.to_string(), request))
    }

    async fn do_fetch(url: String, request: FetchRequest) -> Result<FetchResponse, String> {
        validate_url(&url)?;

        let client = build_client()?;

        let method = reqwest::Method::from_bytes(request.method.to_uppercase().as_bytes())
            .unwrap_or(reqwest::Method::GET);

        let mut req_builder = client.request(method, &url);

        for (key, value) in &request.headers {
            req_builder = req_builder.header(key, value);
        }

        if let Some(body) = &request.body {
            req_builder = req_builder.body(body.clone());
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| format!("request error: {}", e))?;

        let status = response.status();
        let status_code = status.as_u16();
        let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();

        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(key.as_str().to_string(), v.to_string());
            }
        }

        let body = response
            .text()
            .await
            .map_err(|e| format!("read body error: {}", e))?;

        Ok(FetchResponse {
            body,
            status: status_code,
            status_text,
            headers,
        })
    }

    /// Async fetch - call from tokio tasks
    pub async fn fetch_async(url: &str, request_json: &str) -> Result<String, String> {
        let (url, request) = build_request(url, request_json)?;
        let response = do_fetch(url, request).await?;
        serde_json::to_string(&response).map_err(|e| format!("serialize error: {e}"))
    }

    /// Synchronous fetch - blocks the current thread (fallback)
    pub fn fetch_with_url(url: &str, request_json: &str) -> Result<FetchResponse, String> {
        let (url, request) = build_request(url, request_json)?;
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("runtime error: {e}"))?;
        rt.block_on(do_fetch(url, request))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── buffer tests ────────────────────────────────────────────────────

    #[test]
    fn test_byte_length_utf8() {
        assert_eq!(buffer::byte_length_utf8("hello"), 5);
        assert_eq!(buffer::byte_length_utf8(""), 0);
        assert_eq!(buffer::byte_length_utf8("你好"), 6); // 3 bytes per char in UTF-8
    }

    #[test]
    fn test_buffer_compare() {
        assert_eq!(buffer::compare(b"abc", b"abc"), 0);
        assert_eq!(buffer::compare(b"abc", b"abd"), -1);
        assert_eq!(buffer::compare(b"abd", b"abc"), 1);
        assert_eq!(buffer::compare(b"ab", b"abc"), -1);
        assert_eq!(buffer::compare(b"abc", b"ab"), 1);
        assert_eq!(buffer::compare(b"", b""), 0);
    }

    #[test]
    fn test_buffer_copy() {
        let src = [1u8, 2, 3, 4, 5];
        let mut dest = [0u8; 10];
        buffer::copy(&src, &mut dest, 0);
        assert_eq!(&dest[..5], &[1, 2, 3, 4, 5]);
        assert_eq!(dest[5], 0);
    }

    #[test]
    fn test_buffer_copy_with_offset() {
        let src = [9u8, 8, 7];
        let mut dest = [0u8; 8];
        buffer::copy(&src, &mut dest, 4);
        assert_eq!(&dest[4..7], &[9, 8, 7]);
        assert_eq!(dest[0], 0);
    }

    #[test]
    fn test_buffer_copy_truncated() {
        let src = [1u8, 2, 3, 4, 5];
        let mut dest = [0u8; 3];
        buffer::copy(&src, &mut dest, 0);
        assert_eq!(&dest, &[1, 2, 3]);
    }

    #[test]
    fn test_buffer_fill() {
        let mut buf = [0u8; 10];
        buffer::fill(&mut buf, 0xFF, 0, 10);
        assert!(buf.iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn test_buffer_fill_partial() {
        let mut buf = [0u8; 10];
        buffer::fill(&mut buf, 0xAA, 2, 6);
        assert_eq!(&buf[0..2], &[0, 0]);
        assert_eq!(&buf[2..6], &[0xAA; 4]);
        assert_eq!(&buf[6..10], &[0; 4]);
    }

    #[test]
    fn test_buffer_fill_out_of_bounds() {
        let mut buf = [0u8; 5];
        buffer::fill(&mut buf, 1, 3, 100);
        assert_eq!(&buf[0..3], &[0; 3]);
        assert_eq!(&buf[3..5], &[1, 1]);
    }

    #[test]
    fn test_is_ascii() {
        assert!(buffer::is_ascii(b"hello"));
        assert!(buffer::is_ascii(b""));
        assert!(!buffer::is_ascii(&[0x80]));
        assert!(!buffer::is_ascii("café".as_bytes()));
    }

    #[test]
    fn test_is_utf8() {
        assert!(buffer::is_utf8(b"hello"));
        assert!(buffer::is_utf8("你好".as_bytes()));
        assert!(!buffer::is_utf8(&[0xFF, 0xFE]));
    }

    #[test]
    fn test_ascii_slice() {
        assert_eq!(buffer::ascii_slice(b"hello", 1, 4), "ell");
    }

    #[test]
    fn test_hex_slice() {
        assert_eq!(buffer::hex_slice(&[0xAB, 0xCD], 0, 2), "abcd");
        assert_eq!(buffer::hex_slice(&[0x0F, 0xF0], 0, 2), "0ff0");
    }

    #[test]
    fn test_index_of_buffer() {
        assert_eq!(buffer::index_of_buffer(b"hello world", b"world", 0), Some(6));
        assert_eq!(buffer::index_of_buffer(b"hello world", b"hello", 0), Some(0));
        assert_eq!(buffer::index_of_buffer(b"hello world", b"xyz", 0), None);
        assert_eq!(buffer::index_of_buffer(b"aaaa", b"aa", 2), Some(2));
    }

    #[test]
    fn test_index_of_number() {
        assert_eq!(buffer::index_of_number(b"hello", b'e', 0), Some(1));
        assert_eq!(buffer::index_of_number(b"hello", b'l', 0), Some(2));
        assert_eq!(buffer::index_of_number(b"hello", b'z', 0), None);
        assert_eq!(buffer::index_of_number(b"hello", b'l', 3), Some(3));
    }

    #[test]
    fn test_swap16() {
        let mut data = [0x01u8, 0x02, 0x03, 0x04];
        buffer::swap16(&mut data);
        assert_eq!(&data, &[0x02, 0x01, 0x04, 0x03]);
    }

    #[test]
    fn test_swap32() {
        let mut data = [0x01u8, 0x02, 0x03, 0x04];
        buffer::swap32(&mut data);
        assert_eq!(&data, &[0x04, 0x03, 0x02, 0x01]);
    }

    #[test]
    fn test_swap64() {
        let mut data = [0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        buffer::swap64(&mut data);
        assert_eq!(&data, &[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
    }

    // ── net tests ───────────────────────────────────────────────────────

    #[test]
    fn test_is_ip() {
        assert!(net::is_ip("127.0.0.1"));
        assert!(net::is_ip("::1"));
        assert!(!net::is_ip("not_an_ip"));
        assert!(!net::is_ip(""));
    }

    #[test]
    fn test_is_ipv4() {
        assert!(net::is_ipv4("192.168.1.1"));
        assert!(net::is_ipv4("0.0.0.0"));
        assert!(!net::is_ipv4("::1"));
        assert!(!net::is_ipv4("999.999.999.999"));
    }

    #[test]
    fn test_is_ipv6() {
        assert!(net::is_ipv6("::1"));
        assert!(net::is_ipv6("2001:db8::1"));
        assert!(!net::is_ipv6("192.168.1.1"));
    }

    #[test]
    fn test_parse_ip() {
        assert_eq!(net::parse_ip("127.0.0.1"), Some("127.0.0.1".to_string()));
        assert_eq!(net::parse_ip("::1"), Some("::1".to_string()));
        assert_eq!(net::parse_ip("invalid"), None);
    }

    #[test]
    fn test_get_protocol_family() {
        assert_eq!(net::get_protocol_family("TCP"), "IPv4");
        assert_eq!(net::get_protocol_family("UDP"), "IPv4");
    }

    // ── fs tests ────────────────────────────────────────────────────────

    fn temp_dir() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let d = std::env::temp_dir().join(format!("kossjs_test_{}_{}", std::process::id(), id));
        let _ = std::fs::create_dir_all(&d);
        d
    }

    fn drop_dir(d: &std::path::Path) {
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn test_fs_exists_sync() {
        let dir = temp_dir();
        let file = dir.join("exists_test.txt");
        std::fs::write(&file, b"hello").unwrap();

        assert!(fs::exists_sync(file.to_str().unwrap()));
        assert!(!fs::exists_sync(dir.join("no_such_file").to_str().unwrap()));

        drop_dir(&dir);
    }

    #[test]
    fn test_fs_read_file_utf8() {
        let dir = temp_dir();
        let file = dir.join("read_test.txt");
        std::fs::write(&file, "hello world").unwrap();

        let content = fs::read_file_utf8(file.to_str().unwrap(), 0).unwrap();
        assert_eq!(content, "hello world");

        drop_dir(&dir);
    }

    #[test]
    fn test_fs_access() {
        let dir = temp_dir();
        let file = dir.join("access_test.txt");
        std::fs::File::create(&file).unwrap();

        assert!(fs::access(file.to_str().unwrap(), 0).is_ok());
        assert!(fs::access(dir.join("no_such").to_str().unwrap(), 0).is_err());

        drop_dir(&dir);
    }

    #[test]
    fn test_fs_mkdir_rmdir() {
        let base = std::env::temp_dir().join(format!("kossjs_fs_dir_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        let sub = base.join("subdir");

        // mkdir: path, mode, recursive=false (parent exists)
        assert!(fs::mkdir(sub.to_str().unwrap(), 0o755, false).is_ok());
        assert!(sub.exists());

        // rmdir
        assert!(fs::rmdir(sub.to_str().unwrap()).is_ok());
        assert!(!sub.exists());

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn test_fs_mkdir_recursive() {
        let base = std::env::temp_dir().join(format!("kossjs_fs_rec_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let deep = base.join("a").join("b").join("c");

        assert!(fs::mkdir(deep.to_str().unwrap(), 0o755, true).is_ok());
        assert!(deep.exists());

        // rmdir is not recursive, so clean up manually
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn test_fs_rename() {
        let dir = temp_dir();
        let old = dir.join("old.txt");
        let new = dir.join("new.txt");
        std::fs::write(&old, b"data").unwrap();

        assert!(fs::rename(old.to_str().unwrap(), new.to_str().unwrap()).is_ok());
        assert!(!old.exists());
        assert!(new.exists());

        drop_dir(&dir);
    }

    #[test]
    fn test_fs_unlink() {
        let dir = temp_dir();
        let file = dir.join("unlink_test.txt");
        std::fs::write(&file, b"data").unwrap();
        assert!(file.exists());

        assert!(fs::unlink(file.to_str().unwrap()).is_ok());
        assert!(!file.exists());

        drop_dir(&dir);
    }

    #[test]
    fn test_fs_copy_file() {
        let dir = temp_dir();
        let src = dir.join("src.txt");
        let dst = dir.join("dst.txt");
        std::fs::write(&src, b"copy me").unwrap();

        assert!(fs::copy_file(src.to_str().unwrap(), dst.to_str().unwrap(), 0).is_ok());
        assert!(dst.exists());
        assert_eq!(std::fs::read_to_string(&dst).unwrap(), "copy me");

        drop_dir(&dir);
    }

    #[test]
    fn test_fs_readdir() {
        let dir = temp_dir();
        std::fs::write(dir.join("a.txt"), b"").unwrap();
        std::fs::write(dir.join("b.txt"), b"").unwrap();

        let result = fs::readdir(dir.to_str().unwrap(), "utf8", false);
        assert!(result.is_ok());
        let val = result.unwrap();
        let names: Vec<String> = val
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert!(names.iter().any(|n| n.contains("a.txt")));
        assert!(names.iter().any(|n| n.contains("b.txt")));

        drop_dir(&dir);
    }

    // ── constants tests ─────────────────────────────────────────────────

    #[test]
    fn test_fs_flags_not_empty() {
        let flags = constants::fs_flags();
        assert!(!flags.is_empty());
        assert!(flags.iter().any(|(name, _)| *name == "O_RDONLY"));
    }

    #[test]
    fn test_signals_not_empty() {
        let sigs = constants::signals();
        assert!(!sigs.is_empty());
        assert!(sigs.iter().any(|(name, _)| *name == "SIGINT"));
    }

    // ── os tests ────────────────────────────────────────────────────────

    #[test]
    fn test_get_pid() {
        assert!(os::get_pid() > 0);
    }

    #[test]
    fn test_is_big_endian() {
        // Just ensure it returns a bool (no panic)
        let _ = os::is_big_endian();
    }

    #[test]
    fn test_get_available_parallelism() {
        assert!(os::get_available_parallelism() > 0);
    }
}

pub mod process_dlopen {
    use boa_engine::{Context, JsError, JsNativeError, JsObject};

    pub fn dlopen_impl(module: &JsObject, filename: &str, ctx: &mut Context) -> Result<(), JsError> {
        let lib = unsafe { libloading::Library::new(filename) }.map_err(|e| {
            JsNativeError::error().with_message(format!("Cannot open native addon '{}': {}", filename, e))
        })?;

        let register: libloading::Symbol<
            unsafe extern "C" fn(
                env: *mut crate::napi::env::NapiEnv,
                exports: *mut std::ffi::c_void,
            ) -> *mut std::ffi::c_void,
        > = unsafe {
            lib.get(b"napi_register_module_v1").or_else(|_| {
                lib.get(b"node_register_module")
            })
        }.map_err(|e| {
            JsNativeError::error().with_message(format!(
                "Not a valid .node addon '{}': N-API entry not found ({})",
                filename, e
            ))
        })?;

        let napi_env = unsafe { crate::napi::create_napi_env(ctx) };
        let env_ptr = Box::into_raw(napi_env);

        let exports = JsObject::with_object_proto(ctx.intrinsics());
        let exports_ptr = &exports as *const JsObject as *mut std::ffi::c_void;

        unsafe {
            register(env_ptr, exports_ptr);
        }

        let exports_props = exports.borrow();
        let index_count = exports_props.properties().index_properties().count();
        for i in 0..index_count {
            let key: boa_engine::property::PropertyKey = i.into();
            if let Some(desc) = exports_props.properties().get(&key) {
                if let Some(val) = desc.value() {
                    module.insert_property(
                        key,
                        boa_engine::property::PropertyDescriptor::builder()
                            .value(val.clone())
                            .writable(true)
                            .enumerable(true)
                            .configurable(true),
                    );
                }
            }
        }

        std::mem::forget(lib);
        Ok(())
    }
}
