# Task #1116 Stage 2 보고서 — p2 목차 leader 보정 및 p3 위치 가드

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: 구현 및 1차 회귀 검증 완료

## 1. 수정 요약

p2 목차에서 tab leader 점선이 페이지 번호 뒤로 과도하게 연장되는 문제를 보정했다.

수정 전:

```text
p2 dotted leader max_x2 = 732.7
```

수정 후:

```text
p2 dotted leader max_x2 = 624.5
```

페이지 번호 글자가 x≈610~631 범위에 있으므로, leader가 페이지 번호 시작 직전에서 멈추도록 정리됐다.

## 2. 수정 파일

```text
src/renderer/layout/paragraph_layout.rs
src/renderer/layout/text_measurement.rs
src/renderer/svg.rs
tests/issue_1116.rs
```

## 3. 구현 내용

1. `paragraph_layout.rs`
   - cross-run right-tab 보정에서 마지막 leader 하나만 줄이던 처리를 같은 TextRun의 모든 leader로 확장했다.
   - line node 완성 후 같은 줄의 다음 실제 텍스트 시작 x를 기준으로 leader end를 추가 클램프한다.

2. `text_measurement.rs`
   - 같은 run 안에 여러 tab leader가 있을 때 앞 leader가 뒤 leader보다 길게 남지 않도록 후처리했다.
   - 탭 뒤 같은 문자열 안의 실제 콘텐츠 시작 위치를 leader 종료 후보로 반영했다.

3. `svg.rs`
   - 최종 SVG 렌더 단계에서 실제 `char_positions` 기준으로 탭 뒤 숫자 시작 위치를 확인해 leader 종료점을 다시 클램프했다.
   - `"\t 15"`처럼 탭과 페이지 번호가 같은 TextRun에 있는 sample16 목차 케이스를 잡는다.

4. `tests/issue_1116.rs`
   - p2 목차 leader가 페이지 번호 뒤로 연장되지 않는 회귀 테스트를 추가했다.
   - p3 `2. 추진방향`, `3. 주요 추진내용`의 heading 위치가 현재 `LINE_SEG.vpos` 기반 좌표를 유지하는 가드를 추가했다.

## 4. 기준 SVG 재생성

작업지시자 한컴 캡처가 3mm 격자이므로, `--show-grid=3mm` 옵션을 추가하고 같은 단위로 다시 생성했다.

```bash
target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp \
  -o output/poc/render-spacing/hwp3-sample16-hwp5-page3-grid-3mm \
  -p 2 \
  --show-grid=3mm \
  --debug-overlay \
  --show-control-codes
```

생성 파일:

```text
output/poc/render-spacing/hwp3-sample16-hwp5-page3-grid-3mm/hwp3-sample16-hwp5_003.svg
output/debug/task1116/hwp5-p2-grid-3mm/hwp3-sample16-hwp5_002.svg
```

SVG grid pattern 확인:

```text
width=11.3386 height=11.3386
```

## 5. 검증 결과

통과:

```bash
cargo test --test issue_1116 -- --nocapture
cargo test --test issue_874_ktx_toc_page_number_right_align -- --nocapture
cargo test --test issue_1086 -- --nocapture
cargo test --test issue_1105 -- --nocapture
cargo test --test issue_630 -- --nocapture
cargo fmt --all -- --check
git diff --check
```

참고:

- `issue_874` 실행 중 기존 `LAYOUT_OVERFLOW` 진단 로그가 1건 출력되었으나 테스트는 통과했다.
- p3 본문은 3mm 격자 전제에서 상단 목적 박스 주변보다 줄 문단 누적을 주시해야 한다. 이번 구현에서는 p3 좌표를 임의 보정하지 않고 현재 `LINE_SEG.vpos` 기반 위치를 회귀 가드로 고정했다.

## 6. 다음 단계

작업지시자 한컴오피스 시각 판정 후, 필요하면 p3 본문 누적 간격을 별도 Stage 3로 이어간다.
