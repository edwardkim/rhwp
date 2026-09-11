# PR #7029 통합 검토

## 판정: 승인

이 판정은 아래 검증한 통합 코드 범위의 로컬 검토 결과다. 새 통합 PR의 최신 head CI와
mergeability는 별도 원격 병합 게이트이며, 아직 통과·병합 완료로 기록하지 않는다.

## 대상과 경로

- PR: https://github.com/edwardkim/rhwp/pull/7029
- 검토일: 2026-09-11. 작성·검토자: jangster77.
- 기본 경로: collaborator 매개 외부 PR의 승인된 체리픽 통합.
- 원 작성자: lpaiu-cs. 통합 PR owner 자동 reviewer 지정은 하지 않았다.
- base: devel. 검증 시작 기준 `b59323de0448df0a42bcb9cdd12cb80d51fe83d2`.
- branch: `review/lpaiu-cs-7004-7022-20260911`.
- 원 PR 4건의 원 커밋 5개를 출처 보존 체리픽한 head: `b788e340a1e5860a09bf89cc1f5fa71ab4b6b1e9`.
- 검증한 메인터너 보정 code head: `d6b1c25bd584e766dd8121f3678a64919040a37d`.
- PR 제출 직전 문서·오늘할일·PNG commit: `950599e90`.
- 이 파일은 PR 번호 발급 후 같은 PR에 추가하는 후행 문서다. source/test/workflow 변경은 없다.

## 원 PR별 결론

| 원 PR | 판정 | 기록 |
| --- | --- | --- |
| #7004 | 승인 | [개별 리뷰](pr_7004_review.md) |
| #7010 | 메인터너 보정 후 수용 가능, 보정·검증 완료 | [개별 리뷰](pr_7010_review.md) |
| #7014 | 승인 | [개별 리뷰](pr_7014_review.md) |
| #7022 | 승인 | [개별 리뷰](pr_7022_review.md) |

#7010 원 head의 감사 사각을 그대로 승인한 것이 아니다. 위 보정 commit을 포함한 이 통합 후보를
검토했다. #7004/#7022의 Vite 충돌은 죽은 --npm 분기 제거와 Node API 서버를 함께 보존해 해소했다.
원 SHA와 로컬 SHA 전체 매핑 및 보정 파일 해시는 [통합 실행 기록](pr_7004_review_impl.md)에 있다.

## 실제 검증

- Rust 집중 nextest 11개, export 감사 집중 Node 13개 통과.
- 전체 nextest: release-test, 8 threads, no-fail-fast, **9,471개 통과·46개 건너뜀**, 실행 360.994초.
- Studio/npm-editor: **1,649개 통과·2개 건너뜀**.
- oracle Python 8개, Undo workflow Python 5개, E2E manifest 129개 일치 검사 통과.
- format, native/WASM32/workspace all-target Clippy, workspace build, suite manifest 검사 통과.
- 잠금 파일 보호 wrapper의 새 WASM 빌드와 Studio tsc/Vite production build 통과.
- Vite 점유 포트 fallback, JavaScript readiness, 자식 exit 0/7 전파와 서버 종료 통과.
- 실제 Vite Undo 깊이 E2E: 255개 이력 전부 Undo, snapshot 슬롯 0, 원 빈 문서 복원.
- #7014 새 WASM 브라우저: 셀 도형 다중 선택 1→2, 실제 드래그 폭 9070→6818,
  Ctrl+Z 후 9070 및 전체 도형 속성 복원. [실제 화면 4장](pr_7014_review.md#최종-코멘트용-시각-증적).

위 검증은 이번 로컬 후보에서 실행했다. 원 PR의 녹색 CI를 통합 PR의 CI 성공으로 대체하지 않는다.
보정 후 이 기록 단계에서는 문서만 추가했으므로 이미 완료한 빌드·회귀를 중복 실행하지 않았다.

## 검증 한계

#7014의 깊이 2 이상 중첩 셀, Group, header/footer, 회전·이동·저장 재열기, 전체 PDF 배치 일치는
직접 검증 범위가 아니다. #7022의 Windows .cmd 경로와 서버 생성 자체 예외 주입도 실행하지 않았다.
이러한 미검증 범위를 성공으로 확대하지 않는다. 현재 주장 범위에서 확인된 blocker는 없다.

## 후속 코멘트와 정리 계획

- 먼저 이 후행 문서까지 포함한 최신 PR head의 실제 CI·mergeability와 별도 병합 승인을 확인한다.
- 병합 뒤 실제 merge SHA와 devel CI 결과를 확인하고 `post_merge.md`에 따라 후속 처리한다.
- 원 PR #7004/#7010/#7014/#7022와 연결 Task #7003/#7002/#7005/#7019의 실제 상태·closing reference를
  API로 확인한다. 지금은 comment·close를 수행하지 않는다.
- #7014 코멘트에는 개별 리뷰의 raw 이미지 Markdown을 확정 merge SHA로 치환해 직접 표시한다.
  나머지 PR은 관련 없는 이미지를 붙이지 않고 해당 검증 결과와 확정 리뷰 링크를 사용한다.
- UTF-8 body-file을 사용하고 동일 merge SHA 코멘트는 중복 등록 대신 수정한다.
- 커밋 대상은 코드·리뷰·오늘할일과 코멘트용 PNG 4장이다. 임시 로그·스크립트·JSON·SVG는 제외했다.
- 원 contributor fork branch와 공유 `target/pr-review`는 보존한다. 승인된 후속 정리 전에는 이 통합
  branch를 제거하지 않는다.
