# #7333 Stage 11 — 셀 스크린샷의 저장 줄 소유 복원

## 분석

기준은 Stage 10 커밋 `885c5b2d7`과 `pdf/issue7333/aaaaaa-2020.pdf`다. Native 전체
Visual Sweep에서 40·41·42·43·44·47쪽의 스크린샷과 그 위 InFrontOfText 주석 도형이 PDF와
어긋났다.

이 여섯 입력은 모두 빈 1×1 `treat_as_char + TopAndBottom` 표이며, 셀 첫 문단은 도형
여러 개 뒤에 스크린샷 그림을 둔다. 셀 `LINE_SEG`는 두 개다. 첫 줄은 `text_start=0,
vpos=0`, 그림이 속한 두 번째 줄은 그림 control의 빈-control stream 위치와 같은
`text_start`에서 `vpos=1600HU`다.

| 바깥 문단 | 그림 control | 셀의 두 번째 줄 text_start | 저장 y 보정 |
| ---: | ---: | ---: | ---: |
| 523 | 4 | 32 | +1600HU = +21.33px |
| 532 | 7 | 56 | +1600HU = +21.33px |
| 539 | 6 | 48 | +1600HU = +21.33px |
| 549 | 7 | 56 | +1600HU = +21.33px |
| 557 | 6 | 48 | +1600HU = +21.33px |
| 586 | 6 | 48 | +1600HU = +21.33px |

실제 렌더 트리의 `Table → Cell → Image` 방출은 `table_layout.rs`의 빈 TAC 문단 경로다.
기존 구현은 TAC 그림의 순번만으로 `LINE_SEG`를 고르는데, 앞선 `InFrontOfText` 주석 도형은
그 순번에 포함되지 않는다. 그러므로 그림 control은 두 번째 저장 줄을 소유해도 첫 줄로
판정됐다. `table_cell_content.rs`의 직접 fallback도 `line_segs.first()`를 써 같은 잘못된
가정을 갖고 있어 함께 보완한다. p42·p44·p47의 파란 UI 영역은 수정 전 rhwp에서
y=362/388/388px, PDF에서 382/408/408px로 일관되게 약 20px 아래였으며, 저장 1600HU
차이와 일치한다.

이 회차는 셀 내부 `Picture`의 줄 판별을 `control_line_seg_index(para, ctrl_idx)`가
가리키는 저장 줄로 바꾼다. 소유 위치가 없거나 손상된 문서만 기존 TAC 순번 폴백을 쓴다.
같은 셀의 앞선 InFrontOfText 주석 도형은 각자의 control 줄 소유를 유지한다. 즉
스크린샷이 두 번째 줄에 저장된 사실을 도형 전체 이동 규칙으로 일반화하지 않는다.

## 코드 수정

`src/renderer/layout/table_layout.rs`의 빈 TAC 분기에서 기존 `tac_seq_index` 선택 대신
실제 control의 저장 줄을 우선 사용하도록 했다. `src/renderer/layout/table_cell_content.rs`의
직접 fallback도 동일한 줄 선택을 사용한다. 앞선 비-TAC 도형은 순번을 소비하지 않으므로,
그림만 두 번째 줄로 이동하며 주석 도형의 기존 위치는 유지된다.

## 검증 결과

- `cargo fmt --all`, `git diff --check` 통과.
- `CARGO_TARGET_DIR=target/pr-review cargo test --locked --profile release-test --test
  regression_suite_020 issue_7333_overlapping_picture_lines -- --nocapture`: 10 passed,
  0 failed. 새 회귀는 p40·41·42·43·44·47의 각 그림 control bbox y를 한컴 PDF 측정값
  ±1px로 고정한다.
- Native: `CARGO_TARGET_DIR=target/pr-review cargo build --locked --profile release-test
  --bin rhwp` 후 `/tmp/rhwp-issue7333-stage11-cell-picture-native-20260923/issue7333-stage11-cell-picture-native`.
  선택 6/6쪽 완료, 구조 검사 flagged 0, pixel match 평균 95.87725%, ink match 평균
  70.79507%. `review_contact_sheet.png`와 p40·41·42·43·44·47 overlay를 직접 확인했다.
- Fresh WASM: 저장소 루트에서 `CARGO_TARGET_DIR=target/pr-review
  scripts/wasm-pack-locked.sh --target web --out-dir pkg` 실행. `pkg/rhwp.js`와
  `rhwp-studio/public/rhwp.js` SHA-256은 `2b7e7bb01cbbff0cb0d3c9a3222c6cb187f9d9710045013bcfb077d7abef4133`,
  배경 wasm 양쪽은 `701c9e4fd85c5af8e7a502907c56f5c7dd40a0b79ef83b559156ee7b602f2d49`로 일치한다.
  `/tmp/rhwp-issue7333-stage11-cell-picture-wasm-rsvg-20260923/issue7333-stage11-cell-picture-wasm-rsvg`
  에서 선택 6/6쪽 완료, flagged 0, Native와 동일한 지표와 overlay를 얻었다.

이번 회차는 여섯 쪽의 스크린샷·주석 도형 줄 소유만 닫는다. 전체 50쪽의 남은 항목은 다음
스테이지에서 별도로 판정한다.
