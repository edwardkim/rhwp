# PR #8412 self-review — PDF 가져오기 대화창

## 최종 판정

**승인 — #8411의 UI·키보드·안내 개선에서 통합 차단 결함을 발견하지 못했다.**
작성자 self-review이며 GitHub의 타인 승인으로 표시하지 않는다. 코드 head CI는 성공했다.
문서-only trailing head의 CI·충돌 재확인과 작업지시자의 머지 승인은 별도 조건이다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#8412](https://github.com/edwardkim/rhwp/pull/8412) / edwardkim / devel |
| 검토 code head | `533e881d9d3a3979d5eadd9b892a76d0a7c4d7cc` |
| 제품·테스트 구현 | `c288c9e4db`; code head까지 제품·테스트 차이 없음 |
| 최신 base | `1cfa5726cdd8c500df612ec5708103b4a1c4d147`; 재조회 시 전진 없음 |
| 이슈 | [#8411](https://github.com/edwardkim/rhwp/issues/8411), Closes로 연결 |
| 규모·관찰 상태 | 15개 파일, +589/-40; Open, MERGEABLE/CLEAN (2026-10-31 KST 관찰값) |
| 절차 | collaborator self PR + intake/local validation/review-only fast-pass; reviewer 지정 없음 |

## 코드와 증거 대조

| 검토 항목 | 코드·실행 근거 | 판정 |
| --- | --- | --- |
| 기본 화면·맞춤 한도 안내 | `pdf-import-dialog.ts`의 기본 접힘, 빈 입력은 resourceLimits를 생략. 예시/초기화 후 실제 WASM 정책과 표시 JSON 일치 검사 | 충족 |
| 한도·제한 시간 보존 | `options`·`onConfirm`에서 기존 policy validator와 1~300초 범위 유지. 잘못된 JSON·상한 초과·301초는 파일 읽기 전 거부 | 충족 |
| 키보드 | 공통 `ModalDialog`가 select/summary/focusable pre의 기본 동작만 허용하고 편집기로 전파는 차단. PDF Tab은 열린 영역의 활성 입력만 순환 | 충족 |
| 실제 Worker·복구 | 1초 제한 실패 후 30초 재시도 성공, Worker 전달값의 profile/resourceLimits 일치. 처리 중 설정 잠금·취소 유지 | 충족 |
| 선택·취소·기존 문서 | #8398 E2E의 파일/바이트/드롭 열기·취소·재시도·일반 HWP와 문서 보존 검사 유지 | 충족 |
| 낮은 화면·긴 이름·오류 | PDF 전용 CSS에서 내용만 스크롤. 420×320 footer 가시성, 오류 영역 가시성, 두 테마 직접 판독 | 충족 |
| 공통 모달 정상 대조 | `zoom-dialog-transaction.test.mjs`의 배율 변경·취소·트랜잭션 검사 통과 | 충족 |

수정 전 `dialog.ts`만 제공한 격리 대조는 프로필 방향키에서 standard가 유지되어 실패했다.
현재 구현은 해당 검사와 실제 PDF 변환을 통과했다. 파일명·오류·정책 표시에는 `textContent`를
사용하며 입력 문자열을 HTML로 삽입하지 않는다. `settled`·AbortController·active dialog 수명과
`onPrepared` 완료 후 문서 게시 순서는 변경하지 않았다. 검토 후 제품 코드 보정은 필요하지 않았다.

조판 원칙과 Native/Print Visual Sweep은 **비해당**이다. 본문 레이아웃·paint·저장 정보·폰트 소비,
Worker 변환 알고리즘과 기본 자원 정책이 바뀌지 않으며 이번 CSS는 대화창에 한정된다.
UI 픽셀·실제 변환 성공을 문서 조판 피델리티 수치로 승격하지 않는다.

## 검증 결과

로컬 TypeScript, Studio 2,700 PASS·실패 0·기존 skip 2, production build, 브라우저 E2E 3종,
manifest 181/181, 공백·문서 링크 검사를 통과했다.
[완료 기록](../../working/task_m100_8411_completion.md)에 명령·WASM 재사용 근거와 화면을 연결했다.
이번 self-review는 동일 제품·테스트의 결과와 GitHub 실행을 대조했으며 전체 로컬 검사를 중복 실행하지 않았다.

| code head의 GitHub 실행 | 실제 결과 |
| --- | --- |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/41202559129) | SUCCESS. fast_pass=false, Frontend package gates·Build & Test 성공. Rust archive·Native는 범위 분류로 skip |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41202559181) | SUCCESS, JS/TS 분석 실행 성공, GHAS CodeQL neutral |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41202559097) | SUCCESS, Canvas visual diff 성공 |
| [Adapter](https://github.com/edwardkim/rhwp/actions/runs/41202559404) | SUCCESS |
| [Proptest](https://github.com/edwardkim/rhwp/actions/runs/41202559226) | SUCCESS |
| [최종 CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/41204017760) | exact code head status SUCCESS |

GitHub run identity/head SHA·job conclusion을 확인하고 CI 로그의 실제 frontend bundle 완료와
fast_pass=false를 대조했다. skip된 Rust 검사를 실행 통과로 표기하지 않는다.

검증 입력 커밋 확인은 **충족**이다. 아래 파일의 실행 바이트와 code head의 Git blob을 대조했다.

| 기존 저장소 입력 | SHA-256 |
| --- | --- |
| `pdf/issue8403/image-patterns.pdf` | `f121e0f0ca222f397ea80d5b16dba45b1e2d768cc339e1e495ade9e3f512869f` |
| `pdf/pr8224/lseg-tab-insert-2020.pdf` | `b461923b3f4faf474ae678eee99d54e5d385be30b7a48dc459d174ca484a15e3` |
| `samples/para-001.hwp` | `bab4561ceb02cdfa184a1689be9619c08e18d6021cdbc423486b848bc14d267e` |

## 화면·남은 조건

[기본 화면](../../working/assets/issue_8411/default-dark.png)과
[맞춤 한도 안내](../../working/assets/issue_8411/after-dark-expert.png)를 다시 직접 판독했다.
일반 화면의 단순화와 단위·예시·기본값 복원 안내를 확인했다. 기존 캡처 재판독이며 새 head의
재출력으로 표기하지 않는다. 코드·CSS·캡처는 검증 후보 이후 그대로다.
Chromium에서 검증했으며 모든 브라우저의 동작을 보증하는 결과는 아니다.

이번 trailing commit은 이 review·오늘할일과 상태 기록만 변경한다. 최신 base와 merge tree의
공백·링크·기존 기록 보존을 확인한 뒤 push하고, 새 head의 실제 CI 재사용 결과를 확인한다.
머지·이슈 종료·후속 정리는 작업지시자의 승인 뒤 수행한다. 외부 기여자 PR이 아니므로
contributor 후속 댓글 계획은 비해당이다.
