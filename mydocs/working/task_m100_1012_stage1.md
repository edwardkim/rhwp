# Task #1012 Stage 1 — Root cause 진단

이슈: [#1012](https://github.com/edwardkim/rhwp/issues/1012)
계획서: [task_m100_1012.md](../plans/task_m100_1012.md)

## 1. 진단 결과

`test-image.hwp` page 1 의 paragraph 텍스트 라벨 (자리차지/글앞으로/어울림/글뒤로) 이 한컴 viewer 와 다르게 표시되는 문제를 분석.

### 1-1. paragraph 구조

- pi=0: text "자리차지   글앞으로   어울림   글뒤로", controls=6
  - [0] 구역정의, [1] 단정의 (system controls)
  - [2] 그림 (wrap=위아래/TopAndBottom, tac=false, z=0)
  - [3] 그림 (wrap=어울림/Square, tac=false, z=1)
  - [4] 그림 (wrap=글뒤로/BehindText, tac=false, z=3)
  - [5] 그림 (wrap=글앞으로/InFrontOfText, tac=false, z=2)
- line_seg: `ts=0, vpos=15180 HU (=202.4px), lh=1000, sw=42520`

### 1-2. 결함 — text y 좌표 어긋남

`./target/release/rhwp export-svg samples/test-image.hwp -p 0` 결과:
- text y=143.6px (body_top 75.6 + 68px)
- image y=86~334 (Pic[2] TopAndBottom: y=132~334)
- → 라벨이 Picture[2] 영역 내부에서 시작 → 시각적 overlap

### 1-3. root cause

`paragraph_layout.rs` 의 spacing_before 처리 로직 (line 989-1000) 이:
- `spacing_before > 0` 인 경우만 vpos 클램프 분기 진입
- pi=0 의 spacing_before=0 → 분기 SKIP → vpos=15180 무시
- 결과: 텍스트가 column-top (body_top=75.6) 에 그려짐

### 1-4. z-order 보조 결함

SVG 출력 순서:
- 모든 `<text>` 먼저
- 모든 `<image>` 나중
- → 라벨이 image 뒤에 그려져 가려짐 (BehindText image 가 그 라벨을 가리는 잘못된 방향)

단 결함 1-3 해결 시 텍스트가 image 영역 아래로 이동하므로 z-order 충돌 시각 영향 사라짐.

## 2. Stage 2 진입

`paragraph_layout.rs` 의 vpos 클램프 분기를 확장 — `spacing_before=0` + column-top + para_index==0 + line_seg.vpos > 0 인 경우 추가로 `y += vpos0_px` 적용.

## 3. 변경 파일

| 파일 | 상태 |
|------|------|
| `samples/test-image.hwp` / `.hwpx` | PR #1011 fixture — 본 task 에 임시 보존 (devel merge 후 제거) |
| 진단 SVG `/tmp/t1012/` | 보존 (gitignore) |

## 4. 회귀 risk

vpos > 0 인 paragraph 가 column-top 에 위치 (pi==0) 한 일반 케이스에서:
- 기존 동작: text 가 column-top 에 그려짐
- 새 동작: text 가 vpos 만큼 push-down

대부분 일반 paragraph 는 column-top 의 첫 line_seg vpos 가 0 (즉시 그려짐). 본 fix 는 vpos > 0 (인코더가 명시한 push-down) 인 경우만 영향.

Stage 4 의 회귀 sweep 으로 확인 예정.
