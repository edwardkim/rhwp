# Task 1282 Stage 6 — 회전 picture 셀 높이/중심 보정

## 배경

PR 준비 중 사용자 시각 검증에서 회전 picture를 크게 리사이즈하면 셀 높이가 충분히 자동 증가하지
않는 현상이 확인됐다.

추가로 리사이즈 후 개체 속성 대화상자의 회전각 표시가 한컴과 달리 `0`으로 보이는 문제가
확인됐다. 모델의 `rotationAngle`은 34도로 유지됐지만, Studio picture 속성창이 picture의
회전각을 100배 raw 값으로 오해해 `/100` 표시와 `*100` 저장을 하고 있었다.

Stage 2 보정은 다음 기준으로 owner cell height를 계산했다.

```text
picture.vertOffset + picture.common.height + cell padding top/bottom
```

그러나 rhwp-studio 화면에서 실제 선택/렌더 외곽은 회전각이 적용된 visual hull이며,
큰 회전 리사이즈에서는 `common.height`보다 `common.width * sin(angle) + common.height * cos(angle)`가
더 크다. 따라서 `common.height` 기준만으로는 사용자 스샷처럼 회전 외곽이 셀 하단 밖으로
나갈 수 있다.

추가 사용자 검증에서 회전각을 34도에서 0도로 바꾸면 한컴오피스와 달리 위치가 어긋나는
현상도 확인됐다. #1279 계약상 회전 picture는 `common.width/height`가 회전 bbox,
`shape_attr.current_width/current_height`가 실제 그림 크기이므로, 0도 전환 시 common 크기가
current 크기로 바뀌는 것은 맞다. 다만 이때 기존 bbox 중심을 유지하도록 offset도 함께
재계산해야 한컴과 가까워진다.

## 수정 방향

- 셀 내부 picture 변경 후 owner cell 필요 높이를 `common.width/common.height`의 회전 visual hull
  높이 기준으로 계산한다.
- 기존 `vertOffset`과 cell padding은 유지한다.
- 직접 소유 cell은 picture 변경 후 필요한 높이로 grow/shrink 동기화한다.
- 직접 소유 cell path(`path.len() == 1`) 범위는 유지한다.
- picture 속성창의 회전각은 shape와 동일하게 도 단위로 표시/저장한다.
- 회전각 변경 시 0도 포함 모든 분기에서 기존 bbox 중심을 유지하도록 offset을 재계산한다.
- Studio E2E에 리사이즈 후 `rotationAngle` 보존, 속성창 회전각 표시값, 회전 0도 전환 중심 보존,
  축소 후 셀 높이 감소 검증을 추가한다.

## 검증 결과

통과:

```text
cargo fmt --check
cargo test --test issue_1282_rotated_cell_picture_resize -- --nocapture
cargo test --test issue_1279_picture_rotation_save
wasm-pack build --target web --out-dir pkg
cd rhwp-studio && node e2e/table-picture-resize-1282.test.mjs --mode=headless
```

E2E 확인:

```text
rotationAngle: 34 -> 34
object properties rotation input: 34
drag resize owner cell height: 17476 -> 30126
required owner cell height after resize: 30126
rotationAngle-only change: 34 -> 0
rotationAngle=0 bbox center jump: 0.05px
rotationAngle=0 owner cell height: 29569 -> 18092
shrink + rotationAngle=0 owner cell height: 15429
Browser page identity: http://localhost:7700/?url=/samples/ta-pic-001-r.hwp&filename=ta-pic-001-r.hwp, title rhwp-studio
Browser console error/warn: none
```

시각 증적:

- before: `mydocs/report/assets/task_m100_1282_ta_pic_001_r_stage6_before.png`
- after: `mydocs/report/assets/task_m100_1282_ta_pic_001_r_stage6_after.png`
- object properties: `mydocs/report/assets/task_m100_1282_ta_pic_001_r_stage6_props.png`
- rotationAngle=0: `mydocs/report/assets/task_m100_1282_ta_pic_001_r_stage6_rotation0.png`
- rotationAngle=0 object properties: `mydocs/report/assets/task_m100_1282_ta_pic_001_r_stage6_rotation0_props.png`

판정 대기:

- 셀 높이 자동 증가/감소, 회전각 34도 표시, 회전각 0도 전환 후 중심/위치가 사용자 한컴 비교 기준에
  맞는지 시각 판단 필요.
