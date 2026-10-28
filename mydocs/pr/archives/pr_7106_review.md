---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-13
---

# PR #7106 — 문서 파일명 창 제목 self-review

**최종 판정: 승인.** 열린 파일 이름을 브라우저·설치형 PWA 창 제목에 표시하는 범위의 판단이다.
코드 후보의 로컬 검증과 GitHub CI를 완료했다. 문서 trailing head의 최신 required checks와
병합 시점의 상태 확인 및 작업지시자 병합 승인은 별도 조건이다.

## 1. 대상과 범위

| 항목 | 검토 기준 |
| --- | --- |
| PR / issue | [#7106](https://github.com/edwardkim/rhwp/pull/7106) / [#6566](https://github.com/edwardkim/rhwp/issues/6566) |
| 작성자 / 검토 | jangster77 / collaborator self-review; reviewer 미지정 |
| branch / base | `fix/6566-document-window-title-20260913` / `devel` |
| 검증 코드 HEAD | `a47e8ab77f6e616c198f52910b3f22312a9baaf3` |
| 조회한 base | `ad6174255e6aabfaa48131f3f5f2f6287a93e48a` |
| 코드 규모 | Studio 7개 파일, +171/-1; Rust·workflow 변경 없음 |
| 작성 시점 상태 | OPEN, MERGEABLE / CLEAN; merge 전 재조회 필요 |
| 기본 / 보조 경로 | collaborator_self_merge / intake_and_review, local_validation, review_only_fast_pass |

기존에는 파일을 열어도 제목이 `rhwp-studio`로 고정됐다. 이제 일반 탭은
`파일명 - rhwp-studio`, 설치형 PWA의 페이지 제목은 파일명만 표시한다. Windows Chrome이
앱 이름을 덧붙여 실제 OS 창 제목은 `rhwp-studio - 파일명`이 된다.
열기·새 문서·다른 이름 저장·호스트 저장 이름 변경과 표시 모드 전환을 지원한다.

## 2. 코드와 동작 기반 회귀 검토

- [WasmBridge](../../../rhwp-studio/src/core/wasm-bridge.ts): 파일명 확정 후 알림을 보낸다.
  로드 알림은 rollback 구간 밖에 있고, 실패한 교체는 기존 문서와 제목을 유지한다.
  setter는 core 파일명 변경이 성공한 뒤 bridge 이름을 바꾸고 알린다.
- [main](../../../rhwp-studio/src/main.ts)과 [제목 연결](../../../rhwp-studio/src/ui/document-title.ts):
  메인 bridge만 구독하므로 비교용 보조 bridge가 제목을 덮어쓰지 않는다.
  standalone·minimal-ui·window-controls-overlay를 구분하고 일반 브라우저 fullscreen은 설치형으로 분류하지 않는다.
- [unit](../../../rhwp-studio/tests/document-title.test.ts)은 무문서 상태, 특수문자 파일명,
  일반↔설치형 표시 모드 전환을 검사했다.
- [E2E](../../../rhwp-studio/e2e/document-title-issue6566.test.mjs)는 실제 WASM과 제품의 열기 이벤트,
  파일 메뉴 새 문서·HWPX 다른 이름 저장, 공개 `rhwpStudio.notifySaved` 경로를 실행했다.
  초기 무문서 로드 실패, 교체 실패, 보조 bridge의 제목 비간섭을 확인했다.
- 제목 연결 호출만 제거한 통제 실험에서 `expected=새 문서.hwp - rhwp-studio, actual=rhwp-studio`로
  실패했다. 원본 코드를 복원한 뒤 같은 E2E를 다시 통과했다.

미해결 결함을 발견하지 못했다. Save As는 실제 직렬화 바이트가 생성되는지 확인했으나
OS 파일 선택기와 쓰기는 테스트 대역을 사용했다. Windows 저장 이름 변경은 호스트 저장 알림으로 확인했다.

## 3. 조판 원칙 준수 검토

**비해당.** 실제 diff는 제목 메타데이터 알림·UI 연결과 해당 테스트다. 파서, 측정, 줄 소속,
점유 높이, 페이지 분할, 페인팅, backend별 배치, baseline/golden을 바꾸지 않았다.
샘플명에 따른 제품 분기나 렌더링 오차 허용치 변경도 없다. PDF/SVG 조판 visual sweep은
이 변경의 검증 게이트가 아니며, 아래 창 제목 검증을 문서 렌더링 동등성으로 확대하지 않는다.

## 4. 검증 입력 커밋 확인

**충족.** 기존 Git 파일 [samples/para-001.hwp](../../../samples/para-001.hwp)를 재사용했다.

- 출처: 저장소 공개 sample; Mac E2E 및 Windows 파일 열기에 사용했다.
- SHA-256: `bab4561ceb02cdfa184a1689be9619c08e18d6021cdbc423486b848bc14d267e`
- 확인한 commit: `a47e8ab77f6e616c198f52910b3f22312a9baaf3`; 실행 입력과 Git blob bytes가 같았다.
- 손상 로드 검사는 `[1, 2, 3]` 합성 bytes이며 테스트 코드에 포함되어 있다.
- 별도 기준 HWPX/PDF는 사용하지 않았다. 기존 문서를 이름 변경해 다시 커밋하지 않았다.

## 5. 로컬·Windows 검증

[검증 결과 JSON](../assets/pr_7106_validation.json)에 코드 SHA, 입력 해시, 로그 해시와 CI check 원문을 보존했다.

| 실행 | 결과 |
| --- | --- |
| `CARGO_TARGET_DIR=target/issue6566-20260913 scripts/wasm-pack-locked.sh --target web --out-dir pkg --dev` | fresh dev WASM PASS; 최종 코드와 Rust 입력 동일 |
| Studio `npx --no-install tsc --noEmit` | PASS |
| Studio `npm test` | 1,662 PASS / 2 skip / 0 FAIL |
| Studio `npm run build`; `/rhwp/` base production build | PASS |
| Studio `npm run e2e:document-title` | 실제 macOS Chrome 152.0.7977.83 PASS |
| `python3 scripts/check_e2e_manifest.py` | 131/131 PASS |
| Windows Chrome 153.0.8010.37 실제 설치형 PWA | standalone 및 Win32 창 제목 PASS |

Windows는 `win10-ted`에서 SSH → 대화형 예약 작업 → Node/Chrome으로 실행했다.
PowerShell의 Win32 `EnumWindows/GetWindowText`로 검증 전용 Chrome PID의 실제 창 제목을 읽었다.
전달한 production build 79개 파일의 SHA-256이 Mac 산출물과 일치했다.
[Windows 성공 원문](../assets/pr_7106_windows_pwa_result.json)은 다음 제목을 기록한다.

| 동작 | 실제 OS PWA 창 제목 |
| --- | --- |
| 새 문서 | `rhwp-studio - 새 문서.hwp` |
| HWP 열기 | `rhwp-studio - para-001.hwp` |
| 호스트 저장 이름 변경 | `rhwp-studio - Windows 저장 확인.hwp` |

사용자가 일반 Chrome 탭의 정상 표시도 직접 확인했다. 작업 표시줄 썸네일 hover는 별도 자동 검증하지 않았다.
페이지 내용만 찍힌 PNG는 OS 프레임 증거가 아니므로 이 제목 판정의 증적으로 사용하지 않았다.
초기 PWA 자동화 실패는 연결 설정 오류였다. browser-level CDP의 pipe 연결 및 standalone 설정 후
실제 설치형 창 검증을 완료했다. 임시 PWA·예약 작업·서버·전용 프로필은 정리했다.
Rust source/test/helper·편집 command·Undo/Redo 변경은 없어 전용 Rust lint/회귀는 비해당이다.
전체 렌더링 성능이나 Docker 최적화 WASM 검증을 수행했다고 주장하지 않는다.

## 6. 코드 후보 GitHub CI

아래는 모두 위 코드 HEAD의 완료 결과다. heavy Rust job의 SKIPPED를 실행 성공으로 세지 않았다.

- [CI](https://github.com/edwardkim/rhwp/actions/runs/34754863593): Frontend package gates와 Build & Test SUCCESS.
  Rust archive·Lint·Native Skia 등은 SKIPPED였다.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34754863440): Canvas visual diff SUCCESS.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34754863571): SUCCESS; GHAS 후속 check NEUTRAL.
- [Adapter](https://github.com/edwardkim/rhwp/actions/runs/34754863546),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34754863583): SUCCESS.
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/34755208629): SUCCESS.

## 7. 제출·병합 후 처리 계획

기존 오늘할일을 보존하고 이 PR의 기록만 추가한다. 코드 후보 뒤에 single-parent 문서 trailing commit을
만들고, push 전 고정 base/head의 merge tree 충돌·공백·변경 문서 링크·기존 오늘 기록 보존을 검증한다.
base/source를 불필요하게 merge/rebase하지 않는다. push 뒤 최신 head의 실제 fast-pass와 required aggregate를 확인한다.

최종 merge 조건은 최신 trailing head CI 통과, mergeability 재조회와 작업지시자 병합 승인이다.
PR 본문의 `Closes #6566`을 유지한다. 승인된 병합 후에는 merge SHA와 이슈 상태를 확인하고,
해결 위치·실제 Windows 창 제목·썸네일 hover 검증 한계를 PR/issue 후속 comment에 요약한다.
comment 게시가 승인되면 UTF-8 `--body-file`로 게시하고 API로 재조회한다.
후속 검증 CI를 임의로 실행하지 않고 duration refresh 결과를 확인하며, devel 동기화와 이번 작업 소유
branch·target만 정리한다. 이 문서 자체는 GitHub approve나 merge를 수행하지 않는다.
