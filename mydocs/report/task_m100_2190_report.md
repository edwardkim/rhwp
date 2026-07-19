# Task M100 #2190 보고서 - CanvasKit 글머리 기호 폰트 보정

## 대상

- GitHub issue: #2190
- 기준 브랜치: `upstream/devel`
- 작업 브랜치: `codex/task_m100_2190-font-symbols`
- 재현 샘플: `samples/hwpx_sample2.hwpx`
- HWP 2020 기준 PDF: `pdf/hwpx_sample2-2020.pdf`

## 원인

`web/fonts/NotoSansKR-Regular.woff2`의 CanvasKit glyph coverage에서 `U+25A0`(`■`)와
`U+25AA`(`▪`)가 glyph ID `0`으로 확인됐다. CanvasKit 렌더링 경로는 브라우저 폰트 fallback에
의존하지 않으므로, WOFF2 자체에 글머리/도형 기호가 없으면 tofu 또는 누락 glyph로 표시된다.

#2191의 `ParagraphBuilder` 보완은 shaping 경로 안정화이며, 누락된 glyph를 폰트 파일에 추가하지
않으므로 #2190의 직접 원인은 별도로 해결해야 했다.

## 수정 내용

1. `web/fonts/NotoSansKR-Regular.woff2`를 Noto Sans KR weight 400 정적 subset으로 재생성했다.
2. 기존 Regular coverage를 유지하고 `U+2500-257F` Box Drawing, `U+25A0-25FF` Geometric Shapes를
   추가했다.
3. 재현 가능한 생성을 위해 `tools/subset_noto_sans_kr_regular.py`와 source TTF
   `ttfs/opensource/NotoSansKR-Regular.ttf`를 추가했다.
4. 실제 CanvasKit bundle이 새 WOFF2에서 `■`, `▪`, `□`, `○`, `─`, `가`를 모두 non-zero glyph로
   매핑하는 회귀 테스트를 추가했다.
5. `render-diff.yml`이 `web/fonts/**` 변경에도 font coverage 회귀 테스트를 수행하도록 연결했다.

## 검증

- `npm run e2e:canvaskit-font-coverage`
  - `FontMgr` family: `Noto Sans KR`
  - `■`, `▪`, `□`, `○`, `─`, `가` glyph ID 모두 non-zero 확인
- `npm run e2e:renderer-contract`
- `npm run build`
- `CARGO_INCREMENTAL=0 cargo build`
- `git diff --check`
- subset generator 재실행 후 TTF/WOFF2 `cmp` 일치 확인
- MCP HWP 2020 변환
  - 입력: `samples/hwpx_sample2.hwpx`
  - 출력: `pdf/hwpx_sample2-2020.pdf`
  - 페이지 수: 29
  - SHA-256: `1c3fa2d2ee9cf0106d7723d931ea04067d8e559fe6a0e104d0845d2ed79101b6`

## 브라우저 확인

Vite 7700에서 `hwpx_sample2.hwpx`를 CanvasKit 경로로 열어 1쪽의 `■`와 `▪` 글머리 기호가 tofu 없이
표시되는 것을 확인했다.

증적:

- `mydocs/report/assets/task_m100_2190/canvaskit_hwpx_sample2_p1.png`

## Visual Sweep

`mydocs/manual/verification/visual_sweep_guide.md` 기준으로 HWP 2020 PDF와 rhwp native SVG export를 비교했다.

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key issue2190-hwpx-sample2 \
  --hwp samples/hwpx_sample2.hwpx \
  --pdf pdf/hwpx_sample2-2020.pdf \
  --page 1 \
  --rhwp-bin target/debug/rhwp \
  --out output/visual_sweep_issue2190
```

결과:

- SVG pages: 29
- PDF pages: 29
- Selected pages: 1
- pixel match: 82.46432%
- ink match / visual accuracy proxy: 52.99198%
- flags: `render_tree_frame_tail_overflow`, `content_bottom_drift`, `column_line_band_drift`

증적:

- `mydocs/report/assets/task_m100_2190/visual_sweep_compare_001.png`
- `mydocs/report/assets/task_m100_2190/visual_sweep_overlay_001.png`
- `mydocs/report/assets/task_m100_2190/visual_sweep_review_001.png`

판정:

- 이 visual sweep은 native SVG export와 HWP 2020 PDF의 기존 폰트/레이아웃 fidelity 차이를 보여준다.
- #2190의 직접 검증 대상은 브라우저 CanvasKit WOFF2 glyph coverage이며, 해당 경로에서는 글머리 기호가
  정상 표시됨을 확인했다.
- visual sweep 잔여 차이는 #2190의 blocker가 아니라 별도 font fidelity/레이아웃 fidelity 축으로 남긴다.
