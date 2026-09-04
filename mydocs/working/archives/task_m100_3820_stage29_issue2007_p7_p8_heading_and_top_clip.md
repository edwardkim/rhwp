---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-06
---

# Task #3820 Stage 29 — issue2007 p7–p8 제목 소유와 continuation 상단 글리프 clip

## 기준과 범위

- 입력: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 독립 기준: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf` (Hancom 2020)
- 대상: PDF physical p7–p8의 제목/표 시작, PDF physical p16–p17의 continuation 상단 글리프

페이지 수나 native SVG의 source node 존재만으로 정상이라고 판정하지 않는다. 기준 PDF의
실제 페이지 소유와 Canvas/SVG가 적용하는 ancestor clip을 함께 판정한다.

## 재현된 p7–p8 결함

`fidelity_compare.py 6 7`의 최신 native render tree에서 제목
`< 해외 반부패 전담기구 조사기능 현황 >`(source `pi=71`)은 두 조각에 동시에 있다.

| physical page | rhwp source 좌표 | 기준 PDF |
| --- | ---: | --- |
| p7 | `y=964.0`, 따라서 p7 하단에 paint | 제목 없음 |
| p8 | `y=93.6`, body/inner-table clip 상단보다 위여서 paint 소실 | 제목과 7×3 표가 함께 시작 |

따라서 이는 글꼴 raster 차이가 아니라, completed nested-table 다음 제목의 첫 visible
unit을 이전 fragment가 소유하고 다음 fragment에서는 같은 unit을 clip 밖에 남기는
source-owner/viewport 결함이다.

## p16–p17 상단 절단 가설

사용자가 최신 WASM Canvas에서 확인한 physical p16/p17의 첫 글자 상단 절단을 별도
계약으로 다룬다. 현재 tree에서 p16 `<이해관계자 협의>`와 p17 `선호된 대안의 기대효과`의
TextRun top은 모두 `y=114.6`이다. 같은 continuation의 inner table/cell top frame은
그보다 아래에 있어, glyph ink의 ascender가 ancestor Canvas clip에 걸릴 수 있다.

이 경우 native tree에 TextRun이 존재한다는 사실은 paint 성공 증거가 아니다. 수정은
앞 페이지의 잔여 text를 노출하지 않는 범위에서 현재 fragment 첫 visible text의 ink
extent가 clip 안에 들어오게 해야 한다.

## 진행 순서

1. p7→p8 mixed-nested cut에서 제목과 뒤 표를 하나의 새 physical viewport로 소유시키고
   p7 중복 paint를 제거한다.
2. p16–p17의 첫 visible line에 대해 ancestor cell clip과 TextLine ink top의 관계를
   regression으로 고정하고, 필요한 최소 top paint inset을 적용한다.
3. focused Rust regression, `fidelity_compare.py` p7–p8/p16–p17, 기준 PDF page pair로
   결과를 다시 대조한다.

## 구현과 검증 결과

`src/renderer/layout/table_layout.rs`에 clipped `TableCell`의 direct source sibling만
대상으로 하는 seam 보정을 추가했다.

- 표의 첫 paintable row가 다음 fragment에서 시작하고 제목이 현재 clip 안에만 남은 경우,
  현재 fragment의 제목/표 group을 비표시 처리한다. 이는 physical p7의 title만 남고 표는
  p8에서 시작하는 중복 paint를 막는다.
- 같은 group이 다음 fragment에서 clip 바로 위에 남은 경우에는 title과 바로 뒤 표를 같은
  `y` delta로 이동한다. title과 표의 상대 간격은 바꾸지 않으며 clip을 확장하지 않는다.
- continuation의 첫 visible `TextLine`이 ancestor cell clip보다 최대 4px 위에 있을 때만
  0.25px inset으로 이동한다. 이는 p16/p17의 ascender ink 절단을 고치되 이전 page text를
  재노출하지 않는다.

새 회귀는 p7에서 제목이 paint되지 않고 p8에서 제목과 표가 함께 남는지, 그리고 p16/p17의
첫 제목 top이 inner-cell clip `y=117.1` 아래 `y>=117.3`인지 고정한다. 또한 paint 여부
검사 helper가 `RenderNode::visible`을 반영하도록 바로잡았다.

2026-08-06 native 검증 결과는 다음과 같다.

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --check` | 통과 |
| `CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2007_nested_cell_pagination -- --nocapture` | 9 passed |
| `fidelity_compare.py 6 7` | p7 하단 title 제거, p8 title/table 동시 시작 확인 (diff 17.50%, 15.56%) |
| `fidelity_compare.py 15 16` | p16/p17 첫 title top `117.4`; ancestor clip 안쪽 확인 (diff 18.00%, 2.83%) |

중간 비교 산출물은 `/tmp/rhwp-stage29-verify.vVHO96/`에 있다. 해당 raster 비교 환경은
한글 font fallback에 따라 글리프가 box로 보일 수 있으므로, 위의 page owner와 clip 좌표 및
사용자가 실행한 WASM Canvas 확인을 함께 판정 근거로 쓴다.

## 이전 기록 정정

Stage 25와 Stage 27의 native-source-owner 결론은 browser Canvas clip까지 통과했다는
뜻이 아니었다. 특히 Stage 27의 “p5–p7 heading owner 일치”는 이번 physical p7–p8
대조로 반증되므로, 수정 결과와 함께 superseded 상태로 정정한다.
