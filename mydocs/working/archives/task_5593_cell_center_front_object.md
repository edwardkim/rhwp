---
kind: working
status: active
issue: 5593
---

# 세로 가운데 정렬 칸의 비-flow 개체 높이 계상 (#5593)

작업 브랜치: `fix/5593-cell-valign-object-height`
대상: `src/renderer/layout/table_layout.rs` · `src/renderer/height_measurer.rs` ·
`tests/cases/issue_5593_cell_center_front_object.rs` ·
`samples/issue5593_cell_center_front_object.hwpx`

## 한 줄

셀 세로 정렬 기준 콘텐츠 높이를 모으는 `cell_wrap_object_visual_bottom` 의 어울림 필터가
`Square|Tight|Through` 만 세어, **글 앞으로/글 뒤로** 개체가 어디에도 계상되지 않았다.
그래서 가운데 정렬 칸이 글자 줄 높이만으로 중앙을 잡고 개체가 칸 밖으로 밀렸다.

## 이슈가 요구한 것

- 칸 85.0px · 글자 줄 16.0px · 그림 77.5px 인 칸에서 그림이 y=871.0(줄 위치)에 그려져
  칸 아래로 27px 넘치는 것을 멈춘다(00425 바코드 2개).
- `836.5 + (85.0 − 77.5) / 2 = 840.3` 이 기대 위치다 — **개체 높이가 정렬 계산에 들어가야 한다.**

## 원인 좁히기 (어울림 모드별 실측)

보고서에는 개체의 어울림 값이 없다. 같은 기하(칸 6400HU·개체 5830HU·CENTER)로 어울림만
바꾼 합성 문서 6종을 돌려 어느 모드가 보고된 수치를 만드는지 특정했다.

| 어울림 | 개체 y | 칸 밖 | 판정 |
|--------|--------|-------|------|
| Square · Tight · Through | 100.1 | 0.0 | 정상 (필터가 이미 셈) |
| TopAndBottom | 139.6 | 33.8 | 합성 파일 한정 — 실제 한컴 저장본은 개체가 줄을 밀어 LINE_SEG 에 흡수됨 |
| **InFrontOfText** | **134.2** | **28.4** | **보고 수치와 같은 산술** (개체 y == 글자 줄 y) |
| **BehindText** | **134.2** | **28.4** | 동상 |

즉 보고 문서의 개체는 글 앞으로/글 뒤로 계열이다. 이 둘은 줄 흐름을 밀지 않으므로 저장
LINE_SEG 에도 흡수되지 않는다 — 필터에서 빠지면 콘텐츠 높이 어디에도 남지 않는다.

부수 효과가 하나 더 있었다. 이 개체들이 `non_flow_object_extent` 에도 안 잡히므로
`trust_stored_cell_flow` 게이트가 통과되어, 정렬 기준 높이가 저장 LINE_SEG extent(줄 높이)로
교체되는 경로까지 열려 있었다.

## 수정

`cell_wrap_object_visual_bottom`(table_layout · height_measurer 두 사본)의 어울림 필터에
`InFrontOfText | BehindText` 를 추가했다.

`TopAndBottom` 은 **제외한 채로 둔다.** 그 개체는 줄을 실제로 밀어 한컴이 저장 vpos 에
흡수하므로(#1486 악보 셀, 코드 주석의 근거) 여기서 다시 세면 이중 계상이 된다.

## 만지지 않은 경로

- `TopAndBottom` 계상 경로, `trust_stored_cell_flow` 게이트 자체
- 개체 배치(`compute_object_position`)·행 높이 산출 규칙
- 새 CLI 명령 없음, DocumentCore 편집 로직 없음

## 재현 픽스처

`samples/issue5593_cell_center_front_object.hwpx` (4.6KB, 합성). 1×1 표, 칸
`vertAlign="CENTER"` · 높이 6400HU(85.3px), 문단에 글자 줄(13.3px) + `IN_FRONT_OF_TEXT`
사각형 5830HU(77.7px).

## 검증 실측

```
rhwp export-render-tree samples/issue5593_cell_center_front_object.hwpx
  칸(Cell)  y= 98.2 h=85.3  bottom=183.5
  수정 전:  Rect y=134.2 h=77.7 bottom=211.9   ← 칸 밖 28.4px (글자 줄 y 와 같음)
  수정 후:  Rect y=100.1 h=77.7 bottom=177.8   ← 칸 안, 같은 기하의 SQUARE 개체와 일치

rhwp export-svg <같은 파일>
  수정 전: <rect y="134.24" height="77.73">
  수정 후: <rect y="100.12" height="77.73">
```

어울림 SQUARE 픽스처는 수정 전후 y=100.1 로 불변 — 기존 정상 경로 무영향.

## 시험 명령

```
cargo test --profile release-test --test regression_suite_015 issue_5593   # 신규 가드
cargo test --profile release-test --tests                                  # 전체
```

신규 가드는 수정 전 코드에서 실패(`칸 98.2..183.5, 개체 134.2..211.9`), 수정 후 통과.

## fmt 게이트

```
cargo fmt --all -- --check
cargo clippy -- -D warnings
```

## 환경 · 한계

Linux 6.17 · rhwp v0.8.4 · 한컴 오라클 없음. 원 보고 문서(admrul 00425)는 이 환경에 없어
어울림 모드별 실측으로 형상을 특정하고 같은 산술의 합성 픽스처로 재현했다. 원본을 구할 수
있으면 그 문서로 한 번 더 확인하는 것이 좋다.

## PR 메모

`gh pr create --base devel --body-file ...`, 제목·본문 한국어, `closes #5593`.
