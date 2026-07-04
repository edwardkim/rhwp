# PR #1577 처리 보고서 — native Skia raster replay 최적화 (성능, 픽셀 동등)

- PR: https://github.com/edwardkim/rhwp/pull/1577
- 제목: `perf: optimize native Skia raster replay`
- 작성자: humdrum00001010 (#1569 동일 author)
- 연결: 없음 (독립 성능 PR)
- base ← head: `devel` ← `humdrum00001010:codex/skia-raster-perf`
- 처리일: 2026-06-27

## 1. 처리 결정

**admin merge.** native Skia raster replay 경로 최적화 — 큰 `PaintOp` payload Box화(캐시
친화), RawSvg 폰트 DB/typeface 매칭 캐시, PNG 압축 비용 감소. 컨트리뷰터의 시각 동등 주장을
메인테이너가 픽셀 단위로 직접 검증해 통과 확인. CI 전부 pass + 전체 회귀 통과 + 충돌 0건.
(SkiaLayerRenderer::new FontMgr 캐시는 #1569 가 담당, 본 PR 미포함.)

## 2. 변경 범위

paint/renderer-skia 모듈 (17파일 +572/-683, 실제 신규 커밋 = `b91478b8` perf 1개):

- `src/paint/paint_op.rs` — PaintOp variant payload(TextRunNode/RawSvgNode/ImageNode 등) Box화,
  variant별 생성자 함수 추가.
- `src/paint/{builder,json,replay_order,text_*}.rs` — Box 접근 반영 (직렬화 출력 불변).
- `src/renderer/skia/{renderer,font_lookup,image_conv,...}.rs` — replay 최적화, 폰트 DB/매칭 캐시,
  PNG 압축 비용 감소.

> picture_footnote/table_layout 는 #1577 변경이 아니라 #1590(merge-base 이후 devel) 과의 base
> 차이일 뿐(merge-base=507a4cb5, #1590 미포함). #1577 실제 변경은 paint/renderer-skia 만.

## 3. 컨트리뷰터 주장 직접 검증 (메인테이너)

### 시각 동등 (픽셀) — **검증 통과**

#1569 와 달리 PNG byte 가 바뀐다(압축 비용 감소 = 다른 압축 레벨). 따라서 byte-identical 이
아닌 **픽셀 동등**을 검증:

- before(devel) / after(PR) 바이너리를 각각 `--release --features native-skia` 빌드.
- 4개 샘플 page 0 `export-png` → PNG 디코드 후 RGBA 픽셀 비교:
  - `143E433F503322BD33.hwp` (891,662px): **최대 채널차 0, 다른 픽셀 0** | byte 298447→301653
  - `hy-001.hwpx`(HWPX 표/이미지): 픽셀 동일 | byte 189301→171430
  - `eq-01.hwp`(수식): 픽셀 동일 | byte 96753→95372
  - `hwp_table_test.hwp`(표): 픽셀 동일 | byte 61214→70452
- **4개 샘플 전부 디코드 픽셀 100% 동일** → 압축 인코딩만 다르고 렌더 내용 불변 확인.
- CI `Canvas visual diff` / `Render Diff` pass 와 일치. 증빙: `output/poc/pr1577/pixel_equivalence.txt`.

### 성능

- PR 측정: warm in-process page render+PNG median 58.5ms → 29.0ms(~2×), RawSvg 폰트 셋업
  ~11.4ms/render → 0.001ms warm. (Live Editing warm per-page 경로 겨냥.)

## 4. 기타 검증

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze/Canvas visual diff/Render Diff) | 전부 pass |
| 충돌 시뮬레이션 | 0건 |
| `cargo test --features native-skia skia --lib` | 48 passed |
| `cargo test --features native-skia --lib` | 2009 passed / 0 failed |
| `paint::json`(14) / `paint::`(64) (release-test) | passed — Box화 직렬화 무영향 |
| 전체 `cargo test --tests` (release-test) | FAILED 0건 (lib 1962 passed) |
| `cargo clippy --features native-skia --lib -- -D warnings` | clean |

## 5. 산출물

- 본 처리 보고서: `mydocs/pr/archives/pr_1577_review.md`
- 검증 증빙: `output/poc/pr1577/pixel_equivalence.txt`
