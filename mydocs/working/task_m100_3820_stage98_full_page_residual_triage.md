---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 98 — 전체 페이지 잔여 fidelity 재탐색

## 목적

Stage 96--97의 86712 p28--p29와 76076 p81--p82 집중 보정만으로 전체 문서 정합을
완료했다고 간주하지 않는다. 현재 `task/3820-production-fidelity` HEAD `0c2da6a2a`에서
독립 한컴 기준 PDF가 있는 실물 문서를 다시 전수 조사하고, 자동 후보와 페이지별 직접
PDF 판정을 결합해 다음 잔여 결함을 고른다.

대상은 다음과 같다.

1. `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
   ↔ `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
2. `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
   ↔ `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
3. `samples/76076_regulatory_analysis.hwp`와 `samples/86712_regulatory_analysis.hwp`의
   이미 보정한 경계 및 자동 회귀 원장

## 시작 상태

- 현재 브랜치: `task/3820-production-fidelity`
- 시작 HEAD: `0c2da6a2a`
- `rhwp info` 페이지 수:
  - 정책연구 문서: rhwp 215 / 기준 PDF 215
  - issue2007: rhwp 17 / 기준 PDF 17
- 작업트리의 `task_m100_3820_stage96_2279_nested_fragment_oracle.md` 수정은 다른 작업의
  진행 기록이므로 이 stage에서 편집·stage·되돌리기 하지 않는다.
- 검증 바이너리: `target/pr-review/release-test/rhwp`, SHA-256
  `641b6b037f04c4517e25e6da0c0716fb30662e4e3852b200969af46fd832d7ba`

## 판정 절차

1. `fidelity_compare.py --text-only --export-all-svg --layout-ledger`로 전체 페이지의
   text owner, 표 fragment, clip, overlap, page-count 원장을 만든다.
2. 자동 후보와 낮은 pixel 일치 페이지를 합쳐 실제 PDF/SVG를 직접 판정한다.
3. 실제 결함 한 종류만 source→IR→layout→paint 경로로 좁혀 일반 구조 조건으로 고친다.
4. focused 회귀와 전체 문서 전수 판정을 반복한다. `flagged=0`이나 페이지 수 일치만으로
   완료 판정을 내리지 않는다.

## 현재 실행 중인 원장

- `output/task-3820-stage98-policy-current-ledger/`: 정책연구 215쪽 전수
- `output/task-3820-stage98-issue2007-current-ledger/`: issue2007 17쪽 전수

issue2007의 text/layout 원장은 17/17쪽을 완료했다. 페이지 수는 17/17/17(PDF/SVG/render
tree)이며, 구조 원장에서는 p9의 table/footer 후보 2건을 우선 검토 대상으로 올렸다.
텍스트 차이는 p13--p14 경계의 소유 이동 가능성과 p2/p4 raw PUA를 포함하므로 pixel 비교와
PDF 직접 판정 없이 결함으로 확정하지 않는다.

## 다음 단계

- issue2007 전수 pixel 비교에서 p9, p13, p14 및 최저 일치 페이지를 직접 연다.
- 정책연구 215쪽 원장이 끝나면 owner/table/overlap 후보의 교집합을 우선순위화한다.
- 첫 확정 결함의 원인 분석과 코드 수정·회귀는 이 문서에 이어 기록하고 커밋한다.

## 첫 확정 결함 — fidelity 하네스의 font-style 누락

issue2007 17쪽 pixel pass에서 p10--p15 diff가 24--30%로 올라왔지만, 비교 PNG의 rhwp 쪽
한국어가 모두 두부(□)였다. 같은 source의 `visual_sweep.py`는 `export-svg --font-style`을
사용하는 반면 `fidelity_compare.py`의 단일/전체 SVG export 두 경로는 이 옵션을 빠뜨리고
있었다. 따라서 Stage 96에서 마련한 `한양중고딕`/`휴먼명조` local fallback 별칭이 SVG에
들어가지 않았고, 실제 layout 결함보다 하네스의 글꼴 실패가 pixel ranking을 지배했다.

`render_svg`와 `render_all_svg` 모두 `--font-style`을 사용하도록 보정하고, 두 호출 경로의
명령 계약을 Python 회귀로 고정한다. 기존 SVG cache를 제거한 새 output에서 issue2007 전수를
다시 생성해 두부가 사라졌는지와 실제 페이지 경계 차이를 분리한다.

첫 재실행에서 p8의 돋움 계열은 정상 글리프로 바뀌었지만 p11--p15의 주 글꼴
`휴먼명조`는 계속 두부였다. 해당 host의 `HMKMM.TTF`는 EBDT bitmap table을 가진 legacy
TrueType이고, Chrome은 이 local face를 선택한 뒤 표준 한글을 `.notdef`로 그렸다. 이는
fallback chain 자체가 없는 문제가 아니라 깨진 local face가 정상 outline fallback보다 먼저
선택되는 문제다.

SVG는 glyph 좌표가 이미 확정돼 있으므로 `휴먼명조`와 `한양신명조`의 `@font-face src`에서
Batang/바탕, AppleMyungjo, Noto Serif CJK KR outline 대체를 먼저 두고 legacy local face를
마지막으로 내린다. native Canvas 조판 메트릭이나 문서 IR은 바꾸지 않는다.

## Stage 98 검증과 인계

- `venv/bin/python -m unittest tools.fidelity_compare.test_fidelity_compare`: 47/47 성공
- `cargo fmt --check`: 성공
- `cargo test --profile release-test --lib
  renderer::svg::tests::legacy_hanyang_faces_have_portable_local_aliases`: 1/1 성공
- 정책연구 문서는 215/215쪽 text/layout 원장을 완료했고, p74→p75에서 PDF보다 본문이
  한 쪽 일찍 배치되는 실제 owner 차이를 직접 확인했다.
- 정책연구 자동 후보 19쪽의 144dpi review를 생성했으며, 자동 `flagged=0`과 별개로 p74/p75
  직접 판정은 실패다. 따라서 자동 지표만으로 완료 처리하지 않는다.

현재 작업트리에 별도 #4138 조사 변경(`height_measurer.rs`, Stage 96 문서)이 들어와 있어
이 stage 커밋과 검증에서 제외한다. 이 커밋을 깨끗한 detached worktree에서 다시 빌드한 뒤
issue2007 p11 및 17쪽 전수를 새 output 경로로 렌더해 두부 제거를 확인한다. 이후 Stage 99는
정책연구 p74→p75의 body/footnote 예약과 paragraph owner를 분석한다.
