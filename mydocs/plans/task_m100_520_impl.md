# Task #520 구현 계획서 — inline 도형(tac=true, wrap=TopAndBottom) y 좌표 오류 수정

## 1. 핵심 코드 위치 (조사 결과)

| 역할 | 파일 | 라인 |
|------|------|------|
| inline shape y 좌표 산출 (메인) | `src/renderer/layout/paragraph_layout.rs` | 1769–1779 |
| 동일 로직 (빈 문단 변형) | `src/renderer/layout/paragraph_layout.rs` | 2103–2110 |
| inline_pos 등록 자료구조 | `src/renderer/render_tree.rs` | 734–745 |
| inline_pos 가 없을 때 그리기 차단 | `src/renderer/layout/shape_layout.rs` | 217–235 |
| sequential vpos 보정 - TopAndBottom + tac 제외 | `src/renderer/layout.rs` | 1398–1417 |
| 문단 줄 메트릭 산출 (`line_seg[0]/[1]` 가정) | `src/renderer/layout/paragraph_layout.rs` | 208–265 |

`paragraph_layout.rs:1775` 의 산출식:
```rust
let shape_y = (y + baseline - shape_h).max(y);
```
은 “현재 줄의 y, baseline” 값을 그대로 사용한다. 즉 shape 가 등록되는 시점의 `y`/`baseline` 변수가 어느 line_seg 의 것인지에 따라 결과가 갈린다.

또한 `paragraph_layout.rs:208–265` 는 `line_seg[0]=표를 포함한 줄, line_seg[1]=텍스트 줄` 이라는 특정 가정으로 메트릭을 결정한다. 본 케이스(p[1] 의 2 line_seg 는 “텍스트 + (인라인 사각형 포함) 텍스트”)에는 이 가정이 들어맞지 않을 가능성이 있어, 실제 동작 확인이 필요하다.

## 2. 단계 (4 단계)

### Stage 1: 재현 환경 구축 + 진단 로그
- `samples/exam_science.hwp` 페이지 3 SVG 출력을 베이스라인 캡처 (`output/svg/exam_science/exam_science_003.svg`).
- `paragraph_layout.rs:1775` 직전에 `RHWP_DEBUG_LAYOUT=1` 로 다음을 임시 출력:
  - `section`, `para_index`, `tac_ci`, `line_idx`, `y`, `baseline`, `shape_h`, `shape_y`, `common.text_wrap`, `common.treat_as_char`, line_seg vpos/lh.
- 실행 후 셀(섹션 0 의 표 pi=33 셀[0]) 내 p[1] 의 ㉠ 두 개가 어떤 `line_idx` / 어떤 `y` 로 등록되는지 확인.
- Stage 1 결과 보고서: `working/task_m100_520_stage1.md` (관측 로그 + 원인 가설 확정).

### Stage 2: 원인에 맞춰 좌표 산출 수정
Stage 1 관측 결과로 두 가지 갈래 중 하나를 적용:

- **(가)** shape 가 잘못된 line_seg 의 `y`/`baseline` 위에서 등록되는 경우 → `paragraph_layout.rs` 의 line 루프 안에서 “해당 shape character 가 속한 line_seg” 를 명확히 산출하여 그 line 의 `y`/`baseline` 사용.
- **(나)** line_seg 식별은 정확하나 `wrap=TopAndBottom` 의 의미(자기 줄 위/아래에 텍스트 배제)와 `tac=true`(글자처럼 baseline 정렬) 충돌로 `shape_y` 산출이 어긋나는 경우 → `tac=true && wrap=TopAndBottom` 분기를 추가하여 `shape_y` 를 line_seg.y 기반으로 재계산.

수정 대상은 최소 변경 원칙을 지킨다. `layout.rs:1404–1406` 의 `!cm.treat_as_char` 조건은 그대로 유지(시퀀셜 vpos 는 이미 정합) — 변경 시 부작용 가능.

### Stage 3: 검증 + 회귀 확인
- `cargo build --release && cargo test`.
- `samples/exam_science.hwp` 페이지 3 SVG 재출력 → PDF 와 비교 (㉠ 박스가 `이다.` 줄에 위치, `[탐구 과정 및 결과]` 와 겹치지 않음).
- 회귀 샘플:
  - `samples/tac-img-02.hwpx` (tac 이미지)
  - `samples/table-vpos-01.hwpx` (표 + vpos)
  - 그 밖에 `samples/` 의 기존 페이지가 깨지지 않는지 일부 페이지 SVG diff (눈검사).
- `cargo clippy -- -D warnings` 통과.

### Stage 4: 최종 보고서 + orders 갱신
- `report/task_m100_520_report.md` 작성 (수정 요약, before/after 스크린샷 경로, 회귀 결과).
- `orders/20260502.md` 에 Task #520 상태 갱신.
- 진단 로그(eprintln) 는 정식 `RHWP_DEBUG_LAYOUT` 가드 내부로 정리하거나 제거.

## 3. 산출물

- 코드: `src/renderer/layout/paragraph_layout.rs` (예상 단일 파일 수정)
- 문서:
  - `working/task_m100_520_stage1.md` ~ `_stage4.md`
  - `report/task_m100_520_report.md`
- 비교 이미지: `output/svg/exam_science/` (수정 후) — git 비추적

## 4. 위험 / 회피

- p[1] 의 2 line_seg 메트릭 산출(`paragraph_layout.rs:210–265`) 은 “표 + 텍스트” 가정이라 본 케이스에 불일치할 수 있음 → Stage 1 로그로 영향 여부 먼저 확인. 영향 시 Stage 2 에서 가정 분기를 정밀화.
- inline shape 두 개(p[1] ctrls=2) 모두 동일 동작인지 Stage 1 에서 확인.
- HWPX 동등 파일 부재로 `ir-diff` 사용 불가 → 시각 비교만 가능.

## 5. 비-목표

- `wrap=TopAndBottom` 의 비-tac 케이스는 본 타스크 범위 외.
- inline 표/그림/수식 좌표 산출(같은 파일 내 다른 분기)은 본 타스크 범위 외.
