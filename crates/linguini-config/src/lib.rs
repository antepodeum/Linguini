pub const DEFAULT_CONFIG_FILE: &str = "linguini.toml";

#[cfg(test)]
mod tests {
    use super::DEFAULT_CONFIG_FILE;

    #[test]
    fn default_config_file_matches_spec() {
        assert_eq!(DEFAULT_CONFIG_FILE, "linguini.toml");
    }
}
