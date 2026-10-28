# Task #1639 결과보고서 — 빈 host 음수 offset float 표의 배치 순서를 문서 순서로 보존

## 이슈

- GitHub #1639 (#1510 계열, PR #1518 이후 빈-host 잔존): 빈 host 문단에 co-anchored TopAndBottom float 표가
  2개 이상 있고 후행 표의 `vertical_offset` 이 음수면 배치/페이지 순서가 배열(문서) 순서와 역전.
- #1535(PR #1548)·#1549(PR #1568)는 visible-host 계열, 본 건은 빈-host 계열.

## 근본 원인 (소스 확정)

`src/renderer/typeset.rs` `should_sort_para_float_tables`(빈 host 에서 정렬, #986/#1088) + 정렬 키
`vertical_offset as i32`(음수 키) + `sort_by_key` 오름차순 → 음수 offset 표가 양수 형제 앞으로 정렬되어
`ctrl_order` 가 역전 → 배치/렌더/페이지 순서 역전. #1510·PR #1518 은 visible-host 만 보정, 빈-host 음수 잔존.

- 실파일(나린뜰) dump: 빈 문단0, `[2]`설명 +1154, `[3]`본문 u32 4294962885 = i32 -4411 → 1p 본문/2p 설명 역전.
- 파싱은 정상(offset 비트 보존). 정렬 휴리스틱이 "voffset 오름차순 = 문서 순서"를 가정하는데 음수에서 깨진 것.

## 수정 (`src/renderer/typeset.rs`)

빈 host 정렬 게이트에 음수 para-topbottom float 존재 검사를 추가:

- `has_negative_para_float` = `para.controls` 중 `is_para_topbottom_float(common) && signed_hwpunit(vertical_offset) < 0`
- `should_sort_para_float_tables = !para_has_non_whitespace_text(para) && !has_negative_para_float`

음수 혼재 빈 host 는 정렬 OFF → 배열(문서) 순서 보존. 양수 전용 빈 host 정렬(#986/#1088)·visible-host
제외(#1510)는 불변. 정렬 키/배치 downstream(`is_first/last_placed`, `FloatLaneSet`)은 미변경(게이트 1곳만 조정).
경계: `signed_hwpunit < 0` 인 음수만 트리거, `offset == 0` 은 양수와 함께 정렬 유지(주석에 명문화).

## 산출물

- 수정: `src/renderer/typeset.rs` (게이트 + 음수/0 경계 주석)
- 회귀 fixture(issue1549_multipositive Clone and Narrow):
  - `samples/issue1639_empty_host_negative_offset_float.hwpx` — 빈 host + co-anchored float 표 `ci=2` (+200) /
    `ci=3` (-4411) / `ci=4` (+800). 음수 혼재 → 정렬 OFF(문서 순서) 검증용.
  - `samples/issue1639_empty_host_positive_only_float.hwpx` — 위 fixture 에서 `ci=3` 만 양수(+50)로 변형
    (`ci=2` +200 / `ci=3` +50 / `ci=4` +800). 양수 전용 → 정렬 유지(offset 순) 검증용.
- 회귀 테스트: `tests/issue_1639.rs` (3 tests) — (1) 음수 혼재 빈-host 표 배치가 문서 순서 `[2,3,4]`
  보존(수정 전 `[3,2,4]`), (2) render tree y 좌표도 문서 순서대로 위→아래(음수 offset 표가 선행 형제 위로
  점프 안 함, 실측 y=[134.9, 217.1, 304.6]), (3) 양수 전용 빈-host 는 vertical_offset 오름차순 정렬 유지
  `[3,2,4]`(#986/#1088 동작 잠금 — 음수 가드가 양수 정렬까지 끄지 않음).

## 검증

- 전체 `cargo test`: 실패 0(exit 0).
- `issue_986`(2)·`issue_1510`(4)·`issue_1535`(1)·`issue_1549`(2)·`issue_1488`(1)·`page_number_propagation`(2)·
  `issue_1639`(3) 전부 `ok`.
- `cargo fmt -- --check` clean, `cargo clippy -- -D warnings` clean.
- `hwpx_roundtrip_baseline`: 4 passed(신규 fixture 구조 보존, batch 자동 포함).
- 교차검증(메모리 디버깅 교훈: typeset/layout 두 경로 분리): render tree(`build_page_render_tree`)와
  typeset(`dump-pages`) 양 경로 모두 표 순서 `[3,2,4]`(수정 전) → `[2,3,4]`(수정 후) 복원.

## 비고

- 옵션 검토: A(`max(0)` 클램프)는 선행 표가 양수면 미해결 → B(음수 시 정렬 OFF) 채택. C(lane-aware)는 범위 초과.
- 게이트 1곳만 조정, 정렬 키·배치 downstream 미변경 → 기존 동작(page_count 등) 불변. `issue_986`(빈-host
  multi-float, offset 0) 회귀 없음 확인.
- 실파일(나린뜰 비공개)은 fixture 로 커밋하지 않고 로컬 대조에만 사용. 상세 과정은 `mydocs/plans/task_m100_1639.md`.
- float = 페이지네이션 인접 영역. CONTRIBUTING 안내대로 메인테이너 환경 재검증 대상.
