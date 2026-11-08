// #8174 구조 조사 probe. 제품 source: def947c2149df5bc812d801e6e79be19a403b4d1.
// 실행 조건과 관측 범위는 ../README.md에 기록한다. 정식 회귀 검사가 아니다.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = rhwp::DocumentCore::from_bytes(b"invalid")?;
    Ok(())
}
