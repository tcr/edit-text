fn to_strings(input: &[&str]) -> Vec<String> {
    input.iter().map(|x| x.to_string()).collect()
}

pub fn watchexec_args(cmd: &str, ignores: &[&str]) -> watchexec::cli::Args {
    let src_default_ignores = &["*/.DS_Store", "*/.git/**", "*/.svn/**", "*/target/**"][..];

    watchexec::cli::Args {
        cmd: to_strings(&[cmd]),
        ignores: to_strings(&[src_default_ignores, ignores].concat()),
        debug: false,

        // Mostly defaults from cargo-watch.
        paths: to_strings(&["."]),
        filters: to_strings(&[]),
        run_initially: true,
        clear_screen: false,
        signal: None,
        restart: true,
        debounce: 300,
        no_shell: false,
        no_vcs_ignore: false,
        once: false,
        poll: false,
        poll_interval: 0,
    }
}
