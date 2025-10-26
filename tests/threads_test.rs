#[cfg(test)]
mod threads_test {
    use browscap_rs::UserAgentParser;
    use std::fs::File;
    use std::io::BufRead;
    use std::sync::{Arc, Mutex, RwLock};
    use std::thread;

    #[test]
    pub fn multiple_thread() {
        let _s: Option<String> = Some("hello".to_string());

        let parser: Arc<RwLock<UserAgentParser>> =
            Arc::new(RwLock::new(browscap_rs::load_parser_default().unwrap()));
        let lines = load_file_line();
        for _i in 0..10 {
            let thread_lines = Arc::clone(&lines);
            let thread_parser = Arc::clone(&parser);
            thread::spawn(move || {
                let lock = thread_lines.lock().unwrap();
                let option = lock.first();
                if let Some(option) = option {
                    let (b, bmv, p, pv, dt, ua) = parse_file_line(option);
                    let t=thread_parser.read().unwrap();
                    let capabilites = t.parse(ua);
                    assert_eq!(b, capabilites.get_browser().unwrap());
                    assert_eq!(bmv, capabilites.get_browser_major_version().unwrap());
                    assert_eq!(p, capabilites.get_platform().unwrap());
                    assert_eq!(pv, capabilites.get_platform_version().unwrap());
                    assert_eq!(dt, capabilites.get_device_type().unwrap());
                }
            });
        }
    }

    fn parse_file_line(s: &str) -> (&str, &str, &str, &str, &str, &str) {
        let mut split = s.split("    ");
        return (
            split.next().unwrap(),
            split.next().unwrap(),
            split.next().unwrap(),
            split.next().unwrap(),
            split.next().unwrap(),
            split.next().unwrap(),
        );
    }

    fn load_file_line() -> Arc<Mutex<Vec<String>>> {
        let file = File::open("useragents3.txt").unwrap();
        let mut vec: Vec<String> = Vec::new();
        for line in std::io::BufReader::new(file).lines() {
            let line = line.unwrap();
            vec.push(line);
        }
        Arc::new(Mutex::new(vec))
    }
}
