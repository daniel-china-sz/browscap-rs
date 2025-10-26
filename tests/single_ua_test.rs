#[cfg(test)]
mod single_ua_test {
    use browscap_rs::{BrowsCapField, Capabilities, IS_TABLES};

    #[test]
    fn test_load_parser_default() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init();

        let  parser = browscap_rs::load_parser_default();
        match parser {
            Ok(p) => {
                let capabilities: &Capabilities = p.parse("Mozilla/5.0 (iPhone; CPU iPhone OS 10_2_1 like Mac OS X) AppleWebKit/602.4.6 (KHTML, like Gecko) Version/10.0 Mobile/14D27 Safari/602.1");
                print!("{:?}", capabilities)
            }
            Err(e) => {
                println!("{}", e.to_string())
            }
        }
    }

    #[test]
    fn test_load_parser_fields() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init();
        let my_fields: Vec<&BrowsCapField> = vec![&IS_TABLES];
        let parser = browscap_rs::load_parser_with_fields(my_fields);
        match parser {
            Ok(p) => {
                let capabilities: &Capabilities = p.parse("Mozilla/5.0 (iPhone; CPU iPhone OS 10_2_1 like Mac OS X) AppleWebKit/602.4.6 (KHTML, like Gecko) Version/10.0 Mobile/14D27 Safari/602.1");
                print!("{:?}", capabilities)
            }
            Err(e) => {
                println!("{}", e.to_string())
            }
        }
    }
}
