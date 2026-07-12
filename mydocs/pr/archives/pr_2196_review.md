# PR #2196 리뷰 - CanvasKit 글머리 기호 폰트 보정

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/2196 |
| 제목 | `task 2190: CanvasKit 글머리 기호 폰트 보정` |
| 작성자 | `jangster77` |
| base | `devel` |
| head | `codex/task_m100_2190-font-symbols` |
| 관련 이슈 | #2190 |
| 규모 | 문서 작성 시점 참고값: 15 files, +297/-1 |
| mergeable | 문서 작성 시점 참고값: `MERGEABLE` |

최종 merge 조건은 PR head 최신 커밋 기준 GitHub Actions 통과와 작업지시자 승인이다.

## 관련 이슈 요약

#2190은 CanvasKit 렌더링에서 `hwpx_sample2.hwpx`의 글머리 기호가 tofu 또는 누락 glyph로 보이는 문제다.
원인은 `web/fonts/NotoSansKR-Regular.woff2` 자체의 CanvasKit glyph coverage 부족으로 확인됐다.

#2191의 ParagraphBuilder 보완은 shaping 경로 안정화이지만, 누락된 glyph를 폰트에 추가하지는 않으므로
#2190의 직접 해결책이 아니다.

## 변경 범위 분석

- `web/fonts/NotoSansKR-Regular.woff2`를 재생성해 기존 한글/라틴 coverage에 Box Drawing과 Geometric
  Shapes 범위를 추가했다.
- `tools/subset_noto_sans_kr_regular.py`와 `ttfs/opensource/NotoSansKR-Regular.ttf`를 추가해 maintainer가
  동일 입력에서 TTF/WOFF2를 재현할 수 있게 했다.
- `rhwp-studio/e2e/canvaskit-font-coverage.test.mjs`와 npm script를 추가해 실제 CanvasKit bundle에서
  `■`, `▪`, `□`, `○`, `─`, `가`의 glyph ID가 `0`이 아님을 검사한다.
- `render-diff.yml`에 `web/fonts/**` 트리거와 font coverage 회귀 테스트를 연결했다.
- MCP HWP 2020 기준 PDF와 visual sweep 증적을 장기 보존 경로에 포함했다.

## 렌더 영향 및 visual sweep 판정

이 PR은 브라우저 CanvasKit 렌더링 폰트 asset을 변경하므로 렌더 영향 PR이다. #2190의 직접 검증 대상은
CanvasKit WOFF2 glyph coverage이며, HWP 2020 PDF 대비 native SVG visual sweep은 기존 font/layout fidelity
잔여를 확인하는 참고 검증으로 분리해 해석했다.

PR review asset:

- CanvasKit 실제 표시 확인: `mydocs/pr/assets/pr_2196/canvaskit_hwpx_sample2_p1.png`
- visual sweep compare: `mydocs/pr/assets/pr_2196/visual_sweep_compare_001.png`
- visual sweep overlay: `mydocs/pr/assets/pr_2196/visual_sweep_overlay_001.png`
- visual sweep review: `mydocs/pr/assets/pr_2196/visual_sweep_review_001.png`

## 사전 검증 결과

- `npm run e2e:canvaskit-font-coverage`
  - `FontMgr` family: `Noto Sans KR`
  - `■`, `▪`, `□`, `○`, `─`, `가` glyph ID 모두 non-zero 확인
- `npm run e2e:renderer-contract`
- `npm run build`
- `CARGO_INCREMENTAL=0 cargo build`
- `git diff --check`
- subset generator 재실행 후 TTF/WOFF2 byte-for-byte 재현 확인
- `python3 -m py_compile tools/subset_noto_sans_kr_regular.py`

## HWP 2020 MCP 기준 PDF

- 원본: `samples/hwpx_sample2.hwpx`
- 출력: `pdf/hwpx_sample2-2020.pdf`
- 페이지 수: 29
- SHA-256: `1c3fa2d2ee9cf0106d7723d931ea04067d8e559fe6a0e104d0845d2ed79101b6`
- 서버 결과: `run_status=0`, `validation=ok`

MCP 서버 URL/IP와 인증 토큰은 공개 문서에 기록하지 않는다. 접근 정보가 필요하면 인증된 collaborator가
별도 비공개 채널로 요청한다.

## Visual Sweep

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
- selected pages: 1
- pixel match: 82.46432%
- visual accuracy proxy: 52.99198%
- flags: `render_tree_frame_tail_overflow`, `content_bottom_drift`, `column_line_band_drift`

판정:

- 1쪽의 CanvasKit 글머리 기호 `■`, `▪`는 브라우저 표시 확인과 glyph coverage 테스트에서 정상 표시를 확인했다.
- visual sweep의 차이는 native SVG export와 HWP 2020 PDF 사이의 기존 font/layout fidelity 잔여이며,
  #2190의 직접 blocker로 보지 않는다.

## 주요 리스크

- WOFF2 파일이 바뀌므로 CanvasKit 기본 typeface를 사용하는 다른 문서의 glyph coverage가 영향을 받을 수 있다.
  기존 한글/라틴 cmap을 보존하고 symbol 범위만 추가했으며, 회귀 테스트로 핵심 glyph를 고정했다.
- visual sweep은 아직 HWP 2020 PDF와 pixel fidelity가 완전히 같지 않다. 이는 별도 font/layout fidelity
  축으로 남기고, 이번 PR의 merge blocker로 보지 않는다.

## 최종 권고

작업지시자 승인과 PR head 최신 GitHub Actions 통과를 조건으로 merge 권고한다. merge 후 #2190은 close하고,
이슈에는 PR 번호, 검증 요약, 기준 PDF, visual review asset 링크를 남긴다.
