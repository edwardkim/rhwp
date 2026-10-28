//! Public containers produced by the 0.10.1 product must remain readable.
use rhwp::{agent_seal as agent, security_trailer as trailer};

const HOST: &[u8] = b"RHWP T06 visible public host";
const SECRET: &[u8] = b"T06 compatibility payload 2026";
const PASSWORD: &[u8] = b"T06-test-password-only";
const KEY: [u8; 32] = [42; 32];
const AGENT: &[u8] = include_bytes!("../fixtures/issue8174_crypto_compat/agent.bin");
const PASSWORD_DATA: &[u8] = include_bytes!("../fixtures/issue8174_crypto_compat/password.bin");
const PQ: &[u8] = include_bytes!("../fixtures/issue8174_crypto_compat/pq.bin");
const PRIVATE: &[u8] = include_bytes!("../fixtures/issue8174_crypto_compat/pq-private.bin");

#[test]
fn existing_agent_container_preserves_authentication_and_new_roundtrip() {
    let expected = agent::Opened::Sealed {
        plaintext: SECRET.to_vec(),
    };
    assert_eq!(agent::open_with_key(AGENT, &KEY), expected);
    assert!(matches!(
        agent::open_with_key(AGENT, &[43; 32]),
        agent::Opened::Broken { .. }
    ));
    for index in [HOST.len() + 11, HOST.len() + 11 + 24] {
        let mut mutated = AGENT.to_vec();
        mutated[index] ^= 1;
        assert!(matches!(
            agent::open_with_key(&mutated, &KEY),
            agent::Opened::Broken { .. }
        ));
    }
    assert_eq!(
        agent::open_with_key(&agent::seal_with_key(HOST, SECRET, &KEY), &KEY),
        expected
    );
    assert_eq!(agent::open_with_key(HOST, &KEY), agent::Opened::Plain);
}

#[test]
fn existing_password_and_pq_trailers_preserve_authentication_and_visible_layer() {
    assert!(
        matches!(trailer::open(PASSWORD_DATA, PASSWORD), trailer::Opened::Sealed { plaintext, .. } if plaintext == SECRET)
    );
    assert!(matches!(
        trailer::open(PASSWORD_DATA, b"wrong"),
        trailer::Opened::Broken { .. }
    ));
    assert_eq!(trailer::visible_layer(PASSWORD_DATA), HOST);
    for index in [HOST.len() + 14, HOST.len() + 58] {
        let mut mutated = PASSWORD_DATA.to_vec();
        mutated[index] ^= 1;
        assert!(matches!(
            trailer::open(&mutated, PASSWORD),
            trailer::Opened::Broken { .. }
        ));
    }
    assert!(
        matches!(trailer::open_with_privkey(PQ, PRIVATE), trailer::Opened::Sealed { plaintext, .. } if plaintext == SECRET)
    );
    let mut wrong = PRIVATE.to_vec();
    wrong[0] ^= 1;
    assert!(matches!(
        trailer::open_with_privkey(PQ, &wrong),
        trailer::Opened::Broken { .. }
    ));
    let mut mutated = PQ.to_vec();
    mutated[HOST.len() + 18 + 1088 + 24] ^= 1;
    assert!(matches!(
        trailer::open_with_privkey(&mutated, PRIVATE),
        trailer::Opened::Broken { .. }
    ));
    let new = trailer::seal(HOST, SECRET, PASSWORD).unwrap();
    assert!(
        matches!(trailer::open(&new, PASSWORD), trailer::Opened::Sealed { plaintext, .. } if plaintext == SECRET)
    );
    assert_eq!(trailer::open(HOST, PASSWORD), trailer::Opened::Plain);
}
