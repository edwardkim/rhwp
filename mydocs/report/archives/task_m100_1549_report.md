# Task #1549 결과보고서 — visible host 제목/표 세로 레이아웃을 한컴 기준으로 정렬

## 이슈

- GitHub #1549 (#1510 계열, #1535와 별개): visible host 문단(섹션 제목)에 양수 offset co-anchored
  TopAndBottom float 표가 있을 때 host 제목이 표 아래로 밀리거나/표와 겹치거나/간격 없이 붙어 렌더.
- 본 작업은 #1535(PR #1548) 위에 스택된다.

## 근본 원인 (런타임 계측 확정)

`typeset.rs` `place_table_with_text` 가 visible-para float 의 host 제목을 pre-text 가 아니라 표들 뒤
post-text(PageItem `[표…, 제목]`)로 emit 하고, `layout.rs` 의 `visible_float_exclusions` consume 가 **자기 문단
표가 만든 zone 까지** 적용해 제목을 zone 하단으로 점프시킨다. 제목·표가 선행 float exclusion 으로 같은 y
까지 밀리며 작은 양수 offset 이 흡수돼 둘이 붙고, 표의 외곽여백이 배치/exclusion 에 반영되지 않아 제목↔표·
표↔다음 섹션 간격이 0 이 된다.

- render tree 실측: 제목 "완제품 생산" y=193.9‥212.5, 관능검사 표 y=193.9‥393.2(겹침).
- #1535(PR #1548) 클램프는 가드가 `is_current_visible_para_float` 라 빈-host float 을 건너뛰고 host 제목 줄·
  외곽여백을 다루지 않는다. #1510 큰 offset 표는 제목 한참 아래라 영향 없음(기존 테스트 통과 유지).

## 수정 (`src/renderer/layout.rs`, 4파트)

1. **제목이 자기 표 아래로 안 밀림**: `VisibleFloatExclusion` 에 `owner_para` 추가, consume 시 같은 문단
   (`item_para == owner_para`) 텍스트는 자기 표 zone skip.
2. **후행 float 의 흡수된 offset 복원**: #1535 클램프 가드를 `is_para_topbottom_float` 로 확장(빈-host 포함),
   빈-host float 은 zone 하단 아래로 자기 `v_offset` 만큼 복원해 표-표 간격 형성(visible host 는 3 이 처리).
3. **제목/표 겹침·아래 간격**: visible-host 양수 float 표를 `title_flow_y(선행 exclusion 아래로 밀린 para
   흐름) + host 제목 줄높이 + outer_margin_top` 아래로 클램프. 작은 offset 흡수로 붙던 제목·표 분리 + 표
   윗여백만큼 간격. 큰 offset 은 max 라 무영향, v_off==0 보존(#1510).
4. **표/다음 섹션 제목 위 간격**: float exclusion 하단을 `outer_margin_bottom` 만큼 확장.

코어 레이아웃(`layout.rs`) 외 다른 모듈 변경 없음(+61줄). PageItem 순서·`current_height` 누적은 미변경.

## 산출물

- 수정: `src/renderer/layout.rs`
- 회귀 fixture(issue1510 HWPX Clone and Narrow):
  - `samples/issue1549_multipositive_float_tables.hwpx` — 양수 소offset 3표가 제목 라인과 겹침.
  - `samples/issue1549_empty_host_float_clamp.hwpx` — 표 C 를 빈 문단으로 분리한 cross-para 빈-host float.
- 회귀 테스트: `tests/issue_1549.rs` (2 test)
  - 제목이 co-anchored float 표 위 + 표가 제목 줄 침범 안 함(part 1·3). 수정 전 fail(table_top≈138.7 <
    title_bottom≈150.9), 수정 후 pass.
  - 빈-host 후행 float 가 선행 점유밴드 침범 안 함(part 2). 수정 전 fail(C top≈166.9 < A bottom≈218.7),
    수정 후 pass.

## 검증

- 전체 `cargo test`: 실패 0(`test result: ok` 151 그룹).
- `tests/svg_snapshot.rs`: 8/8 통과. `tests/visual_roundtrip_baseline.rs`: 통과.
- `tests/issue_1510.rs`: 4/4(회귀 가드). `tests/issue_1535.rs`: 1/1. `tests/issue_1549.rs`: 2/2(red→green).
- `cargo fmt --all -- --check`: clean. `cargo clippy -- -D warnings`: clean.
- 시각 대조(참고): 작업지시자 제공 한컴 PDF 8종 기준 — 일일작업일지·지단생산일지 제목 위치·제목↔표·
  표↔표·표↔다음섹션 간격이 한컴과 일치(검증점검표 등 단순양식도 일치). 환경: macOS, 한컴 편집기
  미접근(제공된 한컴 PDF 를 1차 정답지로 사용). ※ CONTRIBUTING 안내대로 "한컴 PDF 일치"만으로 머지가
  보장되지 않으며 페이지네이션 인접 영역이라 메인테이너 환경 재검증 대상.

## 비고

- float=페이지네이션 인접 영역. typeset 의 PageItem 순서·페이지 분할 로직은 변경하지 않고 layout 의 좌표
  클램프(자기 zone skip / 빈-host offset 복원 / 제목 줄·외곽여백 반영)만 조정 → 전체 테스트 page_count 불변.
- 실파일(나린뜰_* 비공개)은 fixture 로 커밋하지 않고 로컬 수동 대조에만 사용. 상세 과정은
  `mydocs/plans/task_m100_1549.md`.
