
#[cfg(target_os = "windows")]
pub const JAVA_EXECUTABLE: &str = "java.exe";
#[cfg(not(target_os = "windows"))]
pub const JAVA_EXECUTABLE: &str = "java";

#[cfg(target_os = "windows")]
pub(crate) const CLASSPATH_SEPARATOR: &str = ";";
#[cfg(not(target_os = "windows"))]
pub const CLASSPATH_SEPARATOR: &str = ":";