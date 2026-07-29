# Task m100-3010 처리 결과 — WMF Region scan_count 음수 부호확장 버그 수정

## 이슈

- 이슈: #3004 "WMF Region 파싱 — scan_count(i16)가 음수일 때 Vec::with_capacity에 부호확장되어 거대 할당/패닉 유발"
- 파일: `src/wmf/parser/objects/graphics/region.rs`

## 문제

`Region::parse`는 `ScanCount` 필드를 `i16`(부호 있는 16비트 정수)로 읽은 뒤, 검증 없이 그대로
`scan_count as usize`로 캐스팅해 `Vec::with_capacity`에 넘긴다.

```rust
pub scan_count: i16,
...
let mut a_scans = Vec::with_capacity(scan_count as usize);

for _ in 0..scan_count {
    let (v, c) = crate::wmf::parser::Scan::parse(buf)?;
    ...
}
```

WMF 명세상 스캔라인 개수는 음수일 수 없지만 파서가 이를 검증하지 않는다. Rust에서 음수 `i16`을
`usize`로 캐스팅하면 부호 확장(sign-extension)이 일어난다. 예를 들어 `scan_count == -1`
(`0xFFFF`)이면 64비트 환경에서 `usize::MAX`(`0xFFFFFFFFFFFFFFFF`)가 된다.
`Vec::with_capacity(usize::MAX 근처 값)`은 즉시 "capacity overflow" 패닉을 일으킨다.
즉, 조작된 WMF 파일 하나로 이 라이브러리를 사용하는 애플리케이션 전체를 크래시시킬 수 있는
서비스 거부(DoS) 취약점이다.

## 기존 유사 사례와의 관계 (중복 아님)

같은 "파일에서 읽은 미검증 개수값을 그대로 `Vec::with_capacity`에 넘기는" 클래스의 버그가
최근 다음과 같이 발견·수정되었다.

- EMF `parse_points16`의 `count`(u32) 미검증 — #2992 / PR #2998
- WMF DIB `Colors::parse_from_color_usage`의 `colors_used`(u32) 미검증 — #3000 / PR #3002

이번 건은 `u16`/`u32` 값이 양수 범위를 넘어서는 경우가 아니라, **부호 있는 `i16` 필드를
부호 없는 길이로 오용**하면서 음수 값의 부호확장이 트리거가 된다는 점에서 근본 원인이
다르다. 이슈/PR 검색(`gh issue list --search "scan_count"`,
`gh issue list --search "region.rs"`, `gh pr list --state open --author kevin9327
--search "region"`) 결과 중복 없음을 확인했다.

## 수정

`scan_count`가 음수이면 `Vec::with_capacity` 호출 전에 `ParseError::UnexpectedPattern`을
반환하도록 방어 코드를 추가했다.

```rust
if scan_count < 0 {
    return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
        cause: format!("The scan_count field `{scan_count}` must not be negative"),
    });
}
let mut a_scans = Vec::with_capacity(scan_count as usize);
```

## 테스트 (red → green)

`scan_count = -1`로 설정한 최소 Region 바이트열을 파싱했을 때, 수정 전에는
`Vec::with_capacity`에서 capacity overflow 패닉이 발생(red)했을 것이나, 수정 후에는
`ParseError`로 안전하게 실패(green)하는지 확인하는 유닛 테스트를 추가했다.

```
test wmf::parser::objects::graphics::region::tests::negative_scan_count_returns_error_instead_of_panicking ... ok
```

## 검증

- `cargo check --lib` 통과
- `cargo test --lib negative_scan_count` — 1 passed
- `rustfmt --edition 2021 src/wmf/parser/objects/graphics/region.rs` 적용
