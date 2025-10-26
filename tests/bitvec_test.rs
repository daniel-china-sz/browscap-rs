#[cfg(test)]
mod bitvec_test {

    use base64::engine::general_purpose::STANDARD;
    use base64::prelude::*;
    use bitvec::vec::BitVec;

    #[cfg(test)]
    pub fn bitset_base64(bit_set: &BitVec) -> String {
        let bit_len = bit_set.len();
        let byte_len = (bit_len + 7) / 8;
        let mut bytes = vec![0u8; byte_len];

        for i in 0..bit_len {
            if bit_set[i] {
                let byte_index = i / 8;
                let bit_index = i % 8;
                bytes[byte_index] |= 1 << bit_index;
            }
        }

        // 简单裁剪：找到最后一个非零字节
        let last_non_zero = bytes
            .iter()
            .rposition(|&b| b != 0)
            .map(|pos| pos + 1)
            .unwrap_or(0); // 至少1字节

        let mut str: String = String::new();
        STANDARD.encode_string(&bytes[..last_non_zero], &mut str);
        str
    }
}
