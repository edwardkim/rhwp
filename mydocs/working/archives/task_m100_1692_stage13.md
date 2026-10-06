# Task #1692 Stage 13 - HWP3 22쪽 하드코딩 제거 및 회귀 정리

## 배경

- 직전 커밋: `3c608e899 task 1692: SO-SUEOP p22 미주 표지 위치 보정`
- 22쪽 HWP3 렌더 차이를 맞추는 과정에서 샘플 본문 문자열을 직접 비교하는
  `fixup_hwp3_so_sueop_p22_markers` 보정이 들어가 있었다.
- 샘플명/문구/페이지 기준 하드코딩은 금지되어야 하므로, HWP3 파서/렌더러의 일반 규칙으로
  대체하고 전체 회귀를 다시 확인했다.

## 계획

1. `samples/SO-SUEOP.hwp`, `samples/SO-SUEOP.hwpx`, `pdf/SO-SUEOP-2024.pdf`의 22쪽
   관계도/본문 흐름을 일반 구조 기준으로 유지한다.
2. HWP3 전용 본문 문자열 치환, 관계도 표 본문 재작성, 샘플명 주석을 제거한다.
3. HWP3 legacy 문자 매핑, 비-TAC 컨트롤 앵커 슬롯, 비어 있는 캡션 suppression 등
   포맷 구조 기반 보정으로 대체한다.
4. 기존 HWP3/HWP5 미주 회귀를 함께 돌려 일반 HWP5 문서가 깨지지 않는지 확인한다.

## 진행 기록

- `src/parser/hwp3/mod.rs`
  - `fixup_hwp3_so_sueop_p22_markers`, 관계도 표 본문 문자열 재작성,
    본문 문구 prefix 보정을 제거했다.
  - 캡션 문단은 실제 렌더 가능한 텍스트/컨트롤이 있을 때만 붙이도록 바꿨다.
  - HWP3 비-TAC 표와 미주/각주의 보이지 않는 8유닛 앵커 슬롯을 파서에서 보존하도록
    정리했다.
  - TAC 그림/도형 단독 자리 문단의 표시용 `FFFC`와 line spacing을 일반 조건으로 정리했다.
- `src/parser/hwp3/johab.rs`
  - HWP3 legacy 원문자, 따옴표, 관계도 선문자, 한 점 리더, 화살표를 일반 문자 매핑으로
    추가했다.
- `src/document_core/queries/rendering.rs`
  - HWP3-origin HWP5 변환본의 `spacing_before` flow 복원은 유지하되, 원본 HWP3에서
    재확대하지 않도록 주석과 조건을 정리했다.
- `src/renderer/typeset.rs`, `src/renderer/layout.rs`
  - bisect 결과 `1a8a88fe7`의 미주 구분선 가시성 판정 변경이 일반 HWP5
    `issue_1293_2024_no_separator_20mm` 회귀를 만든 것을 확인했다.
  - `separator_line_type || separator_line_width || separator_length` 기준으로 복원해
    기존 HWP5 미주 흐름 계약을 유지했다.
- `src/renderer/layout/paragraph_layout.rs`, `src/renderer/typeset.rs`
  - SO-SUEOP 샘플명을 직접 언급하던 주석을 HWP3 관계도/비가시 구분선 미주 일반 현상
    설명으로 바꿨다.

## 검증

- `cargo fmt`
- `git diff --check`
- `env CARGO_INCREMENTAL=0 cargo test -q --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 85 passed
- `env CARGO_INCREMENTAL=0 cargo test -q --test issue_1692 -- --nocapture`
  - 10 passed
- `env CARGO_INCREMENTAL=0 cargo test -q --test issue_1116 -- --nocapture`
  - 13 passed
- `env CARGO_INCREMENTAL=0 cargo test --all-targets`
  - 통과
  - `real 2975.19`, `user 1678.09`, `sys 47.50`

## 메모

- `src/` 기준 `SO-SUEOP`, `fixup_hwp3_so_sueop`, 샘플 본문 문자열 검색 결과가 남지 않는다.
- repo `tmp/` 아래에 이번 단계에서 만든 1692 관련 임시 파일은 남지 않았다.
