# PR #6850 접수·검증 기록 — 같은 문단의 어울림 그림과 TAC 표 줄 배치

## 최종 판정: 머지 보류

2026-09-08 PR 생성 후의 접수 기록이다. 필수 로컬 검증은 완료했지만 GitHub CI와
최종 self-review는 아직 완료하지 않았다. 최신 trailing head의 required checks 확인과
메인테이너의 self-review·병합 승인이 필요하다. GitHub approve event는 생성하지 않았다.

## 1. 접수와 경로

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#6850](https://github.com/edwardkim/rhwp/pull/6850) / edwardkim |
| base / head branch | devel / task_m100_6812_edf083614_baseline |
| 검증 코드 | `380f838300db555d511a7267421fd65d317dc111` |
| 최초 PR head | `3e02fcc51ccb7c00acd64e6c060867729b661dc9` (검증 코드 이후 보고서만 추가) |
| 최신 통합 devel | `ac8c9fa2c9bfcaadb74f3b46a8ec2a879c3a8099` |
| 최초 규모 | 28 files, +3,078/-70줄 (이 접수 기록 추가 전) |
| GitHub 상태 | Open, MERGEABLE, BLOCKED — CI 진행 중이며 최종 판정값 아님 |
| 트리야지 | assignee edwardkim, milestone v1.0.0, bug/rust/hwp5/layout/rendering/test/table |

자체 작업 PR이므로 collaborator-self-merge 경로를 적용하고 reviewer를 지정하지 않았다.
intake-and-review, local-validation, visual-fixture-evidence와 1,000줄 초과 대형 PR 지침을
함께 적용한다. 이번 기록은 대형 PR의 최종 코드 검토나 admin merge 승인을 대신하지 않는다.

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

## 5. 남은 절차

이 접수 기록과 오늘할일을 같은 PR의 문서 trailing commit으로 push한다. 제품 코드·시험을
바꾸지 않으므로 로컬 전체 회귀를 반복하지 않는다. 최신 trailing head의 CI 확인 → 승인된
최종 self-review → 별도 승인된 병합 및 #6812 종료 순서를 따른다. 병합 방식은 메인테이너의
merge commit 지시를 따르며 squash나 admin 우회를 자동 선택하지 않는다.
