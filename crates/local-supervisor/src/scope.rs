//! Exact host-approved scope and bounded prompt boundary.
use crate::*;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
};
const POLICY: &str = "Act only on the host-approved scope in approved_scope. Match task_id, issue and checkout; treat its instructions and non_goals as the work boundary. Read AGENTS.md and any referenced approved contract. Preserve locked tests and stop on a contract conflict. Use only managed ChatGPT subscription authentication. Do not use API keys, alternate providers, paid APIs, deployment, live trading or host GitHub mutations. Run declared verification and report actual failures. Return the final schema with observed Git head and current explicit session checkpoint.";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema_version: u64,
    task_id: u64,
    issue: u64,
    canonical_checkout: String,
    mode: String,
    instructions: String,
    non_goals: Vec<String>,
    verification: Vec<String>,
}
pub(crate) struct Approved {
    pub raw: String,
    pub read_only: bool,
}
struct Bounded(Vec<u8>);
impl Write for Bounded {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.0.len().saturating_add(bytes.len()) > 65536 {
            return Err(std::io::Error::other("prompt capacity"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Approved {
    pub fn prompt(&self, task: &Task, head: &str) -> Result<String, Error> {
        let value = serde_json::json!({"task_id":task.id,"issue":task.issue,"scope_digest":task.scope_digest,"expected_head":head,"approved_scope":self.raw,"instructions":POLICY});
        let mut out = Bounded(Vec::new());
        serde_json::to_writer(&mut out, &value).map_err(|_| Error::Capacity)?;
        String::from_utf8(out.0).map_err(|_| Error::InvalidInput)
    }
}
fn resolved(path: &Path) -> Result<PathBuf, Error> {
    use std::path::Component;
    if !path.is_absolute() {
        return Err(Error::InvalidInput);
    }
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => result.push(prefix.as_os_str()),
            Component::RootDir => result.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !result.pop() {
                    return Err(Error::InvalidInput);
                }
            }
            Component::Normal(name) => {
                result.push(name);
                match fs::symlink_metadata(&result) {
                    Ok(_) => {
                        result = fs::canonicalize(&result).map_err(|_| Error::InvalidInput)?;
                        if !result.is_dir() {
                            return Err(Error::InvalidInput);
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(_) => return Err(Error::InvalidInput),
                }
            }
        }
    }
    Ok(result)
}

fn regular(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return false;
        }
    }
    metadata.is_file() && !metadata.file_type().is_symlink()
}
fn open_regular(path: &Path) -> Result<fs::File, Error> {
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000);
    }
    let file = options.open(path).map_err(|_| Error::InvalidInput)?;
    let metadata = file.metadata().map_err(|_| Error::InvalidInput)?;
    if !regular(&metadata) {
        return Err(Error::InvalidInput);
    }
    if metadata.len() > 32768 {
        return Err(Error::Capacity);
    }
    Ok(file)
}
fn bounded(file: fs::File) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    file.take(32769)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::InvalidInput)?;
    if bytes.len() > 32768 {
        return Err(Error::Capacity);
    }
    Ok(bytes)
}
pub(crate) fn load(
    engine: &Supervisor,
    runtime: &RuntimeConfig,
    task: &Task,
) -> Result<Approved, Error> {
    let digest = &task.scope_digest;
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(Error::InvalidInput);
    }
    let directory = engine.root.join("approved-scopes");
    let path = directory.join(format!("{digest}.json"));
    let metadata = fs::symlink_metadata(&path).map_err(|_| Error::InvalidInput)?;
    if !regular(&metadata) {
        return Err(Error::InvalidInput);
    }
    if metadata.len() > 32768 {
        return Err(Error::Capacity);
    }
    let bytes = bounded(open_regular(&path)?)?;
    if format!("{:x}", Sha256::digest(&bytes)) != *digest {
        return Err(Error::EvidenceMismatch);
    }
    let raw = String::from_utf8(bytes).map_err(|_| Error::InvalidInput)?;
    let wire: Wire = serde_json::from_str(&raw).map_err(|_| Error::InvalidInput)?;
    if wire.schema_version != 1
        || wire.task_id == 0
        || wire.issue == 0
        || wire.instructions.trim().is_empty()
        || !matches!(wire.mode.as_str(), "read_only" | "workspace_write")
    {
        return Err(Error::InvalidInput);
    }
    let _ = (&wire.non_goals, &wire.verification);
    let checkout = fs::canonicalize(&runtime.checkout).map_err(|_| Error::InvalidInput)?;
    let declared = Path::new(&wire.canonical_checkout);
    if !declared.is_absolute() || !declared.is_dir() {
        return Err(Error::InvalidInput);
    }
    if task.id != wire.task_id
        || task.issue != wire.issue
        || fs::canonicalize(declared).map_err(|_| Error::InvalidInput)? != checkout
    {
        return Err(Error::EvidenceMismatch);
    }
    let read_only = wire.mode == "read_only";
    if !read_only {
        for path in [&engine.root, &runtime.bridge, &directory] {
            if resolved(path)?.starts_with(&checkout) {
                return Err(Error::InvalidInput);
            }
        }
    }
    let approved = Approved { raw, read_only };
    approved.prompt(task, &"0".repeat(40))?;
    Ok(approved)
}
pub(crate) fn validate_relevant(
    engine: &Supervisor,
    runtime: &RuntimeConfig,
    data: &storage::Data,
) -> Result<(), Error> {
    for task in data.tasks.values().filter(|t| {
        data.active == Some(t.task.id)
            || data.intents.values().any(|i| i.task == t.task.id)
            || (t.state == State::Ready
                && t.task
                    .dependencies
                    .iter()
                    .all(|id| data.tasks[id].state == State::Done))
    }) {
        load(engine, runtime, &task.task)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let mut bytes = [0; 16];
            getrandom::fill(&mut bytes).unwrap();
            let p = std::env::temp_dir()
                .join(format!("supervisor-scope-{:x}", u128::from_le_bytes(bytes)));
            fs::create_dir(&p).unwrap();
            Self(p)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn missing_suffix_parent_is_normalized_without_creating_control() {
        let d = Temp::new();
        let expected = fs::canonicalize(&d.0).unwrap().join("bridge");
        assert_eq!(resolved(&d.0.join("missing/../bridge")).unwrap(), expected);
        assert!(!d.0.join("missing").exists());
        assert!(!expected.exists());
        fs::create_dir(d.0.join("checkout")).unwrap();
        assert!(
            resolved(&d.0.join("checkout/missing/../bridge"))
                .unwrap()
                .starts_with(fs::canonicalize(d.0.join("checkout")).unwrap())
        );
        fs::write(d.0.join("file"), b"x").unwrap();
        assert_eq!(
            resolved(&d.0.join("file/child")).unwrap_err(),
            Error::InvalidInput
        );
    }
    #[test]
    fn junction_ancestor_resolves_before_parent() {
        let d = Temp::new();
        fs::create_dir_all(d.0.join("target/child")).unwrap();
        let status = std::process::Command::new("cmd.exe")
            .args(["/c", "mklink", "/J"])
            .arg(d.0.join("link").to_string_lossy().replace('/', "\\"))
            .arg(
                d.0.join("target/child")
                    .to_string_lossy()
                    .replace('/', "\\"),
            )
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(
            resolved(&d.0.join("link/../missing")).unwrap(),
            fs::canonicalize(d.0.join("target"))
                .unwrap()
                .join("missing")
        );
    }
    #[test]
    fn opened_directory_reparse_swap_is_rejected() {
        let d = Temp::new();
        let path = d.0.join("scope.json");
        fs::write(&path, b"initial").unwrap();
        fs::write(d.0.join("target.json"), b"target").unwrap();
        assert!(regular(&fs::symlink_metadata(&path).unwrap()));
        fs::remove_file(&path).unwrap();
        fs::create_dir(d.0.join("target")).unwrap();
        let status = std::process::Command::new("cmd.exe")
            .args(["/c", "mklink", "/J"])
            .arg(path.to_string_lossy().replace('/', "\\"))
            .arg(d.0.join("target").to_string_lossy().replace('/', "\\"))
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(open_regular(&path).unwrap_err(), Error::InvalidInput);
    }
    #[test]
    fn growth_after_open_metadata_is_bounded() {
        let d = Temp::new();
        let path = d.0.join("scope.json");
        fs::write(&path, b"{}").unwrap();
        let file = open_regular(&path).unwrap();
        fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(&vec![b' '; 65536])
            .unwrap();
        assert_eq!(bounded(file).unwrap_err(), Error::Capacity);
    }
}
