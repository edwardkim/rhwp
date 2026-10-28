# bug-circled-ganada OOB 수정 — `src/renderer/mod.rs` CIRCLED/GANADA

## 개요

`src/renderer/mod.rs`의 `format_circled_digit` / `format_hangul_ganada` 가

```rust
CIRCLED[(n - 1) as usize]
GANADA[(n - 1) as usize]
```

형태로 인덱싱한다. `n: u16` 이 **0**일 때:

* debug 빌드: `0u16 - 1` 에서 overflow panic
* release 빌드: `65535 as usize` 인덱스로 OOB panic (또는 `usize::MAX` 수준의 잘못된 접근)

트리거는 HWP **자동번호 필드 손상** (`n == 0`). 정상 문서에서는 `n >= 1` 이므로 현재
`if n >= 1 && n <= 20 { CIRCLED[(n-1) as usize] }` 가드가 동작해 panic 을 피하지만,
가드는 수동 경계 검사에 의존하고 배열 크기와 동기화되지 않는다. 동일 패턴이
`GANADA[14]` 에도 존재한다.

## 수정

`checked_sub(1)` + `get()` + `unwrap_or_else` 로 방어적 경계 처리로 교체했다.

```rust
// before (1785, 1850 근처)
if n >= 1 && n <= 20 {
    CIRCLED[(n - 1) as usize].to_string()
} else {
    n.to_string()
}

// after
n.checked_sub(1)
    .and_then(|idx| CIRCLED.get(idx as usize))
    .map(|c| c.to_string())
    .unwrap_or_else(|| n.to_string())
```

`GANADA` 도 동일하게 `checked_sub(1).and_then(|idx| GANADA.get(idx as usize))` 로 교체.
`get()` 이 `None` 을 돌려주면 숫자 문자열로 fallback — 문서 손상 시에도 렌더러가
panic 없이 `"0"` 을 출력한다.

* 파일: `src/renderer/mod.rs:1843-1853` (`format_circled_digit`), `src/renderer/mod.rs:1908-1917` (`format_hangul_ganada`)
* 동작 변경 없음 — 정상 범위(①~⑳, 가~하)는 동일 문자, 비정상 범위(0, 21+, 15+ 등)는
  종전과 같은 `n.to_string()` fallback. 차이는 `n == 0` 에서도 안전하다는 점.

## 재현 및 검증

### 단위 검증 (format_number)

```
n=0  circled=0 ganada=0
n=1  circled=① ganada=가
n=14 circled=⑭ ganada=하
n=15 circled=⑮ ganada=15
n=20 circled=⑳ ganada=20
n=21 circled=21 ganada=21
n=65535 circled=65535 ganada=65535
all assertions passed
```

* `format_number(0, CircledDigit) == "0"` — OOB 없이 fallback
* `format_number(0, HangulGaNaDa) == "0"` — 동일
* 경계 `1, 14, 20, 21` 모두 기대값과 일치

### cargo 검증

```
cargo fmt --all -- --check   # 통과
cargo clippy --all-targets   # 통과 (warnings 0, Finished dev)
cargo test --lib -- test_format_number  # 5 passed
cargo test --lib renderer    # 95 passed
```

### 시각 검증 (before / after)

`rhwp export-svg` 로 `samples/basic/BookReview.hwp` 2쪽을 before(수정 전 바이너리:
`C:\Users\swsz9\rhwp\target\debug\rhwp.exe`) / after(수정 후: `rhwp-bug4` 빌드)로
각각 출력했다.

```
before/BookReview_001.svg 137,487 bytes
after/BookReview_001.svg  137,487 bytes
before/BookReview_002.svg 857,459 bytes
after/BookReview_002.svg  857,459 bytes
diff: 0 lines (동일)
```

정상 문서에서는 렌더 결과가 비트 단위로 동일함을 확인 — 방어적 수정이
기존 출력에 영향을 주지 않는다. `n == 0` 손상 문서는 before 에서 panic
가능성이 있었으나 after 에서는 `"0"` 으로 안전하게 렌더된다.

| 파일 | 설명 |
|---|---|
| `before.svg` | 수정 전 export (BookReview 1쪽) |
| `after.svg` | 수정 후 export (동일) |
| `before_p2.svg` / `after_p2.svg` | 2쪽 — 동일 |

## 범위

* 변경 파일: `src/renderer/mod.rs` 2 함수
* 영향: 자동번호 렌더링만 — 레이아웃·쪽수·폰트·표 등 무관
* 위험도: 낮음 — `get()` 기반 방어 코드, 기존 테스트 golden 유지
