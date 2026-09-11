# PR #7022 검토: Vite Node API 기반 E2E 서버 수명주기

## 판정: 승인

## 검토 대상과 권한

- 검토일: 2026-09-11, 검토자: jangster77.
- 기본 경로: collaborator 매개 외부 PR. 원 PR의 사전 reviewer 할당은 완료했으며, 통합 PR의 owner 자동 지정과 구분한다.
- 작성자 lpaiu-cs는 기존 기여자다. 첫 기여자 절차를 새로 적용하지 않는다.
- 검토 브랜치: `review/lpaiu-cs-7004-7022-20260911`.
- 통합 기준: `upstream/devel b59323de0448df0a42bcb9cdd12cb80d51fe83d2`.
- 체리픽 통합 코드 head: `b788e340a1e5860a09bf89cc1f5fa71ab4b6b1e9`.
- 실제 검증 후보: 위 체리픽 head와 #7010 메인터너 보정 commit `d6b1c25bd584e766dd8121f3678a64919040a37d`의 3개 파일. [통합 검증 기록](pr_7004_review_impl.md)의 SHA-256으로 정확한 내용을 구분한다.
- 사용자가 통합 PR 생성을 승인했다. 검증한 코드와 리뷰·오늘할일·필수 PNG를 제출하며 원 PR/이슈 comment·close·merge는 아직 수행하지 않는다.

## 원본 보존 체리픽

```text
7de0bdc8dc230c09cbf0f32676d6bdb2554a0e24 -> b788e340a1e5860a09bf89cc1f5fa71ab4b6b1e9
```

## 변경과 보류 사유 해소

- 연결 Task: #7019. 별도 npm dev 프로세스와 HTTP 폴링 대신 `createServer → listen → resolvedUrls → close`를 사용한다.
- `VITE_URL` 전달, npm script 실행이 필요한 render-diff 경로, 자식 종료 코드 전달을 보존했다. #7004에서 제거한 죽은 `--npm` 분기는 다시 넣지 않았다.
- 초기 원 CI pending은 해소됐다. 최신 원 head의 Build & Test, Canvas visual diff, Adapter, Proptest, CI Impact Policy 성공을 확인했다. CodeQL 표시 NEUTRAL은 별도로 기록한다.
- 로컬에서 17714 포트를 먼저 점유한 상태로 시작해 대체 포트 17715 선택을 확인했다. `/@vite/client`가 200/JavaScript이고 WebSocket client를 포함하는지 검사해 HTML fallback의 거짓 readiness를 배제했다.
- 정상 종료 코드 0 및 의도적 자식 실패 코드 7을 runner가 그대로 반환하고, 각 종료 후 해당 Vite URL이 더 이상 연결되지 않음을 확인했다.
- 실제 npm `e2e:undo-depth`는 Node API 서버로 시작했고 255/255 Undo를 통과했다. #7014 실물 문서 브라우저 검증도 같은 서버 helper에서 수행했다.
- 원 CI 대기 및 통합 충돌본 미검증이라는 초기 보류 사유는 해소됐다. 현재 재현된 blocker 없음.

## 잔여 범위와 코멘트 계획

Windows .cmd 분기, createServer/listen 자체 실패의 강제 주입까지 검증했다고 적지 않는다. 일반 시작·포트 충돌·자식 성공/실패와 정리의 승인이다. 초기 정적 검토에서 언급한 startup 예외 수명주기는 미재현 위험이며 확인된 결함으로 단정하지 않는다.

향후 원 PR/Task #7019에 실제 merge SHA, 원 CI 및 통합 PR/devel CI, 포트 fallback·자식 exit 0/7·서버 close와 실제 Undo E2E 결과를 기록한다. 관련 없는 PDF/PNG와 raw 로그는 첨부하지 않는다. 본 문서 및 필요하면 #7014 브라우저 검증 문서의 확정 commit 링크를 사용한다. 승인 후 body-file로 게시하며 같은 merge SHA의 중복 코멘트는 만들지 않는다.

## 공통 로컬 검증 결과

| 검증 | 실제 결과 |
| --- | --- |
| Rust 집중 nextest | 11개 통과; 조각 삭제 복원 7개, 캐시 목록 1개, 셀 도형 경로 3개 |
| 전체 nextest, release-test, 8 threads, no-fail-fast | 9,471개 통과, 46개 건너뜀, 실패 0개; 실행 360.994초 |
| Rust export 감사 및 추출기 집중 테스트 | 최종 보정본 13개 통과 |
| Studio 및 npm/editor 통합 Node 테스트 | 최종 보정본 1,649개 통과, 2개 건너뜀 |
| 회전 oracle Python 테스트 | 8개 통과 |
| Undo-depth workflow Python 계약 | 5개 통과 |
| E2E 목록 검사 | 추적 파일 129개와 manifest 129행 일치 |
| cargo fmt --all -- --check | 통과 |
| native Clippy, WASM32 Clippy | 각각 -D warnings 통과 |
| workspace build, workspace all-target Clippy | 각각 통과 |
| Rust suite manifest --check | 1,258 sources, 28 suites + 20 exceptions, 48/48 targets 일치 |
| 잠금 파일 보호 wrapper WASM 빌드 | 통과; 210초, 새 pkg 사용 |
| Studio tsc + Vite production build | 통과 |
| 실제 Vite Undo 깊이 E2E | 110라운드, 이력 255개, 조각 삭제 103개, snapshot 슬롯 0, 키보드 Undo 255/255 통과 |

모든 Cargo 계열 명령은 Mac의 `target/pr-review`에서 순차 실행했다. 전체 회귀에 `cargo test --profile release-test --tests`를 사용하지 않았다. 원 PR의 CI 성공을 로컬 실행 결과로 바꿔 기록하지 않는다. 원시 로그·임시 스크립트·JSON은 커밋 대상에서 제외한다.

## 원 PR의 최신 상태

- 원 head: `7de0bdc8dc230c09cbf0f32676d6bdb2554a0e24`; OPEN, non-draft.
- 마지막 조회에서 원 head 추가 변경 없음. 본문 댓글 0개, 제출된 review 0개.
- 아래는 원 head의 원격 결과다. 아직 생성하지 않은 통합 PR의 CI 결과가 아니다. CodeQL의 NEUTRAL은 SUCCESS로 바꿔 적지 않는다.

| 확인 항목 | 결과 | 증적 |
| --- | --- | --- |
| Canvas visual diff | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/actions/runs/34572434025/job/103177355270) |
| adapter inter-diff | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/actions/runs/34572438785/job/103177351119) |
| prop roundtrip | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/actions/runs/34572438706/job/103177341868) |
| Build & Test | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/actions/runs/34572438737/job/103179131918) |
| CodeQL | NEUTRAL | [실제 검사](https://github.com/edwardkim/rhwp/runs/103178139991) |
| CI Impact Policy | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/actions/runs/34573054797) |

## 다음 원격 게이트

이 판정은 GitHub approve/merge 실행이 아니다. 보정을 포함한 최종 통합 commit과 문서·필수 PNG를 확정한 뒤, 승인된 통합 PR의 최신 head CI 및 mergeability를 확인해야 한다. 원 PR 직접 merge나 원 이슈 조기 종료를 수행하지 않는다.
