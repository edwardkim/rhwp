---
kind: report
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820·#3821 — HWP 215쪽 전수 결함 종합 보고서

- **이슈**: #3820, #3821
- **기준 브랜치**: `task/3820-3821-fidelity` (2026-08-08 Stage 57 검증)
- **판정 대상**: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- **정답지**: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf` (한컴 2020 기준 PDF)

## 1. 결론

215쪽 전수 raster·overlay 기준선은 Stage 7에서 **215/215쪽, 누락 0**으로 생성했다.
당시 rhwp 전체 export는 219쪽이었고 Stage 9 시점에도 218쪽이었으나, 이후 표·그림·각주와
reset 경계의 source owner를 순차적으로 보정했다. Stage 11 마감 재검사에서는 한컴 2020
기준 PDF와 rhwp 전체 export가 모두 **215쪽**으로 회복됐고, p166-p215 text-owner 검사는
50/50쪽을 완료했다. 남은 p176→p177·p178→p179 후보도 PDF raster와 RenderTree 직접 대조에서
각주·URL 텍스트 추출 순서의 false positive로 확인했다.

별도 실물 fixture인 `issue2007_nested_cell_pagination_42065.hwp`는 최초 24쪽 과분할에서
17쪽으로 회복됐으며, Stage 56에서 p11에 조기 출력되던 `3 중앙선거관리위원회` 제목과 다음
표 상단선을 제거하고 p12의 정확한 source owner로 복원했다. 현재 rhwp와 기준 PDF는
**17/17쪽**이고 focused integration 15/15가 통과한다.

현재 확정된 상태는 다음과 같다.

1. p118→p119 `TopAndBottom` 그림 앞 본문 owner와 p127 그림 56 page-top geometry는
   Stage 8·9에서 해소됐다.
2. p168 이후 연쇄 pagination divergence의 실제 owner drift는 Stage 11에서 해소됐고,
   정책 문서의 전체 page count는 215/215다.
3. issue2007의 24→17쪽 과분할, PUA 두부 문자, p11→p12 제목 owner는 Stage 11·56에서
   해소됐다.
4. page count와 자동 후보 0건은 전체 시각 fidelity의 무결함 증명이 아니다. #3820은
   후속 실문서 후보가 남아 있으므로 active 상태를 유지하고, 각 페이지는 한컴 PDF와 직접
   대조해 판정한다.

근거:

- [Stage 11 정책 문서 215/215 및 p166-p215 마감](../working/task_m100_3820_stage11.md)
- [Stage 56 issue2007 p11→p12 owner](../working/task_m100_3820_stage56_issue2007_p11_heading_owner.md)

## 2. 전수 검증 기준선 및 최신 상태

아래 실행은 최초 215쪽 전수 raster inventory인 Stage 7 기준선이다. 표의 219쪽과 `+4`는
현재 상태가 아니라 수정 전 결과이며, compare·overlay·review 215/215가 실제 생성됐음을
증명하는 재현 기록으로 보존한다.

실행 명령:

```text
python3 scripts/visual_sweep.py \
  --key stage7-full-215 \
  --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --pages 1-215 --dpi 144 \
  --rhwp-bin target/task-3820-3821-fidelity/release-test/rhwp \
  --out output/task-3820-3821-fidelity/stage7-full-sweep
```

| 항목 | 결과 |
| --- | ---: |
| 요청 / 완료 / 누락 | **215 / 215 / 0** |
| 기준 PDF / 이번 선택 SVG / render tree | **215 / 215 / 215** |
| compare / overlay / review PNG | **215 / 215 / 215** |
| rhwp 전체 export SVG / render tree | **219 / 219** |
| PDF 대비 rhwp 전체 page delta | **+4** |

전수 실행 원장은
`output/task-3820-3821-fidelity/stage7-full-sweep/summary.json`에 있고, 페이지별 증적은
`output/task-3820-3821-fidelity/stage7-full-sweep/stage7-full-215/{compare,overlay,review}/`에 있다.

Overlay의 평균 pixel match는 92.09%지만 평균 ink match는 16.50%다. 글꼴 rasterization,
anti-aliasing, 링크색 차이도 ink score를 크게 바꾸므로 이 숫자만으로 결함을 확정하지 않는다.

### 최신 page-owner 재검증

| 대상 | 최신 결과 | 근거 |
| --- | ---: | --- |
| 정책 문서 PDF / rhwp 전체 export | **215 / 215** | Stage 11 |
| 정책 문서 p166-p215 text-owner 검사 | **50 / 50, 누락 0** | Stage 11 postfix4 |
| issue2007 PDF / rhwp 전체 export | **17 / 17** | Stage 56·57 |
| issue2007 focused integration | **15 / 15** | Stage 56·57 |

정책 문서 최신 원장은 `output/task-3820-3821-fidelity/stage11-postfix4-p166-end/`에 있고,
p176-p179 직접 대조는
`tmp/stage11-current-p176-p179-postfix4/p176-p179-reference-current.png`에 있다.
issue2007 최신 page cut과 시각 증적은
`mydocs/pr/assets/task_m100_3820_stage57_exact_head_pr_gate/`에 보관한다.

## 3. 확정·고우선순위 결함

### D-01 — p118→p119 그림 앞 본문 owner가 한 쪽 이르게 확정됨 — 해소

초기 inventory에서는 p119에서 rhwp가 절차 그림으로 바로 시작하지만, 한컴 PDF는 p118의 본문 뒷부분을 p119
상단에 먼저 배치한 뒤 그림을 둔다. `fidelity_compare`도 p118→p119에서
`rhwp_earlier_than_reference`, shared text 72자, 양쪽 coverage 1.000을 기록했다. 다음 p119 상단에는
Body `TopAndBottom` 그림(`pi=1276`, `bbox=94.5,83.2,448.5,359.0`)이 있어, 그림 앞 paragraph owner
결함으로 우선 분석해야 한다.

증적:

- `output/task-3820-3821-fidelity/stage7-full-sweep/stage7-full-215/review/review_118.png`
- `output/task-3820-3821-fidelity/stage7-full-sweep/stage7-full-215/review/review_119.png`
- `output/task-3820-3821-fidelity/stage7-full-sweep/stage7-full-215/compare/compare_118.png`, `compare_119.png`
- `output/task-3820-3821-fidelity/stage6-full-ledger/float-owner-shift-candidates.tsv`의 p118→p119 행

**해소 상태:** `309c5f123`가 p118의 `pi=1275`를 lines `0..8`, p119를 lines `9..10`과 그림 55로
분할해 PDF owner를 복원했다. 직접 증적은 [Stage 8 visual sweep](../working/task_m100_3820_stage8_visual_sweep.md)에 있다.

### D-02 — p127 본문과 그림 56의 폭/배치 관계가 PDF와 다름 — 해소

초기에는 **사용자 직접 확인 결함이자 자동판정 false negative**였다. p127에서 rhwp와 기준 PDF의 그림 56 주변
본문 행폭·완충 관계가 다르다. 이번 visual sweep의 `page_127.json`은 flag와
`square_wrap_text_overlap_candidates`를 모두 0건으로 기록했다. 즉 현 규칙은 실제 그림-본문 관계의
fidelity 저하를 아직 충분히 검출하지 못한다.

증적:

- `output/task-3820-3821-fidelity/stage7-full-sweep/stage7-full-215/review/review_127.png`
- `output/task-3820-3821-fidelity/stage7-full-sweep/stage7-full-215/compare/compare_127.png`
- `output/task-3820-3821-fidelity/stage7-full-sweep/stage7-full-215/overlay/overlay_127.png`

**해소 상태:** Stage 9는 deferred page-start Square 그림의 source `vertical_offset` 이중 적용을
제거해 frame top을 `130.7px`에서 body top `83.2px`로 복원했다. 이전 형상은 새
`deferred_square_picture_top_drift` detector가 후보화하고, 수정 후 p127/p156 직접 PDF review와
focused Rust·Python 회귀는 [Stage 9 visual sweep](../working/task_m100_3820_stage9_visual_sweep.md)에 있다.

### D-03 — p168 이후의 연쇄 pagination divergence — 해소

Stage 7 초기 inventory에서는 p168→p169 owner-shift 뒤로 논리 내용이 연쇄 이탈했고,
rhwp가 기준 PDF보다 4쪽 많았다. p168 표 44 first fragment를 복원한 뒤에도 Stage 9
시점에는 218/215쪽이었다.

Stage 11은 뒤쪽 범위의 실제 잔존 원인을 두 계약으로 분리해 보정했다.

- p182→p183: native HWP5 empty-host 그림 표 뒤 guide line을 다시 flow 높이로 소비하던
  문제를 제거했다.
- p199→p201: 각주 258)이 다음 문단의 명시적 `vpos=0` reset보다 먼저 배치되어 본문 tail을
  잃던 문제를 수정했다.

수정 후 PDF와 rhwp 전체 export는 **215/215쪽**이고, p166-p215 text-owner 원장은
50/50쪽을 완료했다. 남은 p176→p177·p178→p179 후보는 직접 PDF 대조에서 실제 pagination
차이가 아닌 추출 순서 false positive로 판정했다. 상세 원인과 focused 회귀는
[Stage 11](../working/task_m100_3820_stage11.md)에 기록했다.

증적:

- `output/task-3820-3821-fidelity/stage7-full-sweep/stage7-full-215/review/review_168.png` ~ `review_171.png`
- `output/task-3820-3821-fidelity/stage7-full-sweep/stage7-full-215/compare/compare_168.png` ~ `compare_171.png`
- `output/task-3820-3821-fidelity/stage6-full-ledger/text-owner-shift-candidates.tsv`의 p168→p169 행
- `output/task-3820-3821-fidelity/stage6-full-ledger/text-owner-sequence-candidates.tsv`의 p168→p169, p172→p173 행

### D-04 — issue2007 중첩 셀 pagination과 p11→p12 제목 owner — 해소

Stage 11 최초 재현에서는 rhwp가 24쪽, 기준 PDF가 17쪽이었고 중첩 `RowBreak` continuation이
frame 밖으로 누적됐다. 이후 physical continuation과 PUA glyph를 보정해 17/17쪽으로
회복했다.

PR 준비 재검사에서는 page count가 같아도 p12 소유 제목 `3 중앙선거관리위원회`와 다음 표
상단선이 p11에 조기 출력되는 결함을 발견했다. Stage 56은 명시적 source 쪽 나누기 뒤의
recursive prelude만 제한적으로 되감아 다음 결과를 고정했다.

- p11은 `국세기본법` 마지막 문장으로 끝난다.
- p12는 `3 중앙선거관리위원회`로 시작한다.
- p13 이후와 p15-p17의 기존 PDF 경계가 유지된다.
- rhwp/PDF 17/17, focused integration 15/15다.

증적:

- [Stage 56 분석·검증](../working/task_m100_3820_stage56_issue2007_p11_heading_owner.md)
- [Stage 57 exact-head 검증](../working/task_m100_3820_stage57_exact_head_pr_gate.md)
- [p11-p13 contact sheet](../pr/assets/task_m100_3820_stage57_exact_head_pr_gate/review_p011_p013_exact_head.png)
- [visual sweep 원장](../pr/assets/task_m100_3820_stage57_exact_head_pr_gate/visual_sweep_summary_exact_head.json)

## 4. 자동 검출 inventory

이 목록은 Stage 7 수정 전 역사적 inventory이며 현재 잔존 결함 수가 아니다.

다음은 **결함 확정 목록이 아니라** PDF 대조를 우선해야 할 자동 후보 목록이다. 같은 원인으로
연쇄된 페이지는 한 묶음으로 해석한다.

### 4.1 Raster/구조 visual sweep 후보: 58쪽, 64개 flag

| 규칙 | 후보 쪽 | 해석 우선순위 |
| --- | --- | --- |
| `column_text_flow_collapse` (46) | 7, 9, 28, 75, 77, 119, 134, 171–175, 177–179, 182–185, 187–199, 201, 203–215 | p119 및 p171–215는 D-01/D-03과 결합. 나머지는 독립 PDF review 필요 |
| `line_order_overlap` (3) | 118, 129, 181 | p118은 D-01과 결합; p129·181은 후보 |
| `frame_overflow_pixels` (3) | 161, 167, 204 | 테두리/표 외곽선으로도 발생할 수 있어 candidate-only |
| `content_bottom_drift` (1) | 167 | p167 frame 후보와 함께 review |
| `column_line_band_drift` (1) | 181 | p181 line-order 후보와 함께 review |
| `question_marker_flow_drift` (9) | 20, 24, 42, 47, 68, 174, 176, 182, 183 | 앞 5쪽은 독립 review, 뒤 4쪽은 D-03 연쇄 가능성 |
| `endnote_separator_gap_drift` (1) | 27 | 각주 separator 간격 review |

세부 flag와 candidate 수는
`output/task-3820-3821-fidelity/stage7-full-sweep/stage7-full-215/analysis/page_*.json`에 있다.

### 4.2 Layout ledger 후보

| 후보 | 건수 / 쪽 경계 | 의미 |
| --- | --- | --- |
| reciprocal text owner shift | 8 | 74→75, 90→91, 118→119, 120→121, 129→130, 131→132, 166→167, 168→169 |
| order-preserving text sequence shift | 6 | 74→75, 131→132, 168→169, 172→173 (3행) |
| 상단 float와 결합한 owner shift | 2 | 74→75, **118→119** |
| table fragment | 15 | 66→67, 76→77, 78→79, 90→91, 94→95, 106→107, 157→158, 160→161, 161→162, 163→164, 164→165, 167→168, 176→177, 190, 215 |

원장 파일은 `output/task-3820-3821-fidelity/stage6-full-ledger/` 아래의
`text-owner-shift-candidates.tsv`, `text-owner-sequence-candidates.tsv`,
`float-owner-shift-candidates.tsv`, `table-fragment-candidates.tsv`다. table fragment는 동일
표의 인접 페이지 fragment를 찾는 규칙일 뿐, 행의 소유 페이지가 PDF와 다르다는 자동 확정은 아니다.

## 5. 자동 판정의 한계와 다음 수정 순서

1. 정책 문서 215/215와 issue2007 17/17은 page-owner 회귀 계약으로 유지하되, 전체 시각
   fidelity 완료 판정으로 확대하지 않는다.
2. 후속 #3820 후보는 자동 점수만으로 확정하지 않고 한컴 PDF raster와 페이지별로 직접 대조한다.
3. 수정은 분석 문서 → 코드 → focused PDF review → 영향 범위 재검사 순으로 분리한다.

## 6. 해소로 재분류한 과거 항목

이번 전수 run에서 p108 TIFF 그림 미출력과 p156 Square 그림 여백은 현재 우선 결함으로 분류하지
않았다. 이들은 이전 stage의 focused PDF review에서 각각 PNG 변환 및 outer-margin 보정 후 정상
확인된 항목이다. 다만 이후 regression은 위 전수 기준선을 다시 실행해 판정한다.

## 7. 검증 도구 상태

이번 inventory를 생성한 `scripts/visual_sweep.py`와 `tools/fidelity_compare/fidelity_compare.py`는
이미 p118→p119 owner shift를 상단 float와 연결하는 triage를 갖고 있고, Stage 9는 deferred Square
page-top offset drift도 공통 후보로 추가했다. 관련 Python 회귀는 다음으로 확인했다.

```text
python3 -m py_compile tools/fidelity_compare/fidelity_compare.py
python3 -m unittest scripts/tests/test_fidelity_compare.py scripts/tests/test_visual_sweep.py
# Ran 59 tests ... OK
```

수정 전 p127 geometry의 재발은 후보화한다. Stage 11에서 D-03의 실제 page-owner drift는
해소됐지만, 검출기 회귀 통과와 page count 정합만으로 이 215쪽 문서의 모든 layout fidelity가
보증되는 것은 아니다.
