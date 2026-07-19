# task m100 1692 stage14 - SO-SUEOP 43~46페이지 시각 비교

## 기준

- 기준 PDF: `pdf/SO-SUEOP-2024.pdf`
- 비교 대상:
  - `samples/SO-SUEOP.hwp`
  - `samples/SO-SUEOP.hwpx`
- 비교 산출물:
  - `tmp/pdfs/1692-stage14-pages43-46/compare/pages_043_046_contact.png`
  - `tmp/pdfs/1692-stage14-pages43-46/compare/page_043_pdf_hwp3_hwpx.png`
  - `tmp/pdfs/1692-stage14-pages43-46/compare/page_044_pdf_hwp3_hwpx.png`
  - `tmp/pdfs/1692-stage14-pages43-46/compare/page_045_pdf_hwp3_hwpx.png`
  - `tmp/pdfs/1692-stage14-pages43-46/compare/page_046_pdf_hwp3_hwpx.png`

## 시각 비교 결과

43페이지:

- PDF 기준은 25) 해답 섹션에서 미주 1번부터 58번까지 진행된다.
- HWP3 출력은 1번부터 51번까지 진행되어 PDF보다 수용량이 적다.
- HWPX 출력은 1번부터 50번까지 진행되어 HWP3보다도 더 적게 담긴다.

44페이지:

- PDF 기준은 59번부터 129번까지 진행된다.
- HWP3 출력은 52번부터 120번까지 진행되어 이미 앞 페이지 내용이 밀려 내려온 상태다.
- HWPX 출력은 51번부터 118번까지 진행되어 기준 PDF와 더 크게 어긋난다.

45페이지:

- PDF 기준은 130번부터 191번까지 두 단에 균형 있게 들어가고, footer 위 여백이 남는다.
- HWP3 출력은 121번부터 173번까지만 들어가지만 두 번째 단 하단이 footer와 겹친다.
- HWPX 출력은 119번부터 171번까지만 들어가며, footer 겹침은 적지만 기준 PDF 대비 페이지 단위가 맞지 않는다.

46페이지:

- PDF 기준은 192번부터 223번까지 남은 미주가 배치된다.
- HWP3 출력은 174번부터 223번까지 진행되어 45페이지에서 밀린 내용이 이어진다.
- HWPX 출력은 172번부터 223번까지 진행되어 HWP3보다도 더 많이 밀려 있다.

## 판정

- 45페이지 하단 겹침은 HWP3의 명백한 시각 회귀다.
- 그러나 43~46페이지 전체 기준으로는 HWP3/HWPX 모두 PDF보다 미주 영역의 페이지당 수용량이 부족하다.
- 단순히 45페이지의 173번 미주를 다음 페이지로 넘기는 방식은 footer 겹침은 줄일 수 있지만, PDF 기준 페이지 단위는 더 멀어질 가능성이 있다.
- 다음 수정은 특정 샘플 하드코딩이 아니라 미주/다단 영역의 실제 행 높이, 문단 간격, 또는 본문 영역 계산이 PDF 기준보다 과대 산정되는 원인을 먼저 좁혀야 한다.

## visual_sweep_guide 적용

- `mydocs/manual/verification/visual_sweep_guide.md` 기준으로 `scripts/task1274_visual_sweep.py`를 SO-SUEOP HWP3/HWPX 대상에 직접 적용했다.
- 수정 전 산출물:
  - `output/task1692_visual_sweep/so-sueop-hwp3/compare/compare_043.png`
  - `output/task1692_visual_sweep/so-sueop-hwp3/compare/compare_045.png`
  - `output/task1692_visual_sweep/so-sueop-hwpx/compare/compare_043.png`
  - `output/task1692_visual_sweep/so-sueop-hwpx/compare/compare_045.png`
- 자동 flag는 HWP3 p24 order 후보만 잡았으나, p43~46 compare image에서 미주 separator/상단 gap drift가 명확했다.
- `dump-note-shape` 결과 SO-SUEOP HWP3/HWPX endnote shape는 `separatorLength=0`, `separatorLineType=0`, `separatorLineWidth=1`, `above=864`, `below=576`, `betweenNotes=0` 조합이었다.
- 기존 렌더러는 `separatorLineWidth != 0`만으로 visible separator로 판정하여 길이 0 선을 `col_area.width / 3` 기본 선으로 렌더링했고, no-separator compact flow도 타지 못했다.
- HWPX 렌더에서는 미주 prefix가 앞 공백의 13.33px serif 스타일을 물려받아 본문 10.66px 스타일보다 넓게 그려졌다.

## 수정 후 검증

- 수정 내용:
  - endnote separator visible 판정을 `separatorLineType != 0 && separatorLineWidth != 0` 기준으로 정정.
  - layout의 separator 렌더 조건도 같은 기준으로 통일.
  - visual sweep의 `separatorEnabled` 판정도 렌더러와 같은 기준으로 통일.
  - 미주 prefix 삽입 시 첫 본문 글자 스타일을 prefix 범위에 적용하도록 보정.
- 수정 후 visual sweep 산출물:
  - `output/task1692_visual_sweep_stage14_after_fix/summary.json`
  - `output/task1692_visual_sweep_stage14_after_fix/so-sueop-hwp3/compare/compare_043.png`
  - `output/task1692_visual_sweep_stage14_after_fix/so-sueop-hwp3/compare/compare_045.png`
  - `output/task1692_visual_sweep_stage14_after_fix/so-sueop-hwpx/compare/compare_043.png`
  - `output/task1692_visual_sweep_stage14_after_fix/so-sueop-hwpx/compare/compare_045.png`
- 수정 후 페이지 단위:
  - 43페이지: 1~58
  - 44페이지: 59~129
  - 45페이지: 130~191
  - 46페이지: 192~223
- HWP3/HWPX 모두 기준 PDF와 p43~46 미주 범위가 일치했고, HWP3 p45 footer 겹침이 해소됐다.
- 자동 sweep 결과:
  - HWP3: 46/46쪽, flag 1개(p24 order 후보, 이번 수정 범위 밖)
  - HWPX: 46/46쪽, flag 0개
- 실행 검증:
  - `cargo build`
  - `cargo test issue_1692 --test issue_1692 -- --nocapture`
  - `cargo test issue_1293_clean_visual_sweep_targets_keep_page_counts_and_shape_profiles --test issue_1139_inline_picture_duplicate -- --nocapture`
  - `cargo clippy --all-targets -- -D warnings`
  - `env CARGO_INCREMENTAL=0 cargo test --all-targets`
  - `git diff --check`
