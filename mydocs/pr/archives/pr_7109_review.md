---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7109 저장 세로 위치가 없는 여러 줄 셀 검토

**판정: 승인 — 사용자 제공 한컴 PDF 직접 대조로 독립 시각 증거 보류 해소.** 로컬 체리픽 통합본에 대한 검토다. 원 PR merge/close 또는 GitHub approve를 완료했다는 의미가 아니다.

## 출처와 통합 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#7109](https://github.com/edwardkim/rhwp/pull/7109) / LJYeon12 |
| 원 head | `191fa0046337dc8bde7d4ed396960f625ee370e7` |
| 최신 devel 시작점 | `410d22cdf77e3e9e45159999cab19719c621c025` |
| 통합 code head | `b40c2953cf526b86319f2841c5b5584d7a83d735` |
| 작업 branch | `codex/non-draft-integration-20260914` |
| reviewer | jangster77, 검토 전 요청 등록 확인 |
| source → local | `30c9314e3` → `a161bd646`; `1ff3c0388` → `a4054879d`; `191fa0046` → `09836e4f7` |

사용자 지시로 #7141·#7142를 제외했다. draft #6670·#7118도 제외했다. 현재 대상은 #7099·#7109·#7134·#7136·#7137·#7139의 6개 PR, 원 commit 9개다. 원 작성자와 cherry-pick -x 출처를 보존했다.

## 코드와 증거 판독

`cell_vpos_ladder_is_intact`가 서로 다른 연속 두 줄의 vpos=0을 유효한 저장 사다리에서 제외한다. 세로 정렬과 표 축소 하한이 같은 helper를 사용하므로 실제 배치의 두 줄 높이와 측정 결과를 맞춘다. 같은 text_start 중복, 양수→0 리셋, 페이지·단 첫 줄, 기존 가로 조각 판정을 보존한다. 특정 문서 ID나 임의 보정 상수는 없다.

원본 CI의 신규 테스트는 위/가운데/아래 정렬·두 셀 높이·HWP roundtrip·여유 행 축소·중복/리셋 경계를 다룬다. 이 검토에서 해당 회귀를 다시 실행한 것은 아니다.

공개 입력 `tests/fixtures/multiline_cell_zero_positions/two_lines.hwpx`는 rhwp 합성 문서다. 1000 HU 두 줄 + 위아래 140 HU로 2280 HU가 필요하다는 계약은 입력 치수에서 독립적으로 설명된다. 이것을 한컴 생성 문서라고 부르지 않는다. contributor의 비공개 실제 문서 2~4쪽 개선 수치와 이미지는 검토자가 접근하지 못해 재검증하지 않았다.

이전 MCP 변환은 900초 timeout으로 실패했다. 이후 사용자가 알려 준 기존 한컴 PDF를 독립 기준으로 재사용했다. 아래 PDF 메타데이터·전후 Visual Sweep 판정으로 시각 증거 보류를 해소한다. 실패한 MCP job이 성공했다고 기록하지 않는다.

## CI와 검증 범위

조회한 원 PR head는 위 source SHA와 일치하고, CI에 실패·대기 항목이 없다. skipped/neutral을 실제 검사 통과 수로 합산하지 않는다. [Lint (fmt, clippy, WASM check): SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/34767492226/job/103841709976) · [Frontend package gates: SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/34767492226/job/103841709975) · [Build & Test: SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/34767492226/job/103844440022).

이 통합 head에는 별도 GitHub CI가 아직 없다. 원 PR들의 녹색 상태를 새 통합 head의 CI 성공으로 대신하지 않는다. 사용자 지시에 따라 완료된 CI 전체 회귀를 로컬에서 반복하지 않았다. macOS 전용 `CARGO_TARGET_DIR=target/nondraft-review-20260914`의 native build를 새 head에서 완료했고, fresh WASM web 빌드도 성공했다(host `--no-opt`, 5분 12초). Docker/wasm-opt 표준 빌드 및 성능 검증 성공으로 확대하지 않는다. [WASM 빌드 로그](../assets/non_draft_20260914_wasm-build-six-pr.log.txt). PR 생성 전 fmt·native/WASM32/workspace all-target Clippy·workspace build·suite manifest·unit tier 검사를 모두 통과했다. [명령·exit code·로그](../assets/non_draft_20260914_pre_pr_validation.json). 검증 head는 `9ac32f05b`이며 이후 변경은 검토 기록·오늘할일·로그뿐이다. 통합 GitHub CI는 PR 생성 후 확인한다.

## 공통 조판 원칙 준수

| 항목 | 판정·범위 |
| --- | --- |
| 독립 근거·원인 계층 | 위 코드/입력 계약에 근거. contributor 주장과 직접 관찰을 구분 |
| 문서별 예외·좌표 clamp | 이번 수용 코드에 문서 ID 분기나 픽셀 맞춤 상수 추가 없음 |
| 측정·배치 공통 결과 | #7109는 공통 사다리 helper. #7099는 devel 유지. 나머지는 레이아웃 산식 변경 비해당 |
| 줄 소속·높이 | 변경 범위와 반례는 본문 참조. 모든 중첩 표·모든 문서 보장으로 확대하지 않음 |
| golden·허용치 | 기존 baseline을 완화하지 않음. #7099 원 golden 변경은 수용하지 않음 |
| 실제 시각 증거 | 실행한 Visual Sweep/Studio만 기록. 미공개 문서/미실행 호스트는 미검증 |
| 성능·전체 fidelity | 이번 검토에서 측정·보증하지 않음 |

입력 SHA-256·Git blob 일치·source CI·원 commit 매핑은 [공통 증적](../assets/non_draft_20260914_evidence.json)에 있다. 이미 Git에 있는 HWP/HWPX/PDF를 이름만 바꿔 중복 추가하지 않았다.

## 최종 통합 이후 comment 계획

통합 PR이 승인·CI 완료·merge된 뒤 원 PR에 구현 출처, 실제 수용 범위, 보정 사유, 검증 결과와 미검증 범위를 설명한다. Visual Sweep가 적용된 PR에는 [사용법](../../manual/verification/visual_sweep_guide.md)과 확정 merge SHA의 증적 링크를 포함한다. 원 PR close는 구현 반영을 확인한 뒤 진행하고 contributor fork branch는 보존한다. 현재 원격 comment/close/통합 PR 생성은 하지 않았다.

## 합성 입력 실측과 이전 보류 이력

Visual Sweep의 WASM exporter와 공통 webfont rasterizer로 동일 합성 입력의 전후를 직접 캡처했다. [이전](../assets/pr7109_two_lines_before.png)에서는 두 번째 줄 아래가 셀 clip에 잘리고, [통합](../assets/pr7109_two_lines_wasm.png)에서는 두 줄이 보인다. 96 DPI tree의 셀은 y=69.3, h=30.4로 동일하다. 두 줄 y가 77.9/91.2 → 71.2/84.5px로 바뀌어 두 번째 줄 하단이 104.5 → 97.8px, 셀 아래 경계 99.7px 안으로 들어왔다. 값은 exporter 소수점 1자리 반올림이다. 이는 합성 입력의 결함 해소 근거다.

한컴 MCP job `a5a3a97b-6f43-404b-b105-4a1dbff3b3f5`는 engine 2020, timeout_seconds 900으로 실행했다. status에서 converting 진행을 여러 차례 확인했고, 최종 `failed` / `Hancom 2020 direct conversion exceeded 900 seconds.` / output_bytes=0을 확인했다. [최종 상태](../assets/pr7109_hancom_conversion.json). 이 job에서는 PDF가 생성되지 않았다. 이후 제공된 PDF는 이 실패 job의 산출물로 취급하지 않는다. 입력 생성기 문제인지 변환 worker 문제인지는 이 timeout만으로 판정할 수 없다.

이전 보류는 코드의 신규 회귀 발견이 아니라 독립 PDF 증거 부족이었다. 아래 직접 대조로 공개 재현의 두 줄 정렬·잘림에 관한 보류 조건을 해소했다. 별도 축소 변형·중복/리셋·HWP roundtrip은 원 head의 CI 근거이며, 이 한 장의 PDF가 모든 변형을 독립 검증했다고 확대하지 않는다.

## 사용자 제공 한컴 PDF 재검토와 최종 판정

사용자가 알려 준 `tests/fixtures/multiline_cell_zero_positions/two_lines-2020.pdf`를 그대로 사용했다. 파일명 `-2020`은 실제 출력 제품 버전의 증거가 아니다. `pdfinfo`는 **Creator: Hwp 2022 12.0.0.4605**, **Producer: Hancom PDF 1.3.0.550**, **PDF 1.4**, **1쪽**, **595×841 pt (A4)**, 생성 시각 **2026-09-14 20:58:52 KST**를 보여 준다. 이 메타데이터 조합은 유효한 한컴 기준본이다. HWPX 자체의 마지막 저장 메타데이터 `11.0.0.3524`(2020)와 PDF 출력 제품(2022)은 서로 다른 정보다.

PDF SHA-1: `fcaed0a1e2ab1e77ca61ab939da4809fd1676da0`.
PDF SHA-256: `34c7d6208c7831cdbdf82a11fab01c924426725c6a0e2bf17cb1a484ed2719b8`.
같은 크기·SHA-256의 기존 Git PDF가 없음을 확인했고, 사용자 제공 경로 그대로 이번 검토 보완 commit에 포함한다. `pdf/`에 사본을 만들거나 이름을 바꾸지 않는다. [PDF 원본](../../../tests/fixtures/multiline_cell_zero_positions/two_lines-2020.pdf).

### 직접 판독 결과

동일 HWPX에 대해 이전 WASM과 통합 WASM의 **1쪽 전체**를 새 PDF로 각각 Visual Sweep했다. 검토자가 compare/review/overlay와 PDF raster를 직접 열었다.

| 측정 대상 | 이전 WASM | 통합 WASM | 한컴 PDF |
| --- | ---: | ---: | ---: |
| First line 텍스트 bbox 상단, 96 DPI px | 77.9 | 71.2 | 71.11588 |
| Second line 텍스트 bbox 상단, 96 DPI px | 91.2 | 84.5 | 84.38138 |
| Second line bbox 하단, 96 DPI px | 104.5 | 97.8 | 97.64689 |
| 셀 bbox/대응 괘선 하단, 96 DPI px | 99.7 | 99.7 | 99.89067 |

PDF 텍스트 좌표는 `pdftotext -bbox-layout`의 pt를 96/72로 변환했고, PDF 셀 괘선은 content stream의 y=766.082pt를 `(841-y)×96/72`로 환산했다. WASM tree는 소수점 1자리로 반올림된 값이다. 텍스트 bbox 상단과 typographic baseline을 혼동하지 않는다.

이전 출력은 두 번째 줄 아래가 셀 clip에 잘렸고, 통합 출력은 한컴 PDF와 같이 두 줄이 셀 안에 온전히 보인다. 두 줄 상단 오차는 각각 약 **6.78/6.82px → 0.08/0.12px**로 줄었다. 기존 계약·golden·허용치는 수정하지 않았다.

전후 render tree 변경은 두 줄과 그 자식 테두리의 **y 12개 값**뿐이다. 셀/표 상자와 페이지 크기는 동일하다. PDF 인쇄 용지 A4와 HWPX의 480×480px 페이지, 주변 테두리·글꼴/농도 차이는 남으며 이 PR이 만든 변화가 아니다. 전체 페이지의 한컴 fidelity 일치로 판정하지 않는다.

| Sweep 보조값 | 이전 | 통합 |
| --- | ---: | ---: |
| 비교/판독 쪽 | 1 | 1 |
| 자동 후보 쪽 | 0 | 0 |
| pixel_match | 99.30528% | 99.30775% |
| visual_accuracy_proxy (ink_match) | 8.59548% | 8.63704% |

수정 전 잘림도 자동 후보가 0이므로, 후보 0이나 빈 영역이 많은 전체 pixel_match로 통과를 결정하지 않았다. 위 두 줄의 실제 잘림·위치 개선이 수용 근거다.

[이전/PDF 비교](../assets/pr7109_hancom_before_review_p001.png) ·
[통합/PDF 비교](../assets/pr7109_hancom_after_review_p001.png) ·
[통합 overlay](../assets/pr7109_hancom_after_overlay_p001.png) ·
[메타데이터·좌표·exporter/WASM 해시·run manifest](../assets/pr7109_hancom_review_evidence.json) ·
[fidelity 원장](../assets/pr7109_hancom_ledger/run-state.tsv).

### 재현 명령과 한계

```sh
RHWP_BIN=target/nondraft-review-20260914/debug/rhwp venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 0 --source tests/fixtures/multiline_cell_zero_positions/two_lines.hwpx --reference-pdf tests/fixtures/multiline_cell_zero_positions/two_lines-2020.pdf --label pr7109-two-lines --reference-grade 'User supplied Hancom Hwp2022 12.0.0.4605 PDF' --text-only --export-all-svg --layout-ledger --out-dir /private/tmp/rhwp-nondraft-review-20260914/hancom-two-lines-ledger
venv/bin/python scripts/visual_sweep.py --file-target 7109 tests/fixtures/multiline_cell_zero_positions/two_lines.hwpx tests/fixtures/multiline_cell_zero_positions/two_lines-2020.pdf --rhwp-bin target/nondraft-review-20260914/debug/rhwp --wasm-pkg /private/tmp/rhwp-nondraft-review-20260914/pkg --pages 1 --dpi 96 --out /private/tmp/rhwp-nondraft-review-20260914/hancom-two-lines-after
```

이전 비교는 `--rhwp-bin /Users/tsjang/.Trash/rhwp-pr7107-target-20260914/debug/rhwp`, `--wasm-pkg /Users/tsjang/.Trash/rhwp-pr7107-review-evidence-20260914/review/pkg`, output `hancom-two-lines-before`로 같은 명령을 실행했다. 각 output의 `7109/compare/compare_001.png`, `7109/overlay/overlay_001.png`, `7109/review/review_001.png`가 직접 확인한 원시 이미지다.

코드는 바꾸지 않아 기존 통합 code head 및 native/WASM 빌드를 사용했고, 재빌드·전체 회귀·MCP 재변환은 반복하지 않았다. 추가 검증으로 **독립 PDF 미확보 보류는 해소**, 해당 PR 범위의 검토 판정은 승인으로 갱신한다. 비공개 4쪽 원본, 변형 전체의 한컴 출력, Windows Firefox 확장과 성능은 미검증이다. 통합 PR 생성·최종 head CI·사용자 merge 승인은 별도 단계로 남는다.
