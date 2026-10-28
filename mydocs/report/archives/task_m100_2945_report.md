# Task m100-2945 처리 결과 보고서

## 이슈
#2945 — `write_hwp_string()` 공용 헬퍼 길이 미검증 → u16 캐스팅 랩어라운드로 CTRL_DATA/STYLE 레코드 손상 (HWPX→HWP5 export, 스튜디오 가드 우회)

## 배경
오늘 rhwp-studio 다이얼로그 쪽에서 발견·수정된 u16 랩어라운드 결함 클래스(필드 이름 #2851,
책갈피 이름 #2862, 스타일 이름 #2866, ClickHere 안내문/메모 #2878, 개체 설명문/수식 스크립트 #2892)는
전부 rhwp-studio UI 레이어(다이얼로그 입력창 글자수 제한)에서 대응되었다. 그러나 실제로 u16 길이
프리픽스를 기록하는 Rust 코어 직렬화 계층의 공용 함수 `src/serializer/byte_writer.rs`의
`ByteWriter::write_hwp_string()` 자체는 손대지 않은 채로 남아 있었다.

이 함수는 스타일 이름/영문 이름(`doc_info.rs`), 개체 설명문(`common_obj_attr_writer.rs`,
`control.rs`), 수식 스크립트/버전정보/폰트명(`control.rs`), 글꼴명(`doc_info.rs`) 등 오늘 고쳐진
필드들을 포함해 다수의 호출부가 공유하는 공용 헬퍼다. 스튜디오 UI 다이얼로그를 거치지 않고 HWPX
파일을 import하여 HWP5로 export하는 경로(CLI, 라이브러리 API 직접 호출 등)로는 65,536 UTF-16
코드유닛을 초과하는 문자열이 스튜디오의 글자수 가드를 우회해 그대로 `write_hwp_string()`에
도달하고, `utf16.len() as u16` 캐스팅이 wraparound되어 기록된 길이 필드와 실제 UTF-16 바이트열이
어긋난 손상된 CTRL_DATA/STYLE 레코드가 만들어진다.

동일한 결함 클래스이지만, 개별 호출부(스튜디오 다이얼로그)가 아니라 공용 직렬화 함수 자체를
고치는 것이므로 파급 범위가 훨씬 넓고 diff는 오히려 더 작다.

## 원인
```rust
pub fn write_hwp_string(&mut self, s: &str) -> io::Result<()> {
    let utf16: Vec<u16> = s.encode_utf16().collect();
    self.write_u16(utf16.len() as u16)?;   // 검증 없이 그대로 as u16 캐스팅
    for code_unit in &utf16 {
        self.write_u16(*code_unit)?;
    }
    Ok(())
}
```
`utf16.len()`이 65536(0x10000) 이상이면 `as u16` 캐스팅이 0으로 wraparound되어, 뒤따르는
UTF-16 바이트열과 기록된 길이 필드가 불일치하는 레코드가 생성된다.

## 수정
`write_hwp_string()` 내부에서 `u16::MAX` 초과 시 서로게이트 쌍을 보존하며 안전하게 절단한 뒤
길이를 쓰도록 공용 헬퍼 레벨에서 가드를 추가했다.

```rust
let mut utf16: Vec<u16> = s.encode_utf16().collect();
// 글자수 필드가 u16이므로 초과 시 자르지 않으면 wraparound 로 레코드가 손상된다.
if utf16.len() > u16::MAX as usize {
    utf16.truncate(u16::MAX as usize);
    if matches!(utf16.last(), Some(&u) if (0xD800..=0xDBFF).contains(&u)) {
        utf16.pop();
    }
}
self.write_u16(utf16.len() as u16)?;
```

## 테스트 (Red → Green)
- Red: 수정 전에는 `u16::MAX + 10` 길이 문자열을 `write_hwp_string()`에 쓰면 길이 필드가 0으로
  wraparound되어 `read_hwp_string()`으로 되읽었을 때 길이 필드(0)와 실제 바이트열이 어긋나 파싱이
  깨지거나 빈 문자열이 반환된다.
- Green: `test_write_hwp_string_overlong_truncates_instead_of_wrapping` — `u16::MAX + 10`개의
  `'A'` 문자열을 쓰고 되읽어, 길이 필드와 실제 UTF-16 코드유닛 수가 정확히 `u16::MAX`로 절단되어
  일치함을 검증.

```
cargo test --lib serializer::byte_writer
running 16 tests
...
test serializer::byte_writer::tests::test_write_hwp_string_overlong_truncates_instead_of_wrapping ... ok
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 2275 filtered out
```

`cargo check --lib` 및 `rustfmt --edition 2021 src/serializer/byte_writer.rs` 모두 정상 통과.

## 변경 파일
- `src/serializer/byte_writer.rs` (수정 + 테스트 1건 추가)

## 브랜치 / PR
- 브랜치: `task/m100-2945-byte-writer-u16-guard` (origin/devel 기준)
- 이슈: #2945
