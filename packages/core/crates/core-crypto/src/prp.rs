use crate::primitives::{calculate_mac, SimpleStreamCipher};

const INTERMEDIATE_KEY_LENGTH: usize = 32;
const INTERMEDIATE_IV_LENGTH: usize = 16;

// The minimum input length must be at least size of the key, which is XORed with plaintext/ciphertext
const PRP_MIN_LENGTH: usize = INTERMEDIATE_KEY_LENGTH;

/// Implementation of Pseudo-Random Permutation (PRP).
/// Currently based on Lioness wide-block cipher
pub struct PRP {
    keys: [Vec<u8>; 4],
    ivs: [Vec<u8>; 4]
}

impl PRP {

    /// Creates new instance of the PRP
    pub fn new(key: &[u8], iv: &[u8]) -> Self {
        Self {
            keys: [
                key[0*INTERMEDIATE_KEY_LENGTH..1*INTERMEDIATE_KEY_LENGTH].to_vec(),
                key[1*INTERMEDIATE_KEY_LENGTH..2*INTERMEDIATE_KEY_LENGTH].to_vec(),
                key[2*INTERMEDIATE_KEY_LENGTH..3*INTERMEDIATE_KEY_LENGTH].to_vec(),
                key[3*INTERMEDIATE_KEY_LENGTH..4*INTERMEDIATE_KEY_LENGTH].to_vec()
            ],
            ivs: [ // NOTE: ChaCha20 takes only 12 byte IV
                iv[0*INTERMEDIATE_IV_LENGTH..1*INTERMEDIATE_IV_LENGTH].to_vec(),
                iv[1*INTERMEDIATE_IV_LENGTH..2*INTERMEDIATE_IV_LENGTH].to_vec(),
                iv[2*INTERMEDIATE_IV_LENGTH..3*INTERMEDIATE_IV_LENGTH].to_vec(),
                iv[3*INTERMEDIATE_IV_LENGTH..4*INTERMEDIATE_IV_LENGTH].to_vec()
            ]
        }
    }

    /// Applies forward permutation on the given plaintext and returns a new buffer
    /// containing the result.
    pub fn forward(&self, plaintext: &[u8]) -> Result<Box<[u8]>, String> {
        if plaintext.len() < PRP_MIN_LENGTH {
            return Err(format!("Expected plaintext with a length of a least {} bytes. Got {}.", PRP_MIN_LENGTH, plaintext.len()));
        }

        let mut out = Vec::from(plaintext);
        let data = out.as_mut_slice();

        Self::xor_keystream(data, &self.keys[0], &self.ivs[0])?;
        Self::xor_hash(data, &self.keys[1], &self.ivs[1])?;
        Self::xor_keystream(data, &self.keys[2], &self.ivs[2])?;
        Self::xor_hash(data, &self.keys[3], &self.ivs[3])?;

        Ok(out.into_boxed_slice())
    }

    /// Applies inverse permutation on the given plaintext and returns a new buffer
    /// containing the result.
    pub fn inverse(&self, ciphertext: &[u8]) -> Result<Box<[u8]>, String> {
        if ciphertext.len() < PRP_MIN_LENGTH {
            return Err(format!("Expected plaintext with a length of a least {} bytes. Got {}.", PRP_MIN_LENGTH, ciphertext.len()));
        }

        let mut out = Vec::from(ciphertext);
        let data = out.as_mut_slice();

        Self::xor_hash(data, &self.keys[3], &self.ivs[3])?;
        Self::xor_keystream(data, &self.keys[2], &self.ivs[2])?;
        Self::xor_hash(data, &self.keys[1], &self.ivs[1])?;
        Self::xor_keystream(data, &self.keys[0], &self.ivs[0])?;

        Ok(out.into_boxed_slice())
    }

    // Internal helper functions

    fn xor_hash(data: &mut [u8], key: &[u8], iv: &[u8]) -> Result<(), String> {
        let res = calculate_mac([key, iv].concat().as_slice(), &data[PRP_MIN_LENGTH..])?;
        Self::xor_inplace(data, res.as_ref());
        Ok(())
    }

    fn xor_inplace(a: &mut [u8], b: &[u8]) {
        let bound = if a.len() > b.len() { b.len() } else { a.len() };
        for i in 0..bound {
            a[i] = a[i] ^ b[i];
        }
    }

    fn xor_keystream(data: &mut [u8], key: &[u8], iv: &[u8]) -> Result<(), String> {
        let mut key_cpy = Vec::from(key);
        Self::xor_inplace(key_cpy.as_mut_slice(), &data[0..PRP_MIN_LENGTH]);
        let mut cipher = SimpleStreamCipher::new(key_cpy.as_slice(), iv)?;
        cipher.apply(&mut data[PRP_MIN_LENGTH..]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use getrandom::getrandom;
    use crate::prp::PRP;

    #[test]
    fn test_prp_fixed() {
        let key = [0u8; 4*32];
        let iv = [0u8; 4*16];

        let prp = PRP::new(&key, &iv);

        let data = [1u8; 278];

        let ct = prp.forward(&data).unwrap();
        let pt = prp.inverse(&ct).unwrap();

        assert_eq!(&data, pt.as_ref());
    }

    #[test]
    fn test_prp_random() {
        let mut key = [0u8; 4*32];
        getrandom(&mut key).unwrap();

        let mut iv = [0u8; 4*16];
        getrandom(&mut iv).unwrap();

        let prp = PRP::new(&key, &iv);

        let mut data = [1u8; 278];
        getrandom(&mut data).unwrap();

        let ct = prp.forward(&data).unwrap();
        let pt = prp.inverse(&ct).unwrap();

        assert_eq!(&data, pt.as_ref());
    }
}