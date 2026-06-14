# PR #1366 검토 — hwp2hwpx CLI + 무손실 검증 게이트

- PR: https://github.com/edwardkim/rhwp/pull/1366
- 제목: feat: hwp2hwpx CLI with lossless verification gates + serializer fidelity fixes
- 작성일: 2026-06-11
- 작성자: `idaeho`
- 관련 이슈: #1365 "HWP→HWPX 변환 무손실 게이트: hwp2hwpx CLI + 직렬화 정합 수정 묶음"
- base: `devel` (`430d5edc`)
- head: `idaeho:port/hwp2hwpx-lossless` (`33593a3c`)
- 로컬 검토 브랜치: `local/pr1366-upstream`

## 1. 요약 판단

**수정 요청**을 권고한다.

방향성은 좋다. `rhwp hwp2hwpx` CLI와 `--verify`, `--verify-pages` 게이트는 프로젝트에 필요한 기능이고,
작은 샘플 기준 smoke test도 통과했다. 다만 현재 PR은 최신 `devel`에 바로 수용할 수 없다.

필수 수정 사항:

1. 최신 `devel`과 merge conflict 해소
2. `cargo fmt --check` 실패 해소
3. #1350/#1351에서 고친 HWPX `useFontSpace` 파싱 회귀 복구

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | open |
| draft | false |
| mergeable | `CONFLICTING` |
| 변경량 | 6 files, +755 / -102 |
| 작성자 | `idaeho` |
| 관련 이슈 | #1365 |
| GitHub checks | 없음 |

커밋:

- `96bc735d` — hwp2hwpx CLI + serializer fidelity fixes
- `33593a3c` — ir-diff semantic comparison upgrades

변경 파일:

- `src/main.rs`
- `src/parser/hwpx/header.rs`
- `src/parser/hwpx/section.rs`
- `src/serializer/hwpx/context.rs`
- `src/serializer/hwpx/section.rs`
- `src/serializer/hwpx/table.rs`

## 3. 주요 발견 사항

### 3.1 Blocker — 최신 devel과 충돌

GitHub가 `mergeable=CONFLICTING`으로 표시한다.

로컬에서도 `git merge-tree 3d4c454e9983c03caad394c7cc4d4e9dfa0bd158 origin/devel HEAD`로 충돌을
확인했다.

충돌 지점:

- `src/serializer/hwpx/section.rs`
- 최신 `devel`의 #1364 `Control::Form` 직렬화 arm
- PR #1366의 `Control::SectionDef(_) | Control::ColumnDef(_)` 슬롯 소비 arm

현재 PR 브랜치 기준 `render_control_slot()`에는 `SectionDef/ColumnDef` arm이 추가되어 있다.
반면 최신 `origin/devel`에는 같은 위치에 `Control::Form(form)` writer가 들어 있다.
수용하려면 두 arm을 모두 보존하도록 rebase 또는 conflict resolution이 필요하다.

### 3.2 Blocker — HWPX useFontSpace 파싱 회귀

`src/parser/hwpx/header.rs:524`에서 `useFontSpace`를 파싱하지 않고 무시하도록 바뀐다.

현재 PR 코드:

```rust
b"useFontSpace" | b"useKerning" | b"symMark" => {}
```

이 변경은 #1350/#1351에서 반영한 HWPX `useFontSpace` 보존 수정과 충돌한다. 최신 `devel`은
`useFontSpace`를 `cs.use_font_space = parse_bool(&attr)`로 보존한다.

영향:

- HWPX → IR 경로에서 `useFontSpace="1"`이 다시 `false`로 유실된다.
- HWPX → IR → HWPX, HWPX → IR → HWP5 roundtrip 보존성이 깨진다.
- 이번 PR의 `char_shape_semantic_eq()`도 `use_font_space`를 비교하지 않아 `--verify` 게이트가 이 회귀를
  잡지 못할 가능성이 있다.

필수 조치:

- `useFontSpace` 파싱 복구
- HWPX parser 단위 테스트 추가
  - `useFontSpace="1"` 입력 → `CharShape.use_font_space == true`
  - 가능하면 `useFontSpace="0"`도 함께 검증

### 3.3 Blocker — cargo fmt 실패

`cargo fmt --check`가 실패한다.

대표 파일:

- `src/serializer/hwpx/context.rs`
- `src/serializer/hwpx/section.rs`
- `src/main.rs`

로직 판단과 별개로, 현재 PR head는 포맷 게이트를 통과하지 못한다.

### 3.4 Major — ir-diff 의미론 완화 범위가 넓음

`src/main.rs`에 `char_shape_semantic_eq()`가 추가되며 raw char shape id 대신 의미 비교를 수행한다.
방향은 타당하지만 비교 항목에서 `use_font_space` 같은 보존 대상 metadata가 빠져 있다.

또한 `ir-diff`가 다음 차이를 의미론적으로 제외한다.

- tac=true 객체의 `text_wrap`
- HWP5 tab vendor padding code unit 3~5
- HWP3 variant pair의 line segment
- HWP3 variant pair의 ParaShape indent/spacing 2배 차이
- HWP3 variant pair의 TabDef negative-range sentinel
- 단독 `char_count` 차이

각 항목은 PR 설명상 실측 근거가 있지만, 검증 게이트(`hwp2hwpx --verify`)가 이 완화된 `ir-diff`에
의존하므로 회귀 은폐 가능성을 줄이는 테스트가 더 필요하다.

권장:

- `ir-diff` 의미론 완화는 항목별 fixture 또는 최소 단위 테스트를 추가
- `use_font_space`처럼 명시적으로 보존해야 하는 metadata는 semantic compare에도 포함
- `--verify`가 통과해도 숨길 수 있는 차이 목록을 문서화

## 4. 긍정 평가

- `rhwp hwp2hwpx <in.hwp> <out.hwpx> [--verify] [--verify-pages]` UX는 명확하다.
- `--verify` exit 3, `--verify-pages` exit 4는 자동화 게이트로 쓰기 좋다.
- `samples/equation-lim.hwp` 기준 기본 변환, `--verify`, `--verify-pages` smoke test는 모두 통과했다.
- BinData manifest key를 DocInfo BIN_DATA 1-based index로 맞추는 방향은 HWP5 그림 참조 모델과 부합한다.
- 표 `pageBreak` serializer를 parser 역매핑과 맞춘 것은 타당하다.
- page margin, colPr, notePr를 IR 값으로 쓰려는 방향도 페이지 수 정합 문제를 줄이는 데 맞다.

## 5. 로컬 검증

검토 브랜치: `local/pr1366-upstream`

| 명령 | 결과 |
|---|---|
| `git diff --check origin/devel...HEAD` | 통과 |
| `cargo fmt --check` | 실패 |
| `CARGO_INCREMENTAL=0 cargo check --bin rhwp` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib serializer::hwpx -- --nocapture` | 통과, 89 passed |
| `CARGO_INCREMENTAL=0 cargo test --lib parser::hwpx::header -- --nocapture` | 통과, 22 passed |
| `CARGO_INCREMENTAL=0 cargo test --lib test_parse_char_shape_use_font_space -- --nocapture` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib write_char_pr_use_font_space_roundtrip -- --nocapture` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib test_serialize_char_shape_use_font_space_bit -- --nocapture` | 통과 |

CLI smoke:

| 명령 | 결과 |
|---|---|
| `cargo run --bin rhwp -- hwp2hwpx samples/equation-lim.hwp /private/tmp/pr1366-equation-lim.hwpx` | 통과 |
| `cargo run --bin rhwp -- hwp2hwpx samples/equation-lim.hwp /private/tmp/pr1366-equation-lim-verify.hwpx --verify` | 통과, IR diff 0 |
| `cargo run --bin rhwp -- hwp2hwpx samples/equation-lim.hwp /private/tmp/pr1366-equation-lim-pages.hwpx --verify-pages` | 통과, 1 page |

주의: 기존 useFontSpace 테스트는 HWP5 parser와 HWPX serializer를 덮지만, HWPX parser의
`useFontSpace="1"` 파싱 회귀를 직접 잡지 못한다.

## 6. 권장 조치

작성자에게 다음 수정 요청을 권고한다.

1. 최신 `devel`로 rebase 또는 merge하여 conflict 해소
2. `Control::Form` writer와 `SectionDef/ColumnDef` 슬롯 소비 arm 모두 보존
3. `cargo fmt` 적용
4. HWPX `useFontSpace` 파싱 복구
5. HWPX `useFontSpace` parser 회귀 테스트 추가
6. `char_shape_semantic_eq()`에 `use_font_space` 비교 포함
7. 가능하면 `ir-diff` 의미론 완화 항목별 fixture/테스트 추가

## 7. 후속 처리

작업지시자 지시에 따라 컨트리뷰터에게 정중한 수정 요청 코멘트를 등록했다.

- 코멘트: https://github.com/edwardkim/rhwp/pull/1366#issuecomment-4675816234
- 내용:
  - 첫 기여 감사
  - 기능 방향성 긍정 평가
  - 최신 `devel` 기준 재작업/재요청 권고
  - conflict, `cargo fmt`, `useFontSpace` 회귀, `ir-diff` 의미론 비교 보강 요청

### 2026-06-12 범위 축소 안내 (#1379 머지 후속)

#1379 머지(devel `6da9cab6`)로 PR 중복 영역이 확대되어 범위 축소 안내 코멘트를 추가 등록했다.

- 코멘트: https://github.com/edwardkim/rhwp/pull/1366#issuecomment-4686340488
- 드랍 권장: 다중 run 분할(#1378/#1379 공유 경로로 supersede), 셀 lineseg IR 보존(#1379
  write_sub_list 전면 교체), SecDef/ColDef 슬롯 소비(#1379 colPr depth 가드),
  secPr 용지 여백(#1388 — 메인테이너 측 처리 예정)
- 유지 권장 (고유 가치): hwp2hwpx CLI + verify 게이트, BinData 1-based 매핑,
  pageBreak CELL/TABLE 의미론, footNotePr/endNotePr IR, ir-diff 의미론 업그레이드
- 기존 수정 요청(fmt, useFontSpace 복구 등)은 그대로 유효
- 상태: 컨트리뷰터 응답 대기
