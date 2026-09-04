---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 20 — 215쪽 PDF 대조 후보 자동 추출

## 문제

기준 한컴 PDF는 215쪽이고 현재 native HWP 산출은 220쪽이다. p43, p44–45, p52–53, p66–67,
p83–85, p90, p94, p106, p107–108처럼 physical page owner·표 fragment·각주 reservation이 달라지는
결함은 한 장씩 눈으로 스크롤해 찾는 방식으로는 누락과 재확인이 반복된다. 이 단계의 목표는 **자동 후보
추출**이며, 픽셀 수치만으로 결함/해결을 선언하지 않는다.

## `cli_commands.md`에서 확인한 빠른 신호

| 신호 | 명령/도구 | 잡는 결함 | 한계 | Stage 20 용도 |
| --- | --- | --- | --- | --- |
| 기준 PDF text ↔ SVG text multiset | `tools/fidelity_compare/fidelity_compare.py` | 각주·본문·caption의 누락/과잉, physical owner 이동 | 순서·좌표·같은 문자 수의 위치 변화는 모름 | **전 215쪽 1차 순위** |
| 저해상도 PDF ↔ SVG pixel diff | `fidelity_compare.py` | 사진/표 fragment/줄바꿈/페이지 경계의 큰 차이 | 폰트 raster 차이도 높게 나옴 | text 후보와 합집합으로 2차 순위 |
| cell clipping 원장 | `rhwp export-svg <HWP> --json`의 `overflowCellLines` | 표 셀 내용이 쪽 하단에서 clip되는 결함 | 각주·그림/일반 문단 owner는 모름 | 0이 아닌 쪽을 P0 후보에 추가 |
| bbox ledger | `rhwp export-render-tree <HWP>` | body/footnote/footer overlap, 표·그림 frame 밖 이탈 | 한컴 PDF의 정답 page owner 비교는 못 함 | 자동 기하 후보에 추가 |
| self geometry diff | `rhwp render-diff` | rhwp roundtrip 내부 회귀 | 한컴 PDF와 직접 비교가 아님 | 외부 fidelity 후보 탐지에는 사용하지 않음 |

`render-diff`는 자기 일관성 게이트여서 이번의 한컴 PDF 기준 page owner 차이를 발견하는 도구가 아니다.
따라서 visual sweep을 대체하는 단일 수치로 오용하지 않고, candidate discovery 후 selected visual sweep을
최종 판정으로 유지한다.

## 기존 도구의 즉시 사용 가능 범위와 공백

`tools/fidelity_compare/fidelity_compare.py`는 이미 PDF raster diff와 text report를 만들지만, positional
`REG` 키(`plan`, `manual`, `bunjang`, `korexam`, `math`, `eng`)만 받는다. 이번 기준 쌍은 아래와 같이
등록되어 있지 않아 직접 실행할 수 없다.

- HWP: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 기준 PDF: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`

또한 기존 도구는 범위의 각 페이지에 Chrome raster와 비교 sheet까지 생성한다. 215쪽의 첫 진단에서
그 전체 raster sheet를 만들 필요는 없다. 먼저 SVG/PDF text report와 `export-svg --json`/render-tree
ledger를 한 번 생성하고, ranking 상위·기하 flag 페이지에 한해 low-resolution raster와 high-DPI
`visual_sweep.py`를 실행해야 한다.

## 구현 전 계약

1. 기존 `<등록 키> <시작 0-based> <끝 0-based>` 명령은 호환한다.
2. direct pair는 `--source`, `--reference-pdf`, `--label`, `--reference-grade`를 함께 받으며,
   등록 fixture와 섞지 않는다. provenance에는 절대 경로·등급을 기록한다.
3. `--text-only`는 Chrome/PNG/sheet를 실행하지 않고 SVG text와 PDF text report만 만든다. 215쪽
   첫 pass의 빠른 candidate discovery 용도다. 이 환경에 있는 `pypdf`를 사용하므로 `pypdfium2`·Chrome은
   필요하지 않다.
4. `--export-all-svg`는 `export-svg`를 한 번만 실행해 SVG cache를 채운다. 215쪽 전수에서
   페이지마다 rhwp를 재기동하지 않는다. `--text-only --export-all-svg` 결과 디렉터리는 다음 shard/raster
   pass가 재사용한다.
5. raster diff는 그 다음 단계에서 같은 `--out-dir` cache와 지정 범위를 재사용한다. 후보 외 페이지의
   comparison sheet는 만들지 않는다.
6. `--layout-ledger`는 `export-render-tree` 한 번으로 Body TextLine↔FootnoteArea, Body Table↔Footer,
   Body 표/그림 page-frame 이탈을 `layout-candidates.tsv`에 적는다. 1px stroke/반올림은 noise로 허용하며,
   자동 후보를 곧바로 결함으로 선언하지 않는다.
7. 결과 report에는 requested/completed/missing page를 적어 중단된 run을 완료로 가장하지 않는다.

## 실행 순서

1. 위 direct-pair·text-only·SVG cache 계약을 기존 tool에 최소 확장하고 `--help`와 작은 범위에서
   backward/direct parse를 검증한다.
2. native release-test `rhwp`로 0–214 text-only pass를 한 번 실행한다. 이 단계는 수동 215쪽
   inspection이 아니라 candidate ledger 생성이다.
3. 같은 SVG cache에서 `export-svg --json`과 render-tree bbox ledger를 계산해 cell clipping/overlap/
   page-frame escape 후보를 합친다.
4. text 또는 geometry 후보와 pixel-rank 후보의 합집합만 `visual_sweep.py --pages ...`로
   PDF와 고해상도 review PNG를 만든다.
5. PDF screenshot/source/render tree로 원인군을 확정한 뒤에만 focused regression과 코드 수정 단계로
   넘어간다.

## 이번 단계에서 이미 고정된 결함

Stage 19에서 확인한 미해결 목록은 [Stage 19 기록](task_m100_3738_stage19.md)을 기준으로 삼는다. 자동
rank가 그 목록에 없는 페이지를 내더라도 신규 후보로만 표시하며, 사람이 PDF review를 끝내기 전 결함으로
승격하지 않는다. Stage 19의 p25 그림 25 누락은 commit `24a723029`에서 해결됐으므로 candidate ledger의
baseline으로 되돌리지 않는다.

## Stage 20 종료

direct pair·`--text-only`·`--export-all-svg`·`--layout-ledger` 구현과 215쪽 complete run 결과는
[Stage 20 candidate ledger](task_m100_3738_stage20_candidate_ledger.md)에 고정했다. 이 Stage는 후보를
자동 수집하는 기반을 만든 것이며, 기존 P0 결함이나 native 220쪽/PDF 215쪽 차이를 해결했다고 주장하지 않는다.
다음 Stage는 `overflowCellLines=26`과 page frame을 벗어난 표를 보인 p157–158의 source/fragment를
분석하고, 이 큰 page-map 분기와 개별 P0 owner 결함의 선후 관계를 판정한다.
