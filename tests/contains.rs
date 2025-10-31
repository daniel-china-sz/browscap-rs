#[cfg(test)]
mod t {

    use memchr::memmem;
    use std::hash::BuildHasher;
    use std::hash::Hasher;
    use std::hash::RandomState;
    use std::time::Instant;

    use log::debug;

    #[test]
    fn test_contains_speed() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init();
        let mut vec: Vec<&'static str> = Vec::new();

        for _i in 0..50000000 {
            let str = hash_based_random_string(
                5 as usize,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
            vec.push(Box::leak(str.into_boxed_str()));
        }

        let now = Instant::now();
        let str = "dfoskfwoef.sdjfasdifjefnsdlflsdifowensldofwndsdfffffffffffffffffffffffffffffffffffff";
        const CHUNK_SIZE: usize = 1000;
       for chunk in vec.chunks(CHUNK_SIZE) {
        for pattern in chunk {
             str.contains(pattern) ;
        }
    }
        let time = now.elapsed();
        debug!("用时:{:?}", time);
    }

    fn hash_based_random_string(length: usize, seed: u64) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

        let hasher = RandomState::new().build_hasher();
        let mut result = String::with_capacity(length);
        let mut current_seed = seed;

        for i in 0..length {
            let mut hasher = RandomState::new().build_hasher();
            hasher.write_u64(current_seed);
            hasher.write_usize(i);

            let hash = hasher.finish();
            let idx = (hash % CHARSET.len() as u64) as usize;
            result.push(CHARSET[idx] as char);

            current_seed = hash;
        }

        result
    }
}
