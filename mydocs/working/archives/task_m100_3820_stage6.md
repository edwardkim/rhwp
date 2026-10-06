---
kind: analysis
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-04
---

# Task #3820·#3821 Stage 6 — p118→p119 그림 앞 문단 owner drift 자동 후보화 분석

## 사용자-visible 재현과 정답 기준

같은 `정책연구용역사업 중간진도보고서`의 사용자 쪽번호 118→119 경계에서 rhwp는 그림 앞
본문의 마지막 절을 p118에 계속 남기고 p119에서 절차 그림을 시작한다. 한컴 2020 기준 PDF는
그 절의 뒷부분(`기록되어야 함. 동의 취득 회의록은 …`)을 p119 상단에 먼저 배치한 뒤 같은
절차 그림을 둔다. 즉 그림 자체의 누락이나 그림 위 문자 충돌이 아니라, **TopAndBottom 그림
앞에서 본문 paragraph owner가 한 페이지 이르게 확정되는 page-boundary fidelity 결함**이다.

비교 기준은 `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적
선별기준 연구)-2020.pdf`의 사용자 p118/p119이고, 입력은 동명의 `samples/*.hwp`다. fidelity
도구에는 0-based page 117/118로 넘긴다.

## 기존 자동 판정의 예상 범위

Stage 4의 `square_wrap_text_overlap`은 그림의 물리 box를 본문이 가로지르거나 edge에 맞닿는
경우만 다룬다. 이번 증상은 서로 다른 물리 페이지의 text owner와 TopAndBottom 그림 순서가
어긋난 것이므로 이 규칙으로는 찾을 수 없다.

반면 `fidelity_compare --text-only --export-all-svg --layout-ledger`는 PDF↔SVG 인접 페이지의
reciprocal text difference와 16자 이상 순서 보존 문자열을 `text-owner-*-candidates.tsv`에
기록한다. 먼저 이 기존 ledger가 p118→p119의 문단 절 이동을 실제로 후보화하는지 확인한다.

## 수용 기준과 다음 단계

1. direct-pair text-only 전수 export에서 p118→p119에 `rhwp_earlier_than_reference` owner
   후보와 이동한 실제 본문 문자열이 남는지 확인한다.
2. 기존 owner candidate가 있더라도, 인접 page의 Body text movement와 successor-page
   `TopAndBottom`/`Square`/`Tight`/`Through` 그림을 결합한 `float_owner_shift` **triage 행**을
   fidelity ledger에 추가한다. 이는 generic owner detector의 중복이 아니라, 그림 존재만으로
   결함으로 판정하지 않고 PDF owner 차이가 함께 있을 때만 그림 원인을 함께 보이게 하는 연결이다.
3. 후보는 PDF 시각 review를 요구하는 triage 신호이며, 자동 불합격·전역 page-break 보정의
   근거로 사용하지 않는다.

이 분석 문서를 커밋한 뒤에만 fidelity 도구·test·사용 문서를 수정한다.

## 기존 원장 재현 결과와 보완 근거

현재 binary로 다음 direct pair를 실행했다.

```text
RHWP_BIN=target/task-3820-3821-fidelity/release-test/rhwp \
python3 tools/fidelity_compare/fidelity_compare.py 117 118 \
  --source 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --reference-pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --label stage6-p118-owner --reference-grade '한컴 2020 기준 PDF' \
  --text-only --export-all-svg --layout-ledger \
  --out-dir output/task-3820-3821-fidelity/stage6-owner
```

`text-owner-shift-candidates.tsv`에는 user p118→p119의
`rhwp_earlier_than_reference`, `shared_chars=72`, `source_coverage=1.000`,
`target_coverage=1.000`이 실제로 기록됐다. 따라서 owner detector 자체는 이 결함을 이미
놓치지 않는다. 같은 render tree의 p119에는 Body `Image(pi=1276, ci=0, textWrap=TopAndBottom,
bbox=94.5,83.2,448.5,359.0)`가 있다.

그러나 이 둘은 서로 다른 TSV에 있어, 215쪽 전수 결과에서 사용자가 owner 이동이 그림 앞
문단의 분할 결함임을 다시 교차해석해야 한다. `float-owner-shift-candidates.tsv`는 이 정확한
두 근거를 한 행으로 연결해 review 우선순위를 올린다. 페이지 상단 25% 안의 Body float만
연결하므로, 같은 페이지에 우연히 있는 하단 그림으로 owner shift를 과장하지 않는다.

## 구현 및 검증 결과

`fidelity_compare`에 `float-owner-shift-candidates.tsv`를 추가했다. 이 원장은 기존
`text-owner-shift-candidates.tsv`의 `rhwp_earlier_than_reference` 행을 재사용하고, 다음 물리
페이지의 Body `TopAndBottom`/`Square`/`Tight`/`Through` 그림 중 페이지 상단 25% 안에 놓인
80px 이상 그림만 결합한다. 따라서 그림만 있거나 일반 문단 owner shift만 있는 경우는 후보로
기록하지 않으며, PDF visual review가 여전히 최종 판정이다.

실제 p118→p119 재실행 결과는 다음 한 행이다.

```text
118  119  rhwp_earlier_than_reference  72  1.000  1.000
pi=1276  ci=0  TopAndBottom  bbox=94.5,83.2,448.5,359.0  top_ratio=0.074
```

이제 후보 한 행만으로 “p118에서 rhwp가 본문 72자를 너무 이르게 소유했고, p119 상단에
그림 55가 있다”는 조사 우선순위가 드러난다. `scripts/visual_sweep.py`의 독립 raster 규칙도
같은 pair에서 p118 `line_order_overlap`, p119 `column_text_flow_collapse`을 flag했다.
두 도구가 같은 결함을 다른 축에서 후보화한 것이며, screenshot의 PDF 대조가 결함 확정 근거다.

215쪽 full text-only/layout ledger도 완료했다. SVG와 render tree는 219쪽, 기준 PDF는 215쪽으로
전역 page-count delta `+4`를 별도 기록했고, requested/completed는 215/215·missing 0이다.
generic reciprocal owner 후보 8건 중 상단 float와 결합한 고신호 후보는 2건(p74→p75,
p118→p119)으로 줄었다. 이 원장은 전체 문서의 자동 우선순위 큐이며, 두 건이 모두 결함이라는
자동 판정은 아니다.

```text
python3 -m py_compile tools/fidelity_compare/fidelity_compare.py
python3 -m unittest scripts/tests/test_fidelity_compare.py scripts/tests/test_visual_sweep.py
# Ran 55 tests ... OK
```

증적은 다음 경로에 보관했다.

- `output/task-3820-3821-fidelity/stage6-owner/float-owner-shift-candidates.tsv`
- `output/task-3820-3821-fidelity/stage6-sweep/stage6-p118-owner/review/review_118.png`
- `output/task-3820-3821-fidelity/stage6-sweep/stage6-p118-owner/review/review_119.png`
- `output/task-3820-3821-fidelity/stage6-full-ledger/float-owner-shift-candidates.tsv`
