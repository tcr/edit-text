#[macro_use]
extern crate commandspec;

#[cfg(not(windows))]
mod sh {
    #[test]
    fn sh_exit() {
        let res = sh_execute!(r"exit {a}", a = 42).unwrap_err();
        assert_eq!(res.error_code(), 42);
    }

    #[test]
    fn sh_echo1() {
        let res = sh_command!(
            r"A={a}; echo $A",
            a = "SENTINEL"
        ).unwrap().output().unwrap();
        assert_eq!(res.stdout, b"SENTINEL\n");
    }

    #[test]
    fn sh_echo2() {
        let res = sh_command!(
            r"A={a}; echo $A",
            a = "SENTINEL",
        ).unwrap().output().unwrap();
        assert_eq!(res.stdout, b"SENTINEL\n");
    }

    #[test]
    fn sh_empty() {
        sh_execute!(r"true").unwrap();
    }

    #[test]
    fn sh_empty_comma() {
        sh_execute!(r"true", ).unwrap();
    }
}

#[test]
fn cmd_rustc() {
    let args = vec!["-V"];
    let res = command!(
        r"
            rustc {args}
        ",
        args = args,
    ).unwrap().output().unwrap();
    assert!(res.stdout.starts_with(b"rustc "));
}
