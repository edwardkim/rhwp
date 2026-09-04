# 구현계획서 — Task M100-1271: HWPX 글뒤로 표 분할과 바탕쪽 홀짝 보정

## 설계 요약

대상 증상은 두 층으로 분리한다.

```text
1. 페이지네이션 오프셋
   section0 pi=3 의 paper-anchored BEHIND_TEXT RowBreak 표가
   TypesetEngine 에서 PartialTable 로 분할되어 PDF에 없는 1쪽을 만든다.

2. masterpage 선택 순서
   active_master_page 선택이 구역 간 page_number carry 보정 전에 수행되어,
   최종 쪽번호 기준 Even/Odd 재선택이 보장되지 않는다.
```

이번 구현은 1번을 우선 해결하고, 2번은 같은 이슈 안에서 구조적 회귀 방지 보정으로 처리한다.
단, HWP5 raw parser의 바탕쪽 순서 규칙과 HWPX masterpage `idRef` 연결 로직은 변경하지 않는다.

## Stage 1 — 회귀 재현 테스트와 진단 고정

**목표**: 대상 결함을 재현 가능한 테스트와 dump 기준으로 고정한다.

변경 후보:

- `tests/issue_1271_hwpx_behind_text_table.rs` 또는 `src/renderer/typeset.rs` 테스트 모듈
  - 대상 샘플 기반 smoke test:
    - `samples/hwpx/[2027] 온새미로 1 본교재.hwpx` 로드
    - 기본 TypesetEngine 페이지 수와 앞 4쪽 배치 확인
  - synthetic unit test:
    - `TextWrap::BehindText`
    - `VertRelTo::Paper`
    - `treat_as_char=false`
    - `row_count > 1`
    - `page_break=RowBreak`
    - 본문 가용 높이보다 큰 measured height
    - 기대: 후속 명시 쪽나누기 문단이 PDF 대응처럼 다음 페이지로 오고, 중간 `PartialTable` 페이지가 생기지 않음

진단 명령:

```text
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 1
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 2
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 3
env RHWP_USE_PAGINATOR=1 cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 1
```

주의:

- 테스트가 너무 무거우면 sample 전체 페이지 수 테스트는 `#[ignore]` 또는 문서화된 smoke로 두고, synthetic unit test를 필수 회귀 테스트로 둔다.
- `pdf-large` PDF는 테스트 입력으로 직접 쓰지 않는다. PDF는 시각 기준과 수동 검증 자료로만 사용한다.

보고서:

- `mydocs/working/task_m100_1271_stage1.md`

## Stage 2 — TypesetEngine paper-anchored BehindText 표 out-of-flow 보정

**목표**: paper-anchored 배경성 `BehindText`/`InFrontOfText` 표가 본문 흐름과 페이지 분할을 밀지 않게 한다.

변경 후보:

- `src/renderer/typeset.rs`
  - 현재 `oversized_multirow` 조건은 다행 표가 본문보다 크면 out-of-flow shortcut에서 제외한다.
  - 이 조건을 보강해 다음 조건을 만족하는 표는 `PageItem::Shape` 경로로 유지한다.

후보 조건:

```text
!table.common.treat_as_char
text_wrap in {BehindText, InFrontOfText}
st.col_count == 1
vert_rel_to == Paper
horz_rel_to == Paper 또는 Absolute/Paper 계열 검토
```

설계 의도:

- paper-anchored `BehindText`/`InFrontOfText`는 본문 흐름의 높이 계산 대상이 아니라 페이지 위 절대 배치 개체로 본다.
- `TopAndBottom` 표, para-relative flow 표, 다단 분배가 필요한 표는 기존 `typeset_block_table` 경로를 유지한다.
- #992에서 의도한 “본문보다 큰 다행 표는 분할 필요” 정책은 paper-anchored 배경성 표를 제외한 나머지에 유지한다.

검증:

```text
cargo test --lib typeset_703
cargo test --lib issue_1271
cargo test --lib typeset
```

필요 시 명령은 실제 테스트명에 맞춰 조정한다.

보고서:

- `mydocs/working/task_m100_1271_stage2.md`

## Stage 3 — 최종 쪽번호 기준 masterpage 선택 보정

**목표**: 구역 간 쪽번호 carry 보정 이후에도 `active_master_page`가 최종 쪽번호의 홀짝 기준으로 선택되게 한다.

변경 후보:

- `src/document_core/queries/rendering.rs`
  - masterpage 선택 블록을 helper로 분리한다.
  - helper 입력:

```rust
fn assign_master_pages_for_section(
    result: &mut PaginationResult,
    section_index: usize,
    section: &Section,
)
```

  - 기존 동작 보존:
    - 기본 `Odd`/`Even`/`Both` 선택
    - `hide_master_page` 첫 쪽 감추기
    - 마지막 쪽 확장 바탕쪽 처리
    - overlap/replace_base 처리
  - 호출 시점:
    - 구역 간 `page_number` carry 보정 이후 최종 쪽번호 기준으로 한 번 호출
    - 기존 선행 호출은 제거하거나, carry 후 재호출로 덮어써 중복 상태를 피한다.

주의:

- `is_first_page`와 `is_last`는 구역 내부 페이지 인덱스 기준을 유지한다.
- `is_odd`만 최종 `page.page_number` 기준이어야 한다.
- extra master pages는 재호출 전에 clear하여 중복 push를 막는다.

테스트 후보:

- 작은 synthetic document:
  - section0이 3쪽 또는 4쪽으로 끝나는 케이스
  - section1에 Even/Odd masterpage가 있고 NewNumber가 없음
  - section1 첫 페이지 active masterpage가 carry 후 최종 쪽번호 홀짝과 일치하는지 확인

보고서:

- `mydocs/working/task_m100_1271_stage3.md`

## Stage 4 — 대상 샘플 구조/시각 검증

**목표**: 실제 #1271 샘플에서 PDF와 같은 앞부분 페이지 대응과 바탕쪽 홀짝을 확인한다.

검증 명령:

```text
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx"
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 1
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 2
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 3
```

확인 항목:

```text
1. 총 페이지 수: 46쪽
2. 2쪽: MEMO
3. 3쪽: 1주차 표지
4. 4쪽: section1 본문 시작, page_num=4
5. 5쪽 이후에도 PDF와 페이지 대응이 1쪽 밀리지 않음
```

시각 검증 후보:

```text
cargo run --quiet --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1271 --debug-overlay -p 1
cargo run --quiet --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1271 --debug-overlay -p 2
cargo run --quiet --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1271 --debug-overlay -p 3
cargo run --quiet --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -o output/poc/task1271 --debug-overlay -p 4
```

PDF 참고:

```text
pdfinfo "pdf-large/hwpx/[2027] 온새미로 1 본교재.pdf"
```

보고서:

- `mydocs/working/task_m100_1271_stage4.md`

## Stage 5 — 회귀 검증과 최종 보고

**목표**: 대상 수정이 기존 표 조판, masterpage, HWPX 파서 회귀를 만들지 않는지 확인한다.

필수 검증:

```text
cargo fmt --check
cargo test --lib
cargo test --tests
```

집중 검증:

```text
cargo test --lib typeset
cargo test --lib hwpx
cargo test --test issue_1197_svg_object_zorder
```

필요 시 확장 검증:

```text
cargo clippy -- -D warnings
```

최종 산출물:

- `mydocs/working/task_m100_1271_stage5.md`
- `mydocs/report/task_m100_1271_report.md`
- `mydocs/orders/20260603.md` 상태 갱신

## 제외 범위

- HWPX masterpage `idRef` 연결 로직 재작성
- HWP5 raw 바탕쪽 순서 규칙 변경
- 모든 `BehindText` 다행 표의 일괄 out-of-flow 처리
- 다단 문서의 `BehindText`/`InFrontOfText` 표 분배 정책 변경
- PDF 파일을 자동 테스트 fixture로 직접 사용하는 검증

## 완료 판정

완료 조건:

```text
1. 대상 HWPX의 기본 TypesetEngine 페이지 수가 46쪽이다.
2. section0 앞부분 페이지 대응이 PDF와 일치한다.
3. section1 본문 시작이 page_num=4로 잡힌다.
4. masterpage Even/Odd 선택이 최종 쪽번호 기준으로 정합하다.
5. 기존 #703/#775/#992 계열 표 조판 회귀가 없다.
6. 단계별 완료 보고서와 최종 보고서가 작성된다.
```

## 작업지시자 승인 요청

본 구현계획이 승인되면 Stage 1부터 소스 수정과 회귀 테스트 작성을 시작한다.
첫 수정 범위는 `src/renderer/typeset.rs` 및 #1271 회귀 테스트로 제한한다.
