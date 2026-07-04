# Task 1282 최종 보고서

## 이슈

- GitHub Issue: https://github.com/edwardkim/rhwp/issues/1282
- 제목: 회전된 표 안 그림 리사이즈/셀 높이 조작 정합 개선
- 브랜치: `local/task_m100_1282`

## 변경 요약

- `set_cell_picture_properties_by_path_native`에서 표 셀 내부 picture 속성 변경 후 직접 소유 cell height를 필요한 높이로 동기화하도록 보정했다.
- cell height 기준은 회전 picture의 visual hull 높이까지 포함한 `picture.vertOffset + rotated_bounds(width, height, rotation).height + cell padding top/bottom`이다.
- 직접 표 셀 path(`path.len() == 1`)만 대상으로 하며, 글상자/깊은 중첩 path는 기존 동작을 유지했다.
- 회전각 변경 시 0도 전환을 포함해 기존 bbox 중심을 유지하도록 `horizontal_offset`/`vertical_offset`을 재계산했다.
- `tests/issue_1282_rotated_cell_picture_resize.rs`를 추가해 리사이즈 후 cell/table height와 export/reparse 보존을 검증했다.
- `rhwp-studio/e2e/table-picture-resize-1282.test.mjs`를 추가해 실제 Studio 마우스 드래그 경로에서 cellPath, 크기, cell height, bbox 안정성, rotationAngle 보존, 속성창 회전각 표시, undo, 회전각 0도 전환 중심 보존, 축소 후 cell height 감소를 검증했다.
- picture 속성창의 회전각 표시/저장 단위를 도 단위로 정정해 한컴처럼 34도가 34도로 표시되게 했다.
- picture offset 속성값은 signed 값으로 노출/입력되도록 정리해 음수 `horzOffset`/`vertOffset`이 u32 래핑값으로 표시되지 않게 했다.

## 검증

통과:

```text
cargo build --release
cargo test --release --lib
cargo test --profile release-test --tests
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --test issue_1282_rotated_cell_picture_resize
cargo test --test issue_1279_picture_rotation_save
wasm-pack build --target web --out-dir pkg
cd rhwp-studio && node e2e/table-picture-resize-1282.test.mjs --mode=headless
cd rhwp-studio && npm run build
```

비고:

- `cargo clippy --all-targets -- -D warnings`는 작업지시자가 로컬 터미널에서 직접 실행해 통과를 확인했다.
- `issue_713_rowbreak_table_no_intra_row_split` 회귀는 Stage14에서 `TableCellNode.clip` 의미를 복원해 해소했다.

## 시각 검증 자료

`build-web-apps:frontend-testing-debugging` 기준으로 Browser plugin 경로를 확인했다.

- Browser page identity: `http://localhost:7700/?url=/samples/ta-pic-001-r.hwp&filename=ta-pic-001-r.hwp`, title `rhwp-studio`
- 앱 chrome DOM: 파일/편집/입력/서식 메뉴 확인
- console error/warn: 없음
- Browser screenshot: 로컬 스튜디오 첫 화면 렌더 확인

Stage5 Headless Chrome 캡처는 `samples/ta-pic-001-r.hwp` 기준이었으나, 스크린샷에 파일명이 직접 보이지 않아 Stage6에서 라벨 포함 증적으로 교체했다.

Stage6 Headless Chrome 캡처:

- before: `mydocs/report/assets/task_m100_1282_ta_pic_001_r_stage6_before.png`
- after: `mydocs/report/assets/task_m100_1282_ta_pic_001_r_stage6_after.png`
- object properties: `mydocs/report/assets/task_m100_1282_ta_pic_001_r_stage6_props.png`
- rotationAngle=0: `mydocs/report/assets/task_m100_1282_ta_pic_001_r_stage6_rotation0.png`
- rotationAngle=0 object properties: `mydocs/report/assets/task_m100_1282_ta_pic_001_r_stage6_rotation0_props.png`

Stage11 한컴 PDF 기준 비교 자료:

- restrict 비교: `mydocs/report/assets/task_m100_1282_stage11/comparison_restrict.png`
- restrict 한컴 PDF raster: `mydocs/report/assets/task_m100_1282_stage11/pdf_restrict.png`
- no restrict 비교: `mydocs/report/assets/task_m100_1282_stage11/comparison_no_restrict.png`
- no restrict 한컴 PDF raster: `mydocs/report/assets/task_m100_1282_stage11/pdf_no_restrict.png`

확인 수치:

```text
drag resize picture height: 18160 -> 20970
rotationAngle: 34 -> 34
object properties rotation input: 34
drag resize owner cell height: 17476 -> 30126
required owner cell height after resize: 30126
rotationAngle-only change: 34 -> 0
rotationAngle=0 picture size: 15787 x 14649
rotationAngle=0 bbox center: (212.75, 293.70) -> (212.75, 293.75)
rotationAngle=0 owner cell height: 29569 -> 18092
shrink + rotationAngle=0 owner cell height: 15429
```

Before:

![Task 1282 Stage6 before](assets/task_m100_1282_ta_pic_001_r_stage6_before.png)

After:

![Task 1282 Stage6 after](assets/task_m100_1282_ta_pic_001_r_stage6_after.png)

Object properties:

![Task 1282 Stage6 properties](assets/task_m100_1282_ta_pic_001_r_stage6_props.png)

RotationAngle 0:

![Task 1282 Stage6 rotation 0](assets/task_m100_1282_ta_pic_001_r_stage6_rotation0.png)

RotationAngle 0 object properties:

![Task 1282 Stage6 rotation 0 properties](assets/task_m100_1282_ta_pic_001_r_stage6_rotation0_props.png)

Restrict comparison:

![Task 1282 Stage11 restrict comparison](assets/task_m100_1282_stage11/comparison_restrict.png)

No restrict comparison:

![Task 1282 Stage11 no restrict comparison](assets/task_m100_1282_stage11/comparison_no_restrict.png)

판정:

- 회전된 표 셀 picture의 드래그 리사이즈 전/후가 화면상으로 확인된다.
- 리사이즈 후 owner cell height가 회전 visual hull 기준 필요한 높이까지 증가했다.
- 회전각 0도 단독 변경 후 bbox 중심이 보존되고 owner cell height가 필요한 높이로 감소했다.
- 리사이즈 후 picture의 `rotationAngle`은 34도로 유지되고, 속성창에도 34도로 표시된다.
- 회전각 0도 속성창도 0도로 표시된다.
- 화면 bbox와 실제 그림이 분리되거나 picture가 사라지는 증상은 재현되지 않았다.
- `쪽 영역 안으로 제한` on/off 샘플이 한컴 PDF 기준 비교와 같은 배치로 렌더링된다.
- 제한 on 상태에서는 셀 경계를 침범하지 않고, 제한 off 상태에서는 no 샘플과 같은 흐름으로 분리된다.

## 남은 사항

- 없음.
