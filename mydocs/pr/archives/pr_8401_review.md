# PR #8401 리뷰 — PDF 가져오기 선택 대기 커서

## 최종 판정

**승인.** 코드 head의 로컬 필수 검증·GitHub CI·self-review를 완료했으며 이번 변경에서
추가 차단 결함을 발견하지 못했다. 이 판정은 코드 검토 결과다. 문서 trailing head의 CI 확인과
작업지시자의 별도 머지 승인이 남아 있으며, 머지·이슈 종료는 수행하지 않았다.

- [PR #8401](https://github.com/edwardkim/rhwp/pull/8401), 작성자 edwardkim, base `devel`.
- 검토 code head: `95e15902ccb2cf76bfecdd24391e7d17c03fffeb`.
- 비교 base: `8fbed1a9c89798f925279e42fed6cbeba26b9c71`.
- [이슈 #8398](https://github.com/edwardkim/rhwp/issues/8398). Open PR이며 사용자 지시로 reviewer 지정 없이 self-review한다.
- 검토일: 2026-10-31 KST. [구현·로컬 검증 기록](../../working/task_m100_8398_completion.md).

## 코드와 회귀 검토

`loadFile`은 파일 식별·읽기 구간만 busy로 감싼다. `loadBytes`는 PDF 선택·재시도·변환 대기를
대화창 상태로 안내하고, 성공한 뷰 초기화부터 busy를 적용한다. 일반 HWP 파싱의 busy와 빈 쪽
준비 순서, PDF 대기 중 기존 문서·화면 보존, 취소·세대 판정·성공 게시 경로를 유지한다.
CSS 강제 덮어쓰기나 busy 카운터 초기화가 없다. 실제 파일 입력과 바이트 입력 양쪽을 검증했다.

#5617 → #8360의 기존 빈 쪽 회귀는 일반 문서 파싱 호출을 식별하는 문자열만 갱신했다.
빈 쪽 준비 → 파싱 순서와 PDF의 기존 화면 보존 assertion을 유지했다. #8389가 추가한
E2E의 누락 manifest 행은 목록만 보완했으며, 그 검사 구현 변경이나 이번 실행을 주장하지 않는다.

조판 원칙·전쪽 Visual Sweep은 비해당이다. 변경은 Studio 로딩 UI의 상태 수명이며
Rust·WASM 엔진·조판 입력·레이아웃·paint 값이나 PDF 변환 정책을 변경하지 않았다.
새 브라우저 회귀는 UI 상태·기존 문서 보존을 검사하며 한컴 출력의 피델리티 판정이 아니다.

## 검증 입력과 로컬 결과

검증 입력은 code head의 Git blob과 실제 파일 바이트가 일치한다.

| 입력 | SHA-256 | 용도 |
| --- | --- | --- |
| `pdf/pr8224/lseg-tab-insert-2020.pdf` | `b461923b3f4faf474ae678eee99d54e5d385be30b7a48dc459d174ca484a15e3` | PDF 선택·취소·한도 거부 후 실제 변환 |
| `samples/para-001.hwp` | `bab4561ceb02cdfa184a1689be9619c08e18d6021cdbc423486b848bc14d267e` | 기존 문서와 일반 HWP 열기 대조군 |

- 신규 실제 브라우저 회귀: 수정 전 busy 검사 FAIL → 수정 후 전체 시나리오 PASS.
- 선택·Escape 취소·진행 중 Worker 취소·한도 오류에서 문서 세대·본문·파일 핸들·최근 문서·실제 IndexedDB 복구본 바이트 보존.
- 파일 입력·바이트 입력·확장자를 바꾼 PDF 드롭, 한도 수정 후 실제 PDF 변환, 손상 HWP 실패 후 정상 열기·새 문서 생성 확인.
- 기존 열기·저장 브라우저 회귀 13개, TypeScript, Studio 2,692 PASS / 0 FAIL / 기존 skip 2, E2E manifest와 문서 링크 검사 통과.
- 브라우저에서 실제 받은 최신 devel의 release + wasm-opt WASM SHA-256은 `dcb50238ab65d703738030ff1498da5d6db46e9c74b50f1226f49825edc0527b`로 manifest와 일치했다. Rust 변경이 없어 기존 빌드를 재사용했다.

## 동일 code head의 GitHub 결과

| 검사 | 결과와 실행 |
| --- | --- |
| CI / Build & Test | [성공](https://github.com/edwardkim/rhwp/actions/runs/41158777092) |
| CodeQL | [성공](https://github.com/edwardkim/rhwp/actions/runs/41158773173) |
| Adapter inter-diff | [성공](https://github.com/edwardkim/rhwp/actions/runs/41158773129) |
| Proptest roundtrip | [성공](https://github.com/edwardkim/rhwp/actions/runs/41158773147) |
| Render Diff | [승인받은 2차 실행 성공](https://github.com/edwardkim/rhwp/actions/runs/41158773128/attempts/2) |
| CI Impact Policy | [최종 감사 성공](https://github.com/edwardkim/rhwp/actions/runs/41162269409) |

Frontend package gate에서 실제 WASM 빌드·호환성·글꼴 바인딩·undo depth·반응형 toolbar·패키지
빌드가 성공했다. 원격 Studio 단위 테스트도 2,692 PASS / 0 FAIL / skip 2다. Rust archive 등
변경 범위에서 제외된 작업을 실행했다고 주장하지 않는다. CodeQL의 실제 분석 언어는
JavaScript/TypeScript이며 다른 언어는 선택 제외다.

### 최초 Render Diff 실패와 1회 재실행

최초 실행은 `basic/KTX.hwp` 1쪽 legacy/layer 비교에서 10,002 / 891,662 pixels
(1.12173%), 최대 channel delta 43으로 실패했다. 나머지 두 입력은 0% 차이였다.
다운로드한 PNG를 직접 대조했고 차이는 표 셀 경계에 집중됐다.

이 검사의 `helpers.mjs:loadHwpFile`은 WASM의 `loadDocument`와 글꼴 준비·Canvas 로딩을
직접 호출해 이번 수정의 `main.ts:loadFile/loadBytes`를 거치지 않는다. 엔진·하네스·입력은
base/head에서 동일하다. 로컬 기본 브라우저 및 CI와 같은 Chromium 1660786의 실행과 추가
3회 모두 170 pixels (0.01907%)로 통과했다. 동일 base의
[직전 PR #8400 실행](https://github.com/edwardkim/rhwp/actions/runs/41157117113)도 같은 값이었다.

작업지시자의 명시 승인으로 **동일 head의 실패한 Render Diff만 1회 재실행**했다. 2차 실행은
KTX 170 pixels (0.01907%), 최대 delta 160으로 통과했고, PDF 보고서·CanvasKit 검증도 성공했다.
최초 실패의 정확한 원인은 미확정이다. 재실행 성공을 원인 해결로 기록하지 않으며,
성공했던 다른 CI 재실행·코드 변경·허용 오차 완화·baseline 수정 없이 결과를 확인했다.

## 남은 원격 절차

이번 문서와 오늘할일은 검증한 code head 위의 문서 전용 trailing commit으로 반영한다.
최신 devel과의 실제 merge tree에서 충돌·공백·문서 링크·기존 오늘할일 보존을 확인한 뒤
push하고 새 head의 CI 재사용과 필수 검사 결과를 확인한다. 실제 머지는 별도 승인 대상이다.
