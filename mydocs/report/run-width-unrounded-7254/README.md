# #7254 — 배치·정렬 run 폭의 정수 반올림 제거

`estimate_text_width` 의 마지막 `total.round()` 를 배치와 정렬 기준 폭이 그대로 소비해,
줄 나눔이 쓰는 비반올림 폭과 다른 값으로 줄을 놓고 있었다. run 경계마다 최대 ±0.5px 가
붙고 뒤 run 들이 그만큼 밀린다.

## 실행 정보

| 항목 | 값 |
| --- | --- |
| base | `devel 62f1048f4` release 바이너리 (`md5 6f9ea0c6`) |
| cand | 같은 트리 + 이 PR source, release 바이너리 (`md5 137b96d1`) |
| 도구 | `scripts/visual_sweep.py` (Chrome webfont raster, `--dpi 96`) · `rhwp export-render-tree` · `rhwp layout-anomaly` |
| 기준 PDF | `pdf/table_scattered_header_rowbreak-2024.pdf` (`Haansoft Batang` 9.9519pt) |

## 1. `run-origin-before-after-oracle.png` — 정답지 대비 run 원점

`samples/table_scattered_header_rowbreak.hwp` 1쪽 첫 줄 `【별표 2】` 를 9배로 확대해
수정 전 / 수정 후 / 한/글 2024 PDF 순으로 놓았다.

| run 원점(px) | 정답지 | 수정 전 | 수정 후 |
| --- | ---: | ---: | ---: |
| `【` | 75.479 | 75.6 (+0.121) | 75.6 (+0.121) |
| `별` | 88.752 | 88.6 (−0.152) | 88.9 (+0.148) |
| `2` | 122.174 | 121.6 (−0.574) | 122.3 (+0.126) |
| `】` | 129.850 | 129.6 (−0.250) | 130.0 (+0.150) |
| **최대 오차** | — | **0.574** | **0.150** |

잔여 0.12~0.15px 는 글자 크기 표기 차(PDF 9.9519pt = 13.269px vs rhwp 10pt = 13.333px,
0.48%)로 설명되는 몫이다. 같은 값을 `tests/cases/issue_7254_run_width_unrounded.rs` 가
run 전진폭으로 잠근다(수정 전 FAIL 0.273px, 수정 후 PASS 0.060px).

## 2. `native-t7254-review-p1-{before,after}.png` · `native-t7254-overlay-p1-after.png`

같은 쪽 전체의 Native 3단 review(rhwp · 한/글 PDF · overlay)와 standalone overlay.
표 외곽·행 위치·앞뒤 문단·줄바꿈이 수정 전후로 같고 정답지와도 같은 구성이다.
보조 픽셀값은 `pixel_match 91.154% → 91.148%`, `ink_match 7.418% → 7.407%` 로
소수점 둘째 자리 차이다(판정이 아니라 참고값).

## 3. `hwp3-sample10-p2-dot-leader-before-after.png` — text-overlap 천장이 오른 2문서

`samples/hwp3-sample10-hwp5.hwp` 2쪽(0-based 1) 차례의 점 리더 줄이다. 새로 세는 짝
(`TextLine28/TextRun3` × `TextRun4`)은 **점 리더 run 과 쪽번호 run** 이고, 수정 전 겹침
1.787px → 수정 후 2.827px 로 이 게이트의 보고 기준 2.0px 를 넘었다. 그림에서 보듯 점과
쪽번호 `52` 의 **글자는 수정 전후 모두 겹치지 않는다** — 겹치는 것은 점 리더 run 의
bbox 이고 그 겹침은 수정 전에도 있었다.

같은 문서의 새 짝 41건(및 hwpx 변형 41건) 전부를 `--overlap-tolerance 0.01` 로 다시 재면
수정 전 겹침이 **0.787~1.787px** 다. 수정 전 0px 이던 짝이 새로 겹친 건 0건, 사라진 짝도
0건이다.

## 4. samples 전수 1,139문서 A/B

`python tools/layout_anomaly_batch_report.py --rhwp <bin> --jobs 4`

| 지표 | base | cand | Δ |
| --- | ---: | ---: | ---: |
| pageCount | 15,350 | 15,511 | +161 (전부 `issue2063_huge_cellbreak_table.hwp` — base 에서 TIMEOUT, cand 에서 완주) |
| overflow | 2,369 | 2,372 | +3 (같은 문서) |
| overlap | 129 | 129 | 0 |
| emptyPage | 95 | 95 | 0 |
| offCanvas | 343 | 343 | 0 |
| textOverlap | 3,491 | 3,569 | +78 (3문서 감소 · 2문서 증가) |

위 한 문서를 빼면 **쪽 경계가 움직인 문서가 없다**.
