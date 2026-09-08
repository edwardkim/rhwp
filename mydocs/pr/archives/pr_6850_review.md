# PR #6850 self-review — 같은 문단의 어울림 그림과 TAC 표 줄 배치

## 최종 판정: self-review 승인 — 병합 승인 대기

2026-09-08 메인테이너의 CI 완료 후 다음 단계 진행 지시에 따라 최종 self-review를
수행했다. `b08b765a857afeffabb7002021ccf8fe9d9b1aeb`의 CI 성공과 변경 코드를
확인했으며, 승인된 원본 1페이지 완료 범위에서 병합을 막을 새 결함을 발견하지 않았다.
이 판정은 메인테이너의 병합 승인을 대신하지 않는다. GitHub approve event는 생성하지 않았다.

## 1. 접수와 경로

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#6850](https://github.com/edwardkim/rhwp/pull/6850) / edwardkim |
| base / head branch | devel / task_m100_6812_edf083614_baseline |
| 검증 코드 | `380f838300db555d511a7267421fd65d317dc111` |
| 최초 PR head | `3e02fcc51ccb7c00acd64e6c060867729b661dc9` (검증 코드 이후 보고서만 추가) |
| 최신 통합 devel | `ac8c9fa2c9bfcaadb74f3b46a8ec2a879c3a8099` |
| 최초 규모 | 28 files, +3,078/-70줄 (이 접수 기록 추가 전) |
| 최종 검토 head | `b08b765a857afeffabb7002021ccf8fe9d9b1aeb` |
| GitHub 상태 | Open, MERGEABLE, CLEAN — 최종 검토 시점 참고값, 병합 직전 재확인 필요 |
| 트리야지 | assignee edwardkim, milestone v1.0.0, bug/rust/hwp5/layout/rendering/test/table |

자체 작업 PR이므로 collaborator-self-merge 경로를 적용하고 reviewer를 지정하지 않았다.
intake-and-review, local-validation, visual-fixture-evidence와 1,000줄 초과 대형 PR 지침을
함께 적용했다. 아래 코드 검토를 수행했으며 admin merge는 승인받거나 실행하지 않았다.

## 2. 문제·구현·수용 범위

관련 이슈는 #6812다. 종이 기준 어울림 그림의 점유 영역으로 남은 줄 폭이 부족할 때
같은 문단의 TAC(글자처럼 취급) 표가 다음 배치 가능한 줄로 이동하도록 한다.
기준 좌표·점유 영역 계산과 줄 배치 결과를 typeset 및 renderer가 공유한다.

메인테이너는 `edf083614`의 **원본 1페이지** 해결을 직접 확인하고 이번 완료 범위를
확정했다. 범위 밖 셀 합성 시험 1건과 전용 helper는 별도 승인으로 제출 후보에서 분리했으며
원문은 Git 이력 및 보존 브랜치에 남겼다. 시험을 ignore하거나 기대값을 완화하지 않았다.
셀·복합 inline 확장의 전면 해결을 주장하지 않는다. 남은 20건은 그대로 통과했다.

같은 문서 7쪽의 #6797/#6798 및 #6847에서 통합된 기여자 수정은 본 PR의 신규 해결로
귀속하지 않는다. #6798 직접 회귀는 아니며 최초 발생 시점은 미확정이다.

## 3. 완료한 로컬 검증

- 별도 review worktree·고정 target에서 suite를 준비하고 Cargo를 순차 실행했다.
- 집중 nextest 92 PASS / 0 FAIL, 전체 nextest **9,218 PASS / 0 FAIL / 기존 46 skipped**.
- fmt·native/WASM32/workspace all-target Clippy·workspace build·manifest·unit tier PASS.
- Native Skia lib 4,112 PASS / 13 ignored, 이미지 대체 표시 2 PASS, 직접 PDF 4 PASS.
- 새 devel의 Studio 코드 추가 확인: npm test 1,493 PASS / 2 skipped, TypeScript PASS.
- 표준 Docker WASM 최적화 포함 8분 31초 PASS.
- IR/overflow/off-canvas/text-overlap 코퍼스 래칫에 신규 악화 없음. 원장 미변경.
- 최신 base 대비 신규/변경 HWP/HWPX/HML/PDF가 없어 신규 fixture 보안 입력 대상 없음.
  외부 controlset clipping 검사는 별도 미실행이며 검사 대상 0의 성공을 PASS로 주장하지 않는다.
- 새 integration source는 `tests/cases/`에만 포함하고 파생 suite·manifest·Cargo target은
  커밋하지 않았다. 검사 종료 시 review worktree 변경 없음.

명령·실행 SHA·네 dump 및 skip 경계는
[완료 보고서 10절](../../report/task_m100_6812_report.md#10-6848-추가-통합-및-최종-제출-검증-2026-09-08)에 기록했다.
동일 환경의 통제된 전후 성능 비교는 미측정이며 검증 실행 시간을 성능 개선으로 해석하지 않는다.

## 4. 시각 근거

renderer/typeset 변경이므로 시각 근거가 필요하다. 메인테이너의 수용 판정은 원본 1페이지에
한정한다. 최종 통합 WASM도 동일 원본 11쪽을 로드하고, 1페이지 SVG는 수용 기준본과
**바이트 단위 동일**임을 확인했다. 이는 전체 11쪽의 새 시각 판정이 아니다.

- 원본: `samples/issue6797/156160455-social-pig-farm-income.hwp`.
- 원본 SHA-256: `1b99b763aac36a14a9f463e35ee894a23eb1083780040eab5e0f02a481c694b8`.
- 1페이지 SVG SHA-256: `b4703e335a54c1200f5455c7de5622a384a3edb3ddd7caeb70640ed10904a9ab`.
- 최종 WASM SHA-256: `c22fa7685f3d0b6b66b8efbc69323bec1ae152164bf95684aab7952224af558f`.
- 로컬 SVG·로그·재현 스크립트: `output/6812/integration-380f83830/`.
- 비정상 합성 문서 및 MCP PDF 이미지 누락에 대한 과도한 해석은 정답 근거로 사용하지 않는다.

Merge 후 contributor PR comment 계획: 자체 PR이므로 외부 contributor에게 보낼 별도
검토 댓글은 이번 범위에 없다. #6798 등 다른 PR의 댓글·종료를 자동 수행하지 않는다.

## 5. 최종 코드 검토

최신 base `ac8c9fa2c9` 대비 source diff와 #6812 integration source의 20건을 읽고
다음 경계를 대조했다. CI 통과만으로 코드 검토를 대체하지 않았다.

- `ObjectPlacementFrame::position` 추출 전후 Paper/Page/Column/Para 기준과 정렬·offset
  계산을 대조했다. 그림의 점유 영역과 표의 바깥 여백을 분리해 적용하는 경로를 확인했다.
- `place_inline_box`의 실제 교차 조건·유한값 검사·가용 구간 선택과, typeset의 회피 높이
  계상 → layout의 확정 좌표 소비를 대조했다. 표를 paint 단계에서만 이동시키는 구조가 아니다.
- 단 flush 시 그림 점유 상태와 흐름 하단을 초기화하고 확정 배치를 이전 단에 넘기는 경로,
  pagination의 동일 페이지 비교 및 문단 offset 이동에 새 metadata를 포함한 것을 확인했다.
- inline flow의 지원 제어 조건과 기존 경로로의 복귀, 제어문자 순서·텍스트/표 배치 및
  후속 본문 하단 보호를 읽었다. 셀·복합 inline의 전면 호환성을 보증하는 검토는 아니다.
- 기존 source-side 시험 변경은 새 구조체 필드 초기화다. 새 회귀 시험은 `tests/cases/`에
  있고 파생 suite·manifest 변경은 PR에 없다. EDF 대비 시험 변경은 승인받은 범위 밖
  셀 시험 1건·전용 helper 91줄 제거뿐이며 나머지 20건의 기대값은 그대로다.

합성·변형 입력의 기하 검사는 내부 불변식 검사이며 한컴의 공식 조판 정답을 입증하지 않는다.
한컴과의 시각 수용 근거는 4절의 원본 1페이지에 한정한다. 전체 문서·셀 확장·성능 동등성은
이번 self-review의 완료 주장에 포함하지 않는다.

## 6. GitHub CI 완료 확인

아래는 모두 최종 검토 head `b08b765a8`에 대한 결과다. 단순 문서 fast-pass로 추정하지 않고
실제 check 결과를 확인했다.

| 검사 | 결과와 실행 근거 |
| --- | --- |
| CI / Build & Test | SUCCESS — [run 34137911281](https://github.com/edwardkim/rhwp/actions/runs/34137911281). Lint, Native Skia, archive A/B/C/D build·test 및 frontend package gate 성공 |
| CodeQL | SUCCESS — [run 34137911208](https://github.com/edwardkim/rhwp/actions/runs/34137911208). Rust/Python/JavaScript-TypeScript 분석 모두 성공, CodeQL 최종 check 성공 |
| Render Diff | SUCCESS — [run 34137910908](https://github.com/edwardkim/rhwp/actions/runs/34137910908), Canvas visual diff 성공 |
| Adapter inter-diff | SUCCESS — [run 34137911153](https://github.com/edwardkim/rhwp/actions/runs/34137911153) |
| Proptest roundtrip | SUCCESS — [run 34137911138](https://github.com/edwardkim/rhwp/actions/runs/34137911138) |
| CI Impact Policy | SUCCESS — [run 34139192174](https://github.com/edwardkim/rhwp/actions/runs/34139192174) |

실패·대기 check는 없었다. WASM Build, Frontend unit gates, Workflow promotion preflight,
Refresh nextest target duration data는 SKIPPED이며 실행 PASS로 세지 않는다.
로컬 Docker WASM 및 Studio 검증 결과는 3절과 별개로 유지한다.

## 7. 남은 절차

이 최종 review와 오늘할일·보고서 갱신은 문서만 변경한 로컬 후속 commit으로 남긴다.
승인 후 같은 PR branch에 push하고 최신 trailing head의 required checks를 확인한다.
제품 코드·시험을 바꾸지 않았으므로 로컬 전체 회귀를 반복하거나 문서 기록만을 위해
devel을 다시 병합하지 않는다. 그 뒤 별도 승인에 따라 merge commit 병합 및 #6812 종료를
처리한다. squash나 admin 우회를 자동 선택하지 않는다. 현재 병합·이슈 종료는 미실행이다.
