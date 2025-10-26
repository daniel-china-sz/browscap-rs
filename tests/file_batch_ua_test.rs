#[cfg(test)]
mod file_batch_ua_test {
    use browscap_rs::Capabilities;
    use log::debug;
    use std::fs::File;
    use std::io::{stdin, BufRead, BufReader, Read};
    use std::time::Instant;

    #[test]
    fn test_load_parser_file() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init();
        let file = File::open("useragents.txt").unwrap();
        let reader = BufReader::new(file);
        let parser = browscap_rs::load_parser_default().unwrap();
        let mut i = 0;
        for line in reader.lines() {
            i += 1;
            // debug!("处理第{}行",i);
            let line = line.unwrap();
            if line.starts_with("#") {
                continue;
            }
            let properties: Vec<&str> = line.split("    ").collect();
            if properties.len() < 5 {
                continue;
            }
            let timer=Instant::now();
            let capabilities: &Capabilities = parser.parse(properties.get(5).unwrap());
            let time=timer.elapsed();
            // debug!("解析一条用时：{:?}",time);
            let mut y: usize = 0;
            assert_eq!(
                *properties.get(y).unwrap(),
                capabilities.get_browser().unwrap()
            );
            y += 1;
            assert_eq!(
                *properties.get(y).unwrap(),
                capabilities.get_browser_major_version().unwrap()
            );
            y += 1;
            assert_eq!(
                *properties.get(y).unwrap(),
                capabilities.get_platform().unwrap()
            );
            y += 1;
            assert_eq!(
                *properties.get(y).unwrap(),
                capabilities.get_platform_version().unwrap()
            );
            y += 1;
            assert_eq!(
                *properties.get(y).unwrap(),
                capabilities.get_device_type().unwrap()
            );
        }
    }

     #[test]
    fn test_memory() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init();
        let mut buf=String::new();
        stdin().read_line(&mut buf).unwrap();
        let file = File::open("useragents.txt").unwrap();
        stdin().read_line(&mut buf).unwrap();
        let reader = BufReader::new(file);
        stdin().read_line(&mut buf).unwrap();
        let parser = browscap_rs::load_parser_default().unwrap();
        stdin().read_line(&mut buf).unwrap();
        let mut i = 0;
        for line in reader.lines() {
            i += 1;
            let line = line.unwrap();
            if line.starts_with("#") {
                continue;
            }
            let properties: Vec<&str> = line.split("    ").collect();
            if properties.len() < 5 {
                continue;
            }
            stdin().read_line(&mut buf).unwrap();
            let capabilities: &Capabilities = parser.parse(properties.get(5).unwrap());
            println!("d");
            let mut y: usize = 0;
            assert_eq!(
                *properties.get(y).unwrap(),
                capabilities.get_browser().unwrap()
            );
            y += 1;
            assert_eq!(
                *properties.get(y).unwrap(),
                capabilities.get_browser_major_version().unwrap()
            );
            y += 1;
            assert_eq!(
                *properties.get(y).unwrap(),
                capabilities.get_platform().unwrap()
            );
            y += 1;
            assert_eq!(
                *properties.get(y).unwrap(),
                capabilities.get_platform_version().unwrap()
            );
            y += 1;
            assert_eq!(
                *properties.get(y).unwrap(),
                capabilities.get_device_type().unwrap()
            );
        }
    }
}
