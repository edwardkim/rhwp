# Task #576 구현 계획서 — 안 A 상세화

- **이슈**: [#576](https://github.com/edwardkim/rhwp/issues/576)
- **브랜치**: `local/task576`
- **단계**: Stage 2 (구현 계획)
- **선행 산출**: `mydocs/working/task_m100_576_stage1.md` (정밀 진단 + 영향 범위)
- **작성일**: 2026-05-04

## 1. 정정 본질 (Stage 1 결론 재진술)

`tokenizer.rs::read_command` (L104) 의 prefix-split 키워드 list 가 폰트 스타일 모디파이어 3 개 (`bold/it/rm`) 만 처리. 광범위 sweep 결과 **`times` / `sim` / `TIMES` / `SIM` 4 개 키워드** 가 변수와 결합 시 단일 식별자로 토큰화되는 결함 발현. `exam_science.hwp` 한정 (158 fixture 중).

## 2. 정정 안 — 안 A 상세

### 2.1 변경 위치 — `src/renderer/equation/tokenizer.rs` L104

**현재**:
```rust
for kw in ["bold", "it", "rm"] {
```

**변경 후**:
```rust
// [Task #576] times/sim 키워드도 변수와 인접 시 분리.
// HWP 수식 script 에서 "a timesm" → "a × m", "rm X simZ" → "rm X ~ Z" 로 의미됨.
// 광범위 sweep (158 fixture / 563 unique scripts) 결과 결함 발현 키워드는
// times/sim 만 (대소문자 4 개). alpha/over/sqrt 등 다른 keyword 는 항상
// 공백 구분되어 prefix-split 불필요. 그리스 문자 prefix 충돌
// (예: "alphabet" → "alpha"+"bet") 회귀 위험 0.
for kw in ["bold", "it", "rm", "times", "sim", "TIMES", "SIM"] {
```

### 2.2 핵심 설계 결정

| 항목 | 결정 | 근거 |
|------|------|------|
| 키워드 추가 범위 | sweep 발견 4 키워드만 | 회귀 위험 0 보장 (다른 키워드는 sweep 에서 결함 미발현) |
| 대소문자 처리 | 소문자 / 대문자 별도 등록 (`times`, `TIMES`) | 현재 `matches_at` 가 case-sensitive. lookup_symbol 의 case-insensitive 매칭과 일관성은 추후 안 B 도입 시 검토 |
| `Times` (혼합 케이스) | 미처리 | sweep 미발견. 발견 시 추가 |
| 위치 | 기존 list 확장 (in-place) | 변경 면적 최소 |

### 2.3 변경 LOC

`src/renderer/equation/tokenizer.rs`: **+5 / -1** (주석 +4, 코드 +1 keyword 추가)

## 3. Stage 3 회귀 차단 테스트 추가

### 3.1 신규 unit tests (`tokenizer.rs::tests` 모듈)

```rust
#[test]
fn test_task576_times_prefix_split() {
    let tokens = tokenize("a timesm");
    let values: Vec<&str> = tokens.iter()
        .filter(|t| !t.value.is_empty())
        .map(|t| t.value.as_str()).collect();
    assert_eq!(values, vec!["a", "times", "m"]);
}

#[test]
fn test_task576_sim_prefix_split() {
    let tokens = tokenize("rm X simZ");
    let values: Vec<&str> = tokens.iter()
        .filter(|t| !t.value.is_empty())
        .map(|t| t.value.as_str()).collect();
    assert_eq!(values, vec!["rm", "X", "sim", "Z"]);
}

#[test]
fn test_task576_uppercase_times_split() {
    let tokens = tokenize("1TIMES10");
    let values: Vec<&str> = tokens.iter()
        .filter(|t| !t.value.is_empty())
        .map(|t| t.value.as_str()).collect();
    assert_eq!(values, vec!["1", "TIMES", "10"]);
}

#[test]
fn test_task576_uppercase_sim_split() {
    let tokens = tokenize("rmA SIMC");
    let values: Vec<&str> = tokens.iter()
        .filter(|t| !t.value.is_empty())
        .map(|t| t.value.as_str()).collect();
    assert_eq!(values, vec!["rm", "A", "SIM", "C"]);
}

#[test]
fn test_task576_alpha_no_split() {
    // 회귀 차단: alpha/alphabet 는 분리되지 않아야 함
    let tokens = tokenize("alpha");
    let values: Vec<&str> = tokens.iter()
        .filter(|t| !t.value.is_empty())
        .map(|t| t.value.as_str()).collect();
    assert_eq!(values, vec!["alpha"]);

    let tokens = tokenize("alphabet");
    let values: Vec<&str> = tokens.iter()
        .filter(|t| !t.value.is_empty())
        .map(|t| t.value.as_str()).collect();
    assert_eq!(values, vec!["alphabet"]);
}

#[test]
fn test_task576_keyword_followed_by_space() {
    // times/sim 다음에 공백/기호 오면 분리 불필요 (기존 동작 보존)
    let tokens = tokenize("a times b");
    let values: Vec<&str> = tokens.iter()
        .filter(|t| !t.value.is_empty())
        .map(|t| t.value.as_str()).collect();
    assert_eq!(values, vec!["a", "times", "b"]);
}
```

### 3.2 광범위 sweep
- baseline (변경 전) SVG 생성 — 158 fixture
- after (변경 후) SVG 생성
- diff: exam_science 4 페이지 + 다른 fixture byte-identical

### 3.3 자동 테스트
- `cargo test --lib` — 1125+ 통과 (신규 6 tests 추가 후 1131+)
- `cargo clippy --release --lib` — 신규 경고 0
- `cargo test --test svg_snapshot` — 6/6

## 4. 위험 요소 및 완화

| 위험 | 완화책 |
|------|-------|
| 미발견 결합 패턴 (다른 keyword) | sweep 158 fixture 에서 결함 미발현 검증. 향후 발견 시 list 추가 (안 A 재실행) 또는 안 B 마이그레이션 |
| 대소문자 변형 (`Times`, `tImes`) | sweep 미발견. 향후 발견 시 list 추가 |
| `lookup_symbol` 등 다른 코드 영역과 동기화 | 안 A 는 토크나이저만 수정. lookup_symbol 의 case-insensitive 매칭은 분리된 토큰을 받아 처리하므로 영향 없음 |
| `feedback_rule_not_heuristic` 정합 | ✓ 안 A 는 명시적 키워드 list (룰), 휴리스틱 아님 |
| `feedback_essential_fix_regression_risk` 정합 | ✓ 광범위 sweep + tokenizer unit tests + svg_snapshot |

## 5. Stage 3 실행 절차

1. baseline SVG 생성 (158 fixture sweep)
2. `tokenizer.rs::read_command` L104 키워드 list 확장
3. 신규 6 unit tests 추가
4. `cargo build --release` — 빌드 통과
5. `cargo test --lib` — 1131 통과 확인
6. exam_science.hwp page 4 pi=128 SVG 측정 — 수식 콘텐츠 정상 렌더 확인
7. 광범위 sweep — exam_science 외 byte-identical 확인
8. `cargo clippy`, `cargo test --test svg_snapshot`
9. Stage 3 보고서 작성

## 6. 산출물 (Stage 3)

- `mydocs/working/task_m100_576_stage3.md` — 구현 + 검증 결과
- 코드 diff: `src/renderer/equation/tokenizer.rs`

## 7. 승인 요청

본 구현 계획대로 Stage 3 (구현 + 검증, 안 A) 진입을 승인 요청합니다.
