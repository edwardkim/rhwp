# #7333 Stage 10 — 전체 대역 TAC 표 뒤 저장 흐름 복원

## 분석

기준은 `131d80e78`(Stage 9)과 `pdf/issue7333/aaaaaa-2020.pdf`다. 전체 50쪽 Native·fresh
WASM Visual Sweep에서 14·22·23·33쪽은 표 자체의 위쪽 frame은 PDF와 맞지만, 표 뒤의 후속
문단이 PDF보다 위에 남았다.

| 쪽 | 표 host 문단 | 표 소유 저장 줄 | 다음 문단 저장 위치 | rhwp 후속 문단 차이 |
| --- | ---: | --- | --- | ---: |
| 14 | 220 | `vpos=11840`, `text_height=32546`, `line_spacing=660` | 221: `45046` | 약 -8.8px |
| 22 | 329 | `vpos=10080`, `text_height=33517`, `line_spacing=660` | 330: `44257` | 약 -8.8px |
| 23 | 341 | `vpos=7680`, `text_height=35582`, `line_spacing=660` | 342: `43922` | 약 -8.8px |
| 33 | 452 | `vpos=5600`, `text_height=37557`, `line_spacing=780` | 453: `43937` | 약 -10.4px |

각 행에서 다음 문단의 저장 위치는 정확히
`소유 줄 vertical_pos + text_height + line_spacing`이다. 또한 소유 줄의 `text_height`는
`표 선언 높이 + 표 위·아래 outer margin`과 8HU 이내로 일치한다. 이는 한컴이 표의 paint
frame뿐 아니라 표 뒤 진행 위치까지 해당 저장 줄에 기록한 직접 증거다.

Stage 8의 `stored_empty_full_band_tac_table_top()`은 이 계약의 paint origin만 복원했다. 이후
`layout_table_control_block()`은 일반 TAC의 line spacing 경로를 타지만, 이 네 입력은 표 앞의
가시 장식 도형 때문에 소유 `LINE_SEG`를 최종 흐름에 다시 적용하지 않는다. 결과적으로 표는
맞는 y에 그려지고 다음 문단만 한 `line_spacing`만큼 위에 남는다.

이 회차는 빈 HWP5 host, `treat_as_char + TopAndBottom` 표, 실제 control 소유 저장 줄,
표 outer-box와 같은 `text_height`, 그리고 바로 다음 문단의 정확한 저장 등식이 모두 성립할
때만 다음 문단의 저장 top을 흐름 하한으로 적용한다. 표의 paint 위치, 일반 TAC, 셀 내부 표,
가시 텍스트 host 및 저장 등식이 없는 입력은 기존 경로를 유지한다.

40·41·42·43·44·47쪽의 화면 캡처와 주석 도형 차이는 이 흐름 문제와 별개다. Native/PDF
overlay에서 p40·p41·p43의 스크린샷 내부 좌표는 PDF와 약 21px 다르고, p42·p44·p47은
그보다 큰 frame/anchor 차이가 남는다. 이들은 다음 Stage에서 테이블 셀의 그림 frame과
InFrontOfText 도형의 anchor를 PDF와 source renderingInfo로 대조한다. 이번 흐름 보정으로
그 차이를 넓은 규칙으로 감추지 않는다.

## 코드 수정

`src/renderer/layout.rs`에 `stored_empty_full_band_tac_table_flow_end()`를 추가했다.
이 helper는 `control_line_seg_index()`가 가리킨 실제 표 소유 줄과 다음 문단의 첫 저장 줄을
대조한다. 표의 outer-box와 `text_height`, 그리고 다음 `vertical_pos`가 정확한 등식을
만족할 때만 다음 문단의 저장 top을 반환한다.

`layout_table_control_block()`의 HWP5 TAC 완료 경로는 기존 spacing·outer margin 처리를 모두
마친 뒤 이 값으로 `y_offset`의 하한만 올린다. 따라서 일반 규칙의 결과가 이미 더 아래이면
그 값을 유지하며, 다른 표·다른 wrap·가시 텍스트 host의 위치를 낮추지 않는다.

`tests/cases/issue_7333_overlapping_picture_lines.rs`에는 PDF에서 측정한 표 뒤 첫 가시 줄의
render-tree y를 고정하는 회귀 검사를 추가했다. 대상은 14쪽 pi=222, 22쪽 pi=331, 23쪽
pi=344, 33쪽 pi=454다.

## 검증 결과

- `cargo fmt --all -- --check` 통과
- `git diff --check` 통과
- `CARGO_TARGET_DIR=target/pr-review cargo test --locked --profile release-test --test regression_suite_020 issue_7333_overlapping_picture_lines -- --nocapture`
  - 9 passed, 0 failed
- `CARGO_TARGET_DIR=target/pr-review cargo build --locked --profile release-test --bin rhwp` 통과

Native Visual Sweep은 `rsvg` PNG rasterizer로 다음 명령을 실행했다.

```sh
python3 scripts/visual_sweep.py \
  --out /tmp/rhwp-issue7333-stage10-flow-native-20260923 \
  --rhwp-bin target/pr-review/release-test/rhwp \
  --hwp samples/issue7333/aaaaaa.hwp \
  --pdf pdf/issue7333/aaaaaa-2020.pdf \
  --key issue7333-stage10-flow-native --pages 14,22,23,33 \
  --embed-fonts full --svg-rasterizer rsvg
```

- 4/4 완료, `run_state=complete`, flagged page 0
- 평균 pixel match 96.06908%, 평균 ink match 82.05206%
- 실제 render-tree 첫 가시 후속 줄: 14쪽 815.2px, 22쪽 804.7px, 23쪽 825.8px,
  33쪽 802.5px. PDF 측정값 815.5px, 804.9px, 826.0px, 802.9px와 모두 0.4px 이내다.
- Native overlay: `/tmp/rhwp-issue7333-stage10-flow-native-20260923/issue7333-stage10-flow-native/overlay/`

저장소 루트에서 fresh WASM도 표준 wrapper로 다시 빌드했다.

```sh
CARGO_TARGET_DIR=target/pr-review \
  scripts/wasm-pack-locked.sh --target web --out-dir pkg
```

- `pkg/rhwp.js` = `rhwp-studio/public/rhwp.js` SHA-256 `2b7e7bb01cbbff0c…`
- `pkg/rhwp_bg.wasm` = `rhwp-studio/public/rhwp_bg.wasm` SHA-256 `701c9e4fd85c5af…`
- WASM export 후 동일 4쪽 `rsvg` sweep도 4/4 완료, `run_state=complete`, flagged page 0,
  평균 pixel/ink match가 Native와 동일했다.
- WASM overlay: `/tmp/rhwp-issue7333-stage10-flow-wasm-rsvg-20260923/issue7333-stage10-flow-wasm-rsvg/overlay/`

review contact sheet를 직접 확인해 네 쪽의 표 뒤 문단이 PDF와 같은 line band에 놓인 것을
확인했다. 이 회차는 40·41·42·43·44·47쪽의 화면 캡처·주석 도형 frame/anchor 잔여 차이를
해결하지 않는다. 그 항목은 다음 Stage에서 source renderingInfo와 PDF overlay로 별도 검증한다.
