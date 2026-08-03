---
kind: investigation
status: active
canonical: mydocs/manual/verification/visual_sweep_guide.md
last_verified: 2026-08-02
---

# Task #3738 Stage 7 — HWPX 이월 그림의 outer host vpos 전달 분석

- 선행 commit: `03c9577a4` (`fix: #3738 HWPX 이월 그림 표 page ownership 정합`)
- 기준 자료: 개인정보 제거 원본 HWP·HWPX와 각각의 한컴오피스 2020 PDF
  ([경로·SHA-256·Git/LFS 판정](../../pdf/pr3740/README.md))
- 직전 증적: [Stage 6 visual sweep](task_m100_3738_stage6_visual_sweep.md)

## Stage 6 잔여 결함의 직접 원인

Stage 6에서 `stored_layout_relocated_empty_rowbreak_picture_resets_offset`의 profile gate는 original
HWPX까지 열렸지만, 호출자 `layout.rs`는 `outer_host_stored_vpos_hu` 자체를 여전히
`native_hwp5_layout()`일 때만 수집한다. 따라서 HWPX p344에서는 helper가 필요한 `Some(52230)` 대신
`None`을 받고 즉시 `false`를 반환한다. table은 Stage 6의 typeset gate로 p24에 이월됐지만, picture의
`-52790 HU` page-boundary 상쇄를 reset하지 못해 tree `y=-181.4px`가 남았다.

원본 HWPX p344 XML은 필요한 저장 근거를 모두 가진다.

```text
outer hp:tbl: pageBreak="CELL" → parser RowBreak, TopAndBottom / Para / flowWithText=1
outer host LINE_SEG.vpos = 52230 HU; table vertOffset = 560 HU
cell: 1×1, height = 36782 HU; one empty paragraph, LINE_SEG.vpos = 0
hp:pic: TopAndBottom / Para / flowWithText=1, vertOffset = -52790 HU
caption: Bottom, 3 stored lines; next p345 vpos = 30689 HU
boundary sum: 52230 + 560 - 52790 = 0 HU
```

HWPX `pageBreak="CELL"`의 IR `RowBreak` 정규화, picture/caption 및 geometry는 HWP dump와 같고,
Stage 6의 strict helper predicate도 이 형상을 위해 작성된 조건을 모두 만족한다. 차이는 저장 vpos를
table-cell 경로로 전달하는 호출자 profile gate 하나다.

## 좁은 수정과 검증 경계

`layout.rs`에서 outer host 첫 non-synthetic `LINE_SEG.vpos`를 수집하는 source gate만 `native HWP5
또는 original HWPX stored-layout`으로 확장한다. nested/header/footer table에는 이미 `None`을 전달하는
구조를 바꾸지 않으며, downstream helper의 1×1·RowBreak·TopAndBottom·empty cell·caption·정확한 boundary
sum 조건도 유지한다.

## 구현과 검증 결과

`layout.rs`의 outer host vpos 수집 gate만 `native HWP5 또는 original HWPX stored-layout`으로
확장했다. nested/header/footer의 `None` 전달 경로와 downstream strict helper 조건은 바꾸지 않았다.
release-test binary build 및 `cargo test stored_layout_relocated --lib` focused regression을 완료했다.

HWPX p23–p24 144 DPI sweep에서 p344는 23쪽에 없고, 24쪽 render tree에서 table `y=90.6px`, image
`y=92.5px`, Bottom caption 3줄 `y=434.4/455.7/477.1px`로 복원됐다. review PNG에서 그림 23 full graph,
caption, `○ EU에서 …` 뒤의 표 4가 기준 PDF와 같은 순서·page-local 위치에 있다. Stage 5의 증가-vpos
경로를 확인하는 p13–p15 sweep 3/3도 visual flag 없이 완료했다.

p24에는 일반 문서의 `○` bullet을 exam marker로 오인한 `question_marker_flow_drift` 자동 후보가 남지만,
이는 그림 23 구조 결함이 아니다. overlay 수치는 글꼴 raster와 전체 chart 색 차이를 포함하는 보조값이고,
보관 review PNG 및 p344 render tree가 실제 page ownership·image/caption 복원을 직접 뒷받침한다. 이
조사 범위의 잔여 그림 23 흐름 결함은 확인되지 않았다. 원본 HWP·HWPX·기준 PDF와 review PNG는 계속
보관한다.
