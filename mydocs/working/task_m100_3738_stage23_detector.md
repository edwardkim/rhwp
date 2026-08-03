---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 23 detector — p127 본문 flow 붕괴 자동 후보화

## 출발 근거

유지보수자 직접 비교에서 native HWP p127은 그림 56의 physical owner는 맞지만 그림 왼쪽·주변 본문이
PDF와 달리 좁은 세로 열로 재흐름한다. Stage 22 selected sweep도 실제로 p127을
`question_marker_flow_drift`로 flag했지만, 이는 그림 내부의 분홍 raster를 문항 marker로 오인한 일반
후보다. 본문 flow가 붕괴했다는 결론이나 renderer 수정 대상으로 자동 분류하지 못했으므로, 수동 발견을
기다리는 방식은 충분하지 않다.

## 자동 판정 계약

`visual_sweep.py`는 단별 raster line-band를 이미 PDF와 대조한다. 이를 이용해 다음 세 조건을
같은 column에서 모두 만족할 때 `column_text_flow_collapse`를 추가한다.

1. rhwp/PDF line-band 수 차이가 3 이상일 것
2. 대응 band의 평균 y drift가 80px 이상일 것
3. p90 y drift가 120px 이상일 것

이는 일반 폰트 baseline 차이보다 훨씬 큰 재flow만 우선 후보로 삼는 결합 조건이다. 자동 불합격이나
원인 확정이 아니라, review PNG와 기준 PDF를 즉시 여는 **강한 triage signal**이다.

## 구현과 회귀

- `column_text_flow_collapse_candidates`가 기존 단별 line-band 지표에서 candidate·band count delta·사유를
  만든다.
- `analyze_page`는 다른 semantic flag 유무와 무관하게 이 후보를 `flags`에 기록하고 annotated PNG에
  `TEXT FLOW COLLAPSE` 라벨을 남긴다.
- summary와 [visual sweep guide](../manual/verification/visual_sweep_guide.md)에
  `column_text_flow_collapse_pages`/`flowcollapse`를 추가했다.
- `scripts/tests/test_visual_sweep.py`는 p127에서 관측된 `34 vs 37`, mean `109.4`, p90 `157.0`
  지표를 positive regression으로, band 수가 같은 큰 baseline shift를 negative regression으로 고정한다.

실행:

```text
python3 -m unittest scripts/tests/test_visual_sweep.py
```

결과: **14 tests passed**.

## p127 실증

수정한 detector로 Stage 22와 같은 native revision을 p127에 다시 실행했다.

```text
python3 scripts/visual_sweep.py \
  --key issue3738-stage23-p127-detector \
  --hwp samples/정책연구용역사업\ 중간진도보고서\(살아있는\ 간장\ 기증자의\ 의학적\ 선별기준\ 연구\).hwp \
  --pdf pdf/pr3740/hwp/정책연구용역사업\ 중간진도보고서\(살아있는\ 간장\ 기증자의\ 의학적\ 선별기준\ 연구\)-2020.pdf \
  --pages 127 --dpi 144 \
  --rhwp-bin target/review-planet6897-20260802/release-test/rhwp
```

결과 p127은 `question_marker_flow_drift`와 함께 **`column_text_flow_collapse`**로 자동 flag되었다.
후보의 오른쪽 단 지표는 `rhwp_count=34`, `pdf_count=37`, `mean=109.4px`, `p90=157.0px`,
`band_count_delta=3`이며 사유는 `column_line_count_and_y_flow_diverge`다.

이 Stage는 detector만 완료했다. `pi=1355` 그림 56의 다음-page anchor와 `pi=1356` narrow wrap band를
renderer가 함께 복원하는 결함 자체는 다음 code Stage에서 고친다.
