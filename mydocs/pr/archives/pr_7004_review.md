# PR #7004 검토: 죽은 코드 정리 및 삭제 조각 복원 단순화

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
566d93eaa4be7e998cd19e96a4b48518fded5a5e -> 159b79a458df85d698d4798411de52d01f373ddc
```

## 변경과 보류 사유 해소

- 연결 Task: #7003. 삭제 조각 복원의 문단 배열 재구성을 `splice`로 단순화한 범위를 검토했다.
- 캐시 무효화 경로는 유지된다. 문단 객체의 메모리 주소가 유지된다고 주장하지 않는다.
- Python `Transform`의 NamedTuple 전환은 oracle 테스트로 검증했다. NamedTuple의 일반 tuple 비교·불변성까지 이전 클래스와 완전히 같다고 확대 해석하지 않는다.
- 캐시 목록의 미사용 enumerate와 E2E runner의 죽은 `--npm` 분기를 제거한 취지를 유지했다.
- #7022와 겹친 Vite helper/runner 충돌은 Node API 서버를 채택하면서 #7004의 죽은 분기 제거를 보존했다. 통합 결과는 `b788e340a1e5860a09bf89cc1f5fa71ab4b6b1e9`에 포함돼 있다.
- 초기 보류 사유였던 충돌 해소본의 실행 미검증은 집중·전체 회귀, oracle, 실제 Vite Undo E2E 통과로 해소했다. 현재 검토 범위에서 재현된 blocker 없음.

## 증적과 코멘트 계획

이 PR은 코드 정리·복원 계약 검증이므로 별도의 PDF/PNG를 정확도 증거처럼 만들지 않는다. 향후 원 PR 및 연결 Task에 실제 merge SHA, 체리픽 수용 사실, 조각 복원 집중 7개와 전체 회귀, oracle 8개, Undo 255/255 결과 및 본 문서의 확정 commit 링크를 기록한다. 게시 승인 뒤 UTF-8 body-file을 사용하고 기존 동일 merge SHA 코멘트가 있으면 수정한다.

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

- 원 head: `566d93eaa4be7e998cd19e96a4b48518fded5a5e`; OPEN, non-draft.
- 마지막 조회에서 원 head 추가 변경 없음. 본문 댓글 0개, 제출된 review 0개.
- 아래는 원 head의 원격 결과다. 아직 생성하지 않은 통합 PR의 CI 결과가 아니다. CodeQL의 NEUTRAL은 SUCCESS로 바꿔 적지 않는다.

| 확인 항목 | 결과 | 증적 |
| --- | --- | --- |
| Canvas visual diff | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/actions/runs/34551683563/job/103115774189) |
| adapter inter-diff | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/actions/runs/34551683698/job/103115787479) |
| prop roundtrip | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/actions/runs/34551683700/job/103115793226) |
| Build & Test | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/actions/runs/34551683670/job/103118880187) |
| CI Impact Policy | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/actions/runs/34552765266) |
| CodeQL | SUCCESS | [실제 검사](https://github.com/edwardkim/rhwp/runs/103116425444) |

## 다음 원격 게이트

이 판정은 GitHub approve/merge 실행이 아니다. 보정을 포함한 최종 통합 commit과 문서·필수 PNG를 확정한 뒤, 승인된 통합 PR의 최신 head CI 및 mergeability를 확인해야 한다. 원 PR 직접 merge나 원 이슈 조기 종료를 수행하지 않는다.
