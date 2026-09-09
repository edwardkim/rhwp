---
kind: investigation
status: active
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-05
---

# Task #3738 Stage 32 — 최신 HWP/PDF 잔존 결함 재판정과 sweep 정밀화

## 출발 근거

현재 native release-test binary에서 개인정보 제거 HWP와 한컴 2020 기준 PDF의 전체 페이지 수는
모두 215쪽이다. 과거 Stage의 원장은 코드 진전 전 기준이라, 후보 목록을 그대로 잔존 결함으로
승격하지 않고 최신 binary와 PDF를 다시 대조한다.

`fidelity_compare --text-only --export-all-svg --layout-ledger`는 p90→p91에 text-owner 후보를
남긴다. 그러나 이는 표 row owner의 결함을 뜻하지 않는다. 현재 source `(pi=962, ci=0)` 7×6
RowBreak 표는 PDF와 같이 p90에 `이식대상자와 관계`, p91에 `기타`로 재개한다. PDF/SVG raster,
focused Rust regression, p90 FootnoteArea separator 상한까지 모두 통과했다. 따라서 이 후보는
텍스트 추출층 차이이며 renderer 수정 대상으로 삼지 않는다.

## 조사·수정 계약

1. 이전에 보고된 p90–91, p94–95, p106–108, p127을 최신 PDF/SVG raster와 HWP source/render tree로
   재검증한다. 예전 screenshot이나 text ledger 하나만으로는 결함으로 판정하지 않는다.
2. SVG export의 `LAYOUT_OVERFLOW`와 fidelity layout candidate도 PDF raster에서 실제 소실·겹침이
   보일 때만 renderer 결함으로 승격한다.
3. 전체 215쪽은 visual sweep으로 후보를 자동 수집하되, float 본문 붕괴 신호는 render tree의
   Square/Tight/Through float가 있는 페이지에서만 강한 후보로 낸다.
4. 목차의 우측 페이지번호 tab rail처럼 가상 반쪽 raster 분할이 만드는 false positive는
   `column_text_flow_collapse`로 보고하지 않는다. p127의 실제 Square 그림 본문 붕괴 검출 능력은
   유지하는 focused unit test로 고정한다.
5. 새로 확인되는 PDF-visible 결함만 다음 Stage에서 source→IR→layout→paint 원인 분석과 수정으로
   분리한다.

## 최신 재판정

| 범위 | 최신 PDF/SVG 대조 | 결론 |
| --- | --- | --- |
| p90–91, 표 27 | p90 relationship row, p91 `기타`, separator non-overlap focused regression 통과 | 해결됨; text-owner 후보는 false positive |
| p94–95, 표 28 | 마지막 row와 caption의 페이지 경계가 PDF와 동일 | 해결됨 |
| p106–108, 표 29·그림 52 | 표 조각, 페이지 번호, 그림 owner가 PDF와 동일 | 해결됨 |
| p127, 그림 56 | Square 그림과 본문·각주가 PDF와 동일; 세로 text collapse 없음 | 해결됨 |
| p7, 목차 | PDF/SVG가 정상이나 이전 sweep은 우측 page-number rail을 collapse로 flag | sweep false positive 수정 |
| p30–31, 각주 29 | PDF/SVG에서 p30 각주 29와 p31 본문 재개가 동일 | Chrome 794px raster가 centered 각주 구분선을 page bottom으로 오인한 sweep false positive 수정 |
| p94, 표 28 | 표·caption 페이지 경계는 PDF와 동일 | PDF/RHWP page-number footer antialias bleed 3px을 tail overflow로 남긴 sweep false positive 수정 |
| p115, 각주 152 | PDF/SVG의 URL·페이지 번호 위치가 동일 | 넓은 footer rule을 page bottom으로 오인한 sweep false positive 수정 |
| p129, 각주 175 | 본문·각주가 PDF와 동일하게 분리 | Body와 FootnoteArea를 같은 text flow로 비교한 sweep false positive 수정 |
| p140, 그림 61·62 | 두 TopAndBottom 그림과 caption·본문이 PDF와 동일 | non-reflow 그림의 raster band 차이를 text collapse로 승격한 sweep false positive 수정 |
| p157, 표 37 | 표·본문·페이지 번호가 PDF와 동일 | content area 91% 지점의 넓은 표 하단선을 page bottom으로 오인한 sweep false positive 수정 |
| p160–161, p164 | 표/본문과 페이지 번호가 PDF와 동일 | 같은 content-area border frame false positive가 반복된 범위를 재실행해 해결 확인 |
| p169, 그림 65 | 표·그림이 PDF와 동일 | 색상 SmartArt의 raster red band를 문항 marker 흐름으로 오인한 sweep false positive 수정 |
| p178, 각주 236 | 본문·각주가 PDF와 동일하게 분리 | p129와 같은 Body↔FootnoteArea line-order false positive가 제거됨 |

## Stage 32 변경

`scripts/visual_sweep.py`의 `column_text_flow_collapse` 판정은 이제 render tree에
`Square`/`Tight`/`Through` Image가 있을 때만 활성화한다. 이는 p127처럼 실제로 본문 폭을
바꿀 수 있는 float를 계속 감시하면서, 단일 column 목차의 tab 정렬 숫자를 가짜 두 번째 본문
column으로 오인하지 않게 한다. 또 `detect_frame`의 bottom frame은 60% 이상을 가로지르면서
physical footer 94% 이후에 있는 rule만 채택해, p30처럼 가운데 각주 separator나 p157의 표
하단선이 page frame을 짧게 만들고 이후 bottom-flow 지표 전체를 오염시키는 경우를 막는다. footer가
양쪽 raster에서 같은 위치에 있고 antialias
bleed만 최대 4px 다른 경우도 page-number tail overflow에서 제외한다. `scripts/tests/test_visual_sweep.py`에
p7과 같은 count/drift를 float 부재 조건에서 억제하는 회귀, wrap mode tree 검사, 794px Chrome raster의
centered footnote separator 및 page-number footer 회귀를 추가했다. 마지막으로 render tree의 document
순서가 Body 마지막 줄 뒤에 FootnoteArea 첫 줄을 두는 경우를 하나의 본문 flow로 비교하지 않도록 top-level
container 경계를 보존한다. 같은 container 안의 실제 text-line overlap은 계속 후보로 남긴다. 또한
question-marker 흐름 후보는 raster red/ink 조건만으로 올리지 않고, render tree/PDF의 실제 `문N` marker
drift가 함께 있을 때만 활성화한다.

focused 실행:

```bash
python3 -m unittest scripts/tests/test_visual_sweep.py
python3 scripts/visual_sweep.py --key issue3738-stage32-p7 \
  --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --rhwp-bin target/task-3820-3821-fidelity/release-test/rhwp --page 7
python3 scripts/visual_sweep.py --key issue3738-stage32-p30 \
  --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --rhwp-bin target/task-3820-3821-fidelity/release-test/rhwp --page 30
python3 scripts/visual_sweep.py --key issue3738-stage32-p94 \
  --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --rhwp-bin target/task-3820-3821-fidelity/release-test/rhwp --page 94
python3 scripts/visual_sweep.py --key issue3738-stage32-p115 \
  --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --rhwp-bin target/task-3820-3821-fidelity/release-test/rhwp --page 115
python3 scripts/visual_sweep.py --key issue3738-stage32-p129 \
  --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --rhwp-bin target/task-3820-3821-fidelity/release-test/rhwp --page 129
python3 scripts/visual_sweep.py --key issue3738-stage32-p140 \
  --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --rhwp-bin target/task-3820-3821-fidelity/release-test/rhwp --page 140
python3 scripts/visual_sweep.py --key issue3738-stage32-p157 \
  --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --rhwp-bin target/task-3820-3821-fidelity/release-test/rhwp --page 157
python3 scripts/visual_sweep.py --key issue3738-stage32-p160-164 \
  --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --rhwp-bin target/task-3820-3821-fidelity/release-test/rhwp --pages 160-161,164
python3 scripts/visual_sweep.py --key issue3738-stage32-p167-169 \
  --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --rhwp-bin target/task-3820-3821-fidelity/release-test/rhwp --pages 167,169
python3 scripts/visual_sweep.py --key issue3738-stage32-p178 \
  --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --rhwp-bin target/task-3820-3821-fidelity/release-test/rhwp --page 178
```

결과는 33 Python tests 통과, p7·p30·p94·p115·p129·p140·p157 모두 `flagged_page_count=0`이다. p94의 page-number는
`page_number_footer_bleed`로 명시적으로 suppressed되고 p115 frame은 PDF/RHWP 모두 default page
bottom으로 복원된다. p129의 Body↔FootnoteArea pair도 더는 line-order 후보가 아니며 p140은
`TopAndBottom` 그림만 있어 reflow float 조건을 충족하지 않는다. p160–161·p164도 동일한 frame 보정 후
무플래그다. p167·p169도 frame/SmartArt marker 보정 후 무플래그다. 전체 215쪽 sweep은 최신 binary로 별도
실행해 새 PDF-visible 후보만 후속 Stage로 이월한다. p178은 p129와 같은 container-boundary 후보였고 재실행에서
무플래그다.
