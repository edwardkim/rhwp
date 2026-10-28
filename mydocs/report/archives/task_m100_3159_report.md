# Task M100 #3159 구현 보고서

## 목표

WMF 레코드 파서 `RecordSize`의 검증되지 않은 정수 산술 두 곳을 제거해, 적대적/잘린 WMF 바이트에서
발생하는 패닉(디버그) 및 거대 할당·OOM(릴리스)을 방어한다. 이 경로는 HWP 문서에 임베드된 WMF
그림을 SVG로 변환할 때 신뢰할 수 없는 입력에 대해 실행된다.

## 원인

`src/wmf/parser/records/mod.rs`의 `RecordSize`:

1. **`byte_count()` 곱셈 오버플로**: `(self.0 * 2) as usize`. `self.0`(WORD 개수)은 u32이며,
   `self.0 * 2`가 u32 산술로 계산된다. `self.0 >= 0x8000_0000`이면 오버플로 → 디버그 빌드
   `attempt to multiply with overflow` 패닉.

2. **`remaining_bytes()` 뺄셈 언더플로**: `self.byte_count() - self.1`. 레코드가 자신의 크기를
   실제 고정 필드보다 작게 선언하면, 레코드 파서가 고정 필드를 읽어 소비 바이트 `self.1`이
   `byte_count()`를 초과한다. 이때 뺄셈이 usize 언더플로 → 디버그 빌드 패닉, 릴리스 빌드에서는
   거대한 값이 `consume_remaining_bytes` → `read_variable`의 `Vec::with_capacity`로 전달되어
   할당 패닉/OOM.

`renderer::svg::convert_wmf_to_svg`는 `WMFConverter::run().ok()`로 호출하는데, `.ok()`는 `Result`
오류만 삼키고 패닉은 잡지 못하므로 문서 렌더링 전체가 중단된다.

## 재현 경로

`META_LINETO`는 `record_function`(2) + `y`(2) + `x`(2) = 6바이트를 소비한다. 레코드가
`RecordSize = 2`(byte_count = 4)로 선언되면 소비(6) > 선언(4) → `remaining_bytes()` 언더플로.
`converter/mod.rs`의 레코드 루프는 `byte_count() == 0`만 걸러내고 최소 헤더 크기는 검증하지 않는다.

## 변경

`src/wmf/parser/records/mod.rs`:

```rust
pub fn byte_count(&self) -> usize {
    self.0 as usize * 2               // usize 산술로 승격해 곱셈 오버플로 제거
}

pub fn remaining_bytes(&self) -> usize {
    self.byte_count().saturating_sub(self.1)  // 소비 초과 시 0으로 포화
}
```

`saturating_sub`으로 소비가 선언 크기를 초과하면 "남은 바이트 없음"으로 안전 처리한다. 이후
루프는 다음 레코드에서 정렬 불일치로 정상 `ParseError` 후 종료되며 무한 루프는 발생하지 않는다.

## 검증

red → green (디버그 빌드, `RUSTFLAGS="-C linker=rust-lld"`):

| 테스트 | 수정 전 | 수정 후 |
|---|---|---|
| `remaining_bytes_does_not_underflow_when_consumed_exceeds_declared_size` | FAIL (`subtract with overflow` @ :107) | PASS |
| `byte_count_does_not_overflow_on_large_word_size` | FAIL (`multiply with overflow` @ :87) | PASS |
| 주변 `wmf` 테스트(`negative_scan_count_returns_error_instead_of_panicking` 포함) | PASS | PASS |

`rustfmt --check`(변경 파일 코드 내용): PASS. (저장소 전반의 CRLF newline 경고는 기존 상태이며
이번 변경과 무관.)

## 관련

동일 방어 클래스의 기존 이슈(#3000 colors_used, #3004 scan_count, #3012·#3156 point count)와
취지는 같으나 위치가 다른 `RecordSize` 산술 결함이다.
