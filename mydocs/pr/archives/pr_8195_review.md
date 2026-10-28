# PR #8195 리뷰 — Studio 편집 명령 오류·복구 계약

## 최종 판정

**승인** — 검토 code head의 오류 분류, 복구 검증과 호스트 전달 계약에서 병합을 막을 결함을 발견하지 않았다.
동일 코드의 로컬 검증과 GitHub Actions 성공을 확인했다. 사용자는 최종 검토·필요한 리뷰 문서
커밋·문제없을 때 merge를 승인했다. 이 문서를 담은 trailing head의 필수 검사, 실제 fast-pass 판정,
최신 base와의 병합·링크 검사 및 exact-head merge 조건은 push 후 다시 확인한다.

## 접수 정보와 경로

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#8195](https://github.com/edwardkim/rhwp/pull/8195) / postmelee / devel |
| 검토 code head | `09b08b7b9f66791e79bfbbfc5ca3973164dada65` |
| 제품 코드·테스트 commit | `fc77d9e7e5ac6218f8a43d8f4da032333d9bfd0d` |
| 고정 base | `60972534680a0c5e51e54ec189b92e252e81a867` |
| 규모 | 19개 파일, +1,130 / -129 |
| 관련 이슈 | [#8104](https://github.com/edwardkim/rhwp/issues/8104), Refs 관계 |
| 작성 시점 참고값 | Open, non-draft, MERGEABLE/CLEAN, reviewer 없음. merge 직전 재확인 필요 |

base route: `collaborator_self_merge.md`.
modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`, `post_merge.md`.
loaded documents: 위 문서와 `pr_review_workflow.md`, `pr_review/README.md`, `review_template.md`.
Self PR이므로 reviewer를 지정하지 않는다. 보정 코드·통합 cherry-pick·충돌이 없으므로 별도
review_impl은 생략한다. 처리 순서는 리뷰·오늘할일 trailing → 병합·링크 검사 → push → 최신 CI 확인 → merge다.

## 코드와 주장 대조

| 검토 대상 | 확인한 내용과 근거 |
| --- | --- |
| `core/command-error.ts` | rollback outcome만으로 성공을 확정하지 않는다. 실제 refresh·probe와 editing/history/dirty 검사를 모두 통과해야 succeeded다. trap은 unknown, rollback 실패는 failed이며 원 호출은 typed failure를 유지한다. |
| `engine/command.ts`, `history.ts` | 최초 snapshot 실패의 before 복원·discard 뒤에만 restored를 기록한다. history 양 스택의 동일 엔트리·순서를 확인한다. #3350 원 PR #3656 → #3661 → #3662의 rollback·원 오류/AggregateError와 #5769 지연 after snapshot 계약을 보존했다. |
| `engine/input-handler.ts`, `cursor.ts` | executeOperation·undo/redo의 공통 경계에서 편집 checkpoint를 잡는다. strict cursor 조회는 일반 fallback과 구분한다. IME·deferred pagination·미지원 역연산 실패는 성공으로 인증하지 않는다. 정상 명령의 refresh 분기는 유지한다. |
| 세션·중복 | 문서 generation과 instance UUID로 이전 문서 보고를 구분한다. 복구 중/blocked의 재시도는 동일 typed error를 받는다. 늦은 보고는 새 문서의 blocked 상태를 설정하지 않는다. |
| `main.ts`, `embed/runtime.ts` | 직접 이벤트·조회 API와 capability를 협상한 MessagePort에만 새 통지를 제공한다. 실패 RPC에 result를 싣지 않으며 동일 event ID를 쓴다. 일반 초기화·로드 오류는 전역 경로에 남긴다. |
| 계약 범위 | 공통 명령 경계가 raw WASM·별도 저장 API까지 보호한다고 주장하지 않는다. document-agent의 기존 recovered와 v1 검사를 구분한다. 호스트의 오류 소유권 검사·저장 차단·복구 UX 책임을 문서화했다. |

[계약 문서](../../tech/studio_command_error_contract.md)와
[구현·검증 보고](../../report/task_m100_8104_report.md)의 주장을 실제 검사 내용과 대조했다.
RuntimeError는 실패 주입이며 실제 엔진 panic 발생 증거로 승격하지 않았다.

## 검증 입력과 결과

이전 구현 단계에서 같은 제품 코드로 실행한 TypeScript, 전체 npm 2,440 PASS,
Studio build, fresh WASM 기반 Chrome 오류 계약 E2E 및 #8102 정상 셀 편집 회귀 PASS를 재사용했다.
검토 code head까지 후속 변경은 MANIFEST 열거값과 결과보고뿐이다.
최종 검토에서 다음 명령을 추가 실행했고 **52 PASS / 0 FAIL / 0 skip**이었다.

```sh
cd rhwp-studio
node --test tests/command-error-contract.test.ts tests/command-history-snapshot.test.ts tests/embed-protocol.test.ts
```

브라우저 기존 실행은 실제 명령 경계에서 clean/dirty snapshot 부분 변경·rollback, 내용 바이트와
선택·history·dirty 보존, 후속 편집/undo, mutation 전 거부, rollback 실패·부분 변경·trap,
render·cursor 조회 실패, 셀 선택 삭제 복구, 실제 문서 교체, window error와 MessagePort를 검사했다.
단위 검사는 미협상 연결, malformed·unsupported·stale·duplicate payload와 실패 RPC error-only 응답도 실행했다.
추가 변경이 없는 동일 코드의 전체 npm·브라우저 검증을 중복 실행하지 않았으며 새 실행으로 기록하지 않는다.

| exact code head GitHub Actions | 실제 결과 |
| --- | --- |
| [CI / Frontend package gates 및 Build & Test](https://github.com/edwardkim/rhwp/actions/runs/40071735589) | success, head SHA 일치 확인 |
| [Render Diff / Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/40071735620) | success, head SHA 일치 확인 |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/40071735598) | Analyze success, GHAS CodeQL neutral |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/40071735605) | success |
| [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/40071735582) | success |
| [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/40073264787) | success |

Rust 미변경에 따른 skipped job과 실행된 frontend package gate를 구분했다. 실패·pending은 없었다.
신규/변경 HWP/HWPX/PDF sample은 없다. E2E 입력은 커밋된 테스트가 새 문서 API와 명령으로
생성하는 합성 편집 문서이며, 외부 파일·독립 한컴 PDF를 수용 근거로 사용하지 않았다.
따라서 입력 파일 누락·대용량 PDF·신규 sample 보안 검사는 비해당이다.

## 조판 원칙·시각 영향과 남은 범위

조판 원칙 준수 검토: 비해당. Rust 조판·측정·배치·serializer·정상 출력 규칙·픽셀 기대값은
변경하지 않았다. 변경한 값은 오류 복구 상태이며, 기존 화면 갱신을 호출한 뒤 상태를 검증한다.
좌표·높이 공통 결과, 저장 LineSeg 수용, 분할·이어받기 변경이 없어 Native/fresh WASM 전쪽 TSV,
독립 Print PDF 및 devel 대비 2%p 비교는 이 PR의 검증 대상이 아니다. Canvas CI 통과를
한컴 조판 일치 증거로 해석하지 않았다. 기준값 완화와 실패 검사 제거는 없다.

기존 MANIFEST #7998 미등재 1건과 문서 메타데이터 누락 20건은 base와 동일하며 이번 변경의
신규 오류가 아니었다. 변경 문서의 링크·메타데이터 검사 결과는 구현 보고에 기록했다.
현재 base는 구현 검증 기준과 동일하여 리베이스 기능 보존 추가 검증은 비해당이다.

호스트 앱 소비 구현·번들 반영·배포는 [alhangeul-macos #594](https://github.com/postmelee/alhangeul-macos/issues/594)의
별도 범위다. 이번 승인과 merge를 해당 앱 배포 완료로 보고하지 않는다. issue close·외부 comment는
이번 merge 명령에 포함하지 않으며, 실제 merge 결과는 작업 대화와 GitHub PR 상태로 확인한다.
