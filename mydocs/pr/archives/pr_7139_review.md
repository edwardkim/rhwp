---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7139 메뉴 OLE 삭제의 도형 명령 경로 검토

**판정: 승인.** 로컬 체리픽 통합본에 대한 검토다. 원 PR merge/close 또는 GitHub approve를 완료했다는 의미가 아니다.

## 출처와 통합 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#7139](https://github.com/edwardkim/rhwp/pull/7139) / planet6897 |
| 원 head | `e64abb81a2212693206e0e00b3fe4ec698813675` |
| 최신 devel 시작점 | `410d22cdf77e3e9e45159999cab19719c621c025` |
| 통합 code head | `b40c2953cf526b86319f2841c5b5584d7a83d735` |
| 작업 branch | `codex/non-draft-integration-20260914` |
| reviewer | jangster77, 검토 전 요청 등록 확인 |
| source → local | `564ec0128` → `95dfea4ed`; `e64abb81a` → `b40c2953c` |

사용자 지시로 #7141·#7142를 제외했다. draft #6670·#7118도 제외했다. 현재 대상은 #7099·#7109·#7134·#7136·#7137·#7139의 6개 PR, 원 commit 9개다. 원 작성자와 cherry-pick -x 출처를 보존했다.

## 코드와 증거 판독

메뉴 `insert:picture-delete`에서 `ole`을 shape/line/group과 같은 도형 삭제 경로로 보낸다. 키보드 삭제 및 잘라내기 경로와 종류 집합이 일치한다. 기존 `recordObjectMutation`과 선택 해제/화면 갱신 흐름을 재사용하므로 undo 기록을 우회하지 않는다.

소스 가드 2건은 분기 문자열을 확인하는 검사다. 이것만으로 실제 편집 동작을 입증하지 않고 #7136과 합친 fresh WASM/Studio에서 원본 OLE 그림·수식의 삭제/undo를 확인한다. 실제 결과는 후속 절에 기록한다.

원 PR report의 “신고자 첨부물이라 저장소에 넣지 않았다”는 설명과 달리 원본은 이미 Git에 있다. 같은 파일 경로·hash로 재현하고 중복 fixture를 만들지 않는다. #7105의 OLE 내용 편집까지 해결한 것으로 종료하지 않는다.

## CI와 검증 범위

조회한 원 PR head는 위 source SHA와 일치하고, CI에 실패·대기 항목이 없다. skipped/neutral을 실제 검사 통과 수로 합산하지 않는다. [Lint (fmt, clippy, WASM check): SKIPPED](https://github.com/edwardkim/rhwp/actions/runs/34829274818/job/103928858487) · [Frontend package gates: SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/34829274818/job/103928856467) · [Build & Test: SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/34829274818/job/103930913417).

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

## Fresh WASM + 실제 Studio의 OLE 동작

원본 11쪽 문서를 개발 API `loadDocument`와 `CanvasView.loadDocument`로 열었다. 기본 Canvas2D Studio에서 메뉴와 동일한 automation dispatcher의 `insert:picture-delete`를 실행했다. 파일 선택 대화상자나 Windows Firefox 확장 검증으로 표현하지 않는다.

| 대상 | 원본 종류 | 속성 조회 | 삭제 전 → 삭제 → undo | 명령 결과 |
| --- | --- | --- | --- | --- |
| 본문 0/18/0 OLE 그림 | ole | 성공 | 전체 OLE 22 → 21 → 22 | 삭제·undo 모두 ok |
| 본문 0/22/0 레거시 OLE 수식 | ole | 성공 | 전체 OLE 22 → 21 → 22 | 삭제·undo 모두 ok |

[구조/명령 결과](../assets/non_draft_20260914_studio_result.json) · [undo 복원 화면](../assets/pr7139_ole_undo_restored.png). 22개 전부의 개별 삭제를 실행한 것이 아니라 그림·수식 대표 각 1개를 직접 조작했다. 속성 **조회** 성공을 임의의 속성 변경 전부 통과로 확대하지 않는다. 최종 브라우저 console/pageerror는 0개다. 합친 수정 범위에서 머지를 막을 결함을 발견하지 못했다.
