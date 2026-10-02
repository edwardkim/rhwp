# #7333 Stage 8 — 표 셀 이미지의 실제 y 좌표 잔여 차이

## 분석

Stage 7까지 100% 두 쪽 보기의 선택 오버레이 페이지 원점 불일치는 해소했다. 이 회차는
Native Visual Sweep에서 실제 기하 차이로 확인된 14·22·23쪽을 다룬다. Stage 6의 render tree와
PDF의 이미지 영역을 ECC 정합으로 대조했을 때 rhwp 이미지가 PDF보다 위에 놓였다.

| 쪽 | 문단 | 이미지 control | PDF 대비 rhwp y 차이 |
| --- | ---: | ---: | ---: |
| 14 | 220 | 0 | 약 -3.57px |
| 22 | 329 | 0 | 약 -3.18px |
| 23 | 341 | 0 | 약 -3.64px |

14쪽은 글자처럼 취급하는 1×1 표 셀 안의 그림이고, 22·23쪽은 같은 형태의 표 셀에서
`Square` 배치 그림이다. 세 입력의 테이블·셀·그림 저장 좌표와 layout path를 대조해 공통
원인이 셀 content origin인지, treat-as-character baseline인지, 떠 있는 그림의 문단 상대
오프셋인지 판별한다. 실제 공통 계약을 확인하기 전에는 baseline이나 기대 PDF를 바꾸지 않는다.

## 코드 수정

`src/renderer/layout.rs`의 table PageItem 경로에
`stored_empty_full_band_tac_table_top()`을 추가했다. 아래 조건을 모두 만족하는 경우만
두 번째 저장 줄의 page-relative `vertical_pos`와 표의 위 outer margin으로 표 상단을
결정한다.

- 빈 host 문단이고, 표가 `treat_as_char + TopAndBottom`이다.
- 구현용 합성 `LINE_SEG`가 아니며 두 번째 줄의 `vertical_pos`가 앞 줄보다 크다.
- 두 번째 줄의 `text_height`가 `선언 표 높이 + 위·아래 outer margin`과 8HU 이내로 같다.
- 앞 줄의 `text_height`는 이 full-band의 절반보다 작고, 같은 서명을 만족하는 줄이 하나뿐이다.
- 저장 좌표가 현재 단 높이를 유의미하게 넘지 않는다.

따라서 일반 인라인 표, 셀 내부 표, 가시 텍스트 host, 재조판이 필요한 문단은 기존 흐름과
baseline 경로를 유지한다. 이 문서의 14·22·23쪽은 PageItem으로 그려지는 full-width TAC
표라 기존 paragraph inline 경로가 아니라 이 경로에서만 보정해야 했다.

회귀 검사도 14·22·23쪽의 바깥 표선 상단을 각각 350.6px, 327.2px, 295.2px로 고정했다.

## 검증 결과

- `cargo fmt --all -- --check` 통과
- `git diff --check` 통과
- `CARGO_TARGET_DIR=target/pr-review cargo test --locked --profile release-test --test regression_suite_020 issue_7333_overlapping_picture_lines -- --nocapture`
  - 8 passed, 0 failed
- Native Visual Sweep:
  ```sh
  python3 scripts/visual_sweep.py \
    --out /tmp/rhwp-issue7333-stage8-visual-20260923 \
    --rhwp-bin target/pr-review/release-test/rhwp \
    --hwp samples/issue7333/aaaaaa.hwp \
    --pdf pdf/issue7333/aaaaaa-2020.pdf \
    --key issue7333-stage8 --pages 14,22,23 --embed-fonts full
  ```
  - 3/3 페이지 완료, structural flagged page 0
  - 14쪽: pixel 93.96610% → 95.88364%, ink 73.61134% → 81.21472%
  - 22쪽: pixel 93.42632% → 94.19735%, ink 70.29911% → 73.24093%
  - 23쪽: pixel 93.18105% → 95.02110%, ink 71.06926% → 77.82058%
  - overlay: `/tmp/rhwp-issue7333-stage8-visual-20260923/issue7333-stage8/overlay/overlay_014.png`,
    `overlay_022.png`, `overlay_023.png`

이번 회차는 세 full-band 표의 page-relative y 오차를 해소했다. 문서 전체의 다른 도형,
글꼴 또는 잉크 차이를 해소했다고 판단하지 않는다.

### Studio/WASM 검증

저장소 루트에서 다음 표준 명령으로 WASM을 다시 빌드했다.

```sh
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg
```

- `rhwp-studio`에서 `npx tsc --noEmit` 통과
- `node --test tests/picture-hit-policy.test.ts tests/object-selection-page.test.ts`: 6 passed
  - 두 쪽 보기에서 선택 핸들이 선택된 실제 페이지의 `getPageLeftResolved()` 원점을 쓰는
    Stage 7 회귀 검사를 포함한다.
- `pkg/rhwp.js` = `rhwp-studio/public/rhwp.js`,
  `pkg/rhwp_bg.wasm` = `rhwp-studio/public/rhwp_bg.wasm` SHA-256 일치
- Chrome WASM export 후 `rsvg` rasterizer Visual Sweep: 14·22·23쪽 3/3 완료,
  structural flagged page 0, 평균 pixel 95.63059%, 평균 ink 80.06995%
  - overlay: `/tmp/rhwp-issue7333-stage8-wasm-rsvg-20260923/issue7333-stage8-wasm-rsvg/overlay/`
  - Chrome webfont rasterizer는 이 큰 SVG의 raster 단계에서 JavaScript 호출 스택 한계가
    발생해, 이 회차의 WASM PNG는 동등한 exported SVG를 `rsvg`로 rasterize해 확인했다.
