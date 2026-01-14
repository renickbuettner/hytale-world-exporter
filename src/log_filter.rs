/// Log filter module for filtering log lines
///
/// This module provides functionality to filter out noise from log files,
/// such as INFO messages and setup/shutdown progress lines.

/// Patterns that indicate lines to filter when "errors only" mode is enabled
const FILTER_PATTERNS: &[&str] = &[
    "INFO]",
    "-=|Setup|",
    "=|Setup|",
    "-=|Shutdown Modules|",
    "=|Shutdown Modules|",
];

/// Checks if a log line should be filtered out
///
/// # Arguments
/// * `line` - The log line to check
/// * `filter_enabled` - Whether filtering is enabled
///
/// # Returns
/// * `true` if the line should be hidden (filtered out)
/// * `false` if the line should be shown
pub fn should_filter_line(line: &str, filter_enabled: bool) -> bool {
    if !filter_enabled {
        return false;
    }

    FILTER_PATTERNS.iter().any(|pattern| line.contains(pattern))
}

/// Determines the log level of a line for styling purposes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
}

/// Detects the log level of a given line
///
/// # Arguments
/// * `line` - The log line to analyze
///
/// # Returns
/// The detected LogLevel
pub fn detect_log_level(line: &str) -> LogLevel {
    if line.contains("ERROR") {
        LogLevel::Error
    } else if line.contains("WARN") {
        LogLevel::Warning
    } else {
        LogLevel::Info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_disabled() {
        assert!(!should_filter_line("INFO] Some message", false));
        assert!(!should_filter_line("-=|Setup|50.0", false));
    }

    #[test]
    fn test_filter_info_lines() {
        assert!(should_filter_line("INFO] Some message", true));
        assert!(should_filter_line("[2026-01-14] INFO] Test", true));
    }

    #[test]
    fn test_filter_setup_lines() {
        assert!(should_filter_line("=|Setup|7.000000000000001", true));
        assert!(should_filter_line("-=|Setup|9.0", true));
        assert!(should_filter_line("-=|Setup|10.0", true));
    }

    #[test]
    fn test_filter_shutdown_lines() {
        assert!(should_filter_line("-=|Shutdown Modules|86.0", true));
        assert!(should_filter_line("=|Shutdown Modules|88.0", true));
    }

    #[test]
    fn test_keep_error_lines() {
        assert!(!should_filter_line("ERROR] Something went wrong", true));
    }

    #[test]
    fn test_keep_warn_lines() {
        assert!(!should_filter_line("WARN] Warning message", true));
    }

    #[test]
    fn test_detect_log_level() {
        assert_eq!(detect_log_level("ERROR] Test"), LogLevel::Error);
        assert_eq!(detect_log_level("WARN] Test"), LogLevel::Warning);
        assert_eq!(detect_log_level("INFO] Test"), LogLevel::Info);
        assert_eq!(detect_log_level("Some other line"), LogLevel::Info);
    }
}

