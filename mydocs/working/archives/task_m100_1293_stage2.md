# Task 1293 Stage 2: 미주 모양 정규화 접근자 구현

## 목적

한컴 공식 미주 모양 항목을 rhwp 내부에서 명확한 의미로 접근하도록 정규화한다.

## 작업 범위

1. HWP5/HWPX 파서의 실제 저장 슬롯과 한컴 UI 의미를 재확인한다.
2. `FootnoteShape`에 공식 의미 접근자를 추가한다.
3. 기존 타입셋/렌더/각주 높이 계산에서 직접 필드 접근을 줄이고 접근자를 사용한다.
4. HWPX `aboveLine`/`belowLine`/`betweenNotes` 매핑 테스트를 공식 의미 기준으로 갱신한다.

## 구현 전 판단

기존 `FootnoteShape` 필드는 라운드트립을 위해 유지한다. 다만 렌더링 로직은 아래 공식
의미를 반환하는 접근자를 사용해야 한다.

- `separator_above_margin_hu`: 본문과 구분선 사이
- `separator_below_margin_hu`: 구분선과 미주/각주 내용 사이
- `between_notes_margin_hu`: 앞 번호 주석 내용과 다음 번호 주석 내용 사이

## 검증 예정

- `cargo test --test issue_1050_footnote_serialize`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1139_exam_2022_endnote_shape_matches_hancom_reference -- --nocapture`
- `cargo test --lib compact_endnote -- --nocapture`

## 구현 내용

### 1. `FootnoteShape` 공식 의미 접근자 추가

`src/model/footnote.rs`에 한컴 UI 의미를 직접 드러내는 접근자를 추가했다.

- `separator_above_margin_hu()`: 공식 `구분선 위`
- `separator_below_margin_hu()`: 공식 `구분선 아래`
- `between_notes_margin_hu()`: 공식 `각주/미주 사이`

기존 원본 슬롯은 라운드트립 보존을 위해 유지했다. 렌더러/타입셋은 이 슬롯을 직접
해석하지 않고 접근자를 사용하도록 바꾸는 방향으로 정리했다.

### 2. HWPX `noteSpacing` 공식 의미 매핑

`src/parser/hwpx/section.rs`의 `<hp:noteSpacing>` 파싱을 공식 의미 기준으로 정정했다.

- `betweenNotes` → `between_notes_margin_hu()`
- `belowLine` → `separator_below_margin_hu()`
- `aboveLine` → `separator_above_margin_hu()`

`aboveLine`이 실제로 존재할 때는 더 이상 `separator_margin_top` sentinel fallback을 덮어쓰지 않는다.
일부 오래된 HWPX에서 `aboveLine`이 없을 가능성만 기존 fallback으로 유지했다.

### 3. 공통 렌더/높이 계산 경로 정리

아래 경로에서 직접 슬롯 접근을 공식 의미 접근자로 교체했다.

- `src/document_core/commands/object_ops.rs`
  - `get_endnote_shape_native()`가 JSON으로 반환하는 `separatorMarginTop`,
    `separatorMarginBottom`, `noteSpacing`을 공식 의미 기준으로 반환한다.
- `src/renderer/typeset.rs`
  - 미주 구분선 item의 `margin_above`, `margin_below`, 미주 사이 값 계산을 접근자로 통일했다.
- `src/renderer/layout/picture_footnote.rs`
  - 각주 영역 높이 추정과 렌더링에서 `구분선 위/아래`, `각주 사이`를 접근자로 통일했다.
- `src/renderer/height_measurer.rs`
  - 각주 높이 측정도 동일 접근자를 사용한다.

### 4. 테스트 기준 갱신

기존 테스트가 원본 슬롯을 한컴 UI 의미처럼 직접 검증하던 부분을 정규화 접근자 검증으로
바꿨다. 원본 슬롯 자체가 필요한 HWPX contract 테스트에는 원본 슬롯과 접근자 값을 함께
확인하도록 했다.

## 검증 결과

- `cargo fmt --all -- --check` 통과
- `cargo test --test issue_1050_footnote_serialize -- --nocapture`
  - 7 passed
- `cargo test --lib parse_endnote -- --nocapture`
  - 3 passed
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 51 passed
- `cargo test --lib compact_endnote -- --nocapture`
  - 26 passed

## 남은 판단

이 단계는 공식 의미 접근자와 HWPX `aboveLine` 매핑을 바로잡는 기초 작업이다.
아직 `구분선위20` 샘플의 전체 visual sweep과 PR #1292 보정 커밋 재평가는 수행하지 않았다.
다음 단계에서는 `구분선위20`, `구분선아래20`, `미주사이20` 샘플의 실제 separator/내용 간격을
정량화하고, 기존 #1284 보정 중 공식 모델로 대체 가능한 부분을 분리해야 한다.
