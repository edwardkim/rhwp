# seo-rii 9건 원격 처리 계획

작업지시자가 2026-11-09 통합 push·Open PR·CI 성공 뒤 self-review·merge·후속 처리 전 범위를 승인했다. 제품 소스와 테스트를 보존하고 검토 문서·댓글 증거 계획만 정본에 맞춰 보완해 제출한다.

대상은 #8749·#8761·#8764·#8765·#8767·#8769·#8777·#8779·#8781이며 Draft는 없다.
upstream/devel 기준 원 기여 SHA 9개를 모두 포함하는 `review/integration-536-seo-rii-preview-flutter`를 사용한다.
제품 source `be778bae9b3803050146dcc47a896940587a612e`와 최종 로컬 검증 source/기록 commit을 구분한다.

1. 제출 직전 원 PR 9개의 head·reviewer·상태와 최신 devel을 재조회하고 clean merge-tree를 확인한다.
   다른 source가 추가됐으면 실제 영향 범위를 검증하며 미검증 상태에서 push하지 않는다.
2. 승인된 upstream 통합 branch에 fast-forward push하고 devel 대상 Open PR을 만든다. 원 fork branch에는 push하거나 삭제하지 않는다.
3. 새 head의 필수 GitHub CI·CodeQL·정책 감사 완료를 확인한다. 원 head의 녹색 CI를 새 head의 성공으로 대신하지 않는다.
   실패하면 원인별로 조사한다. workflow 변경·취소·재실행·보안 분류 변경은 별도 규칙의 승인이 필요하다.
4. exact head의 self-review를 게시하고 최신 base의 merge 가능 상태·필수 check·필요한 review 요건을 확인한다.
   승인된 정상 merge 경로에서 head를 일치시켜 merge commit으로 통합한다. 보호 설정이나 reviewer 요건을 임의로 우회하지 않는다.
5. 원 코드 merge SHA 확인 뒤 post_merge.md의 순서대로 review 9건·공통 검토 기록을 archive로 보존하고,
   오늘할일과 원 PR별 결과를 mydocs 운영 기록 commit으로 devel에 반영한다. 코드·workflow·기존 fixture는 섞지 않는다.
6. primary devel을 upstream/devel과 fast-forward 동기화한다. 원 PR 9건의 간접 MERGED를 API로 확인한다.
   자동 MERGED가 반영되지 않은 건만 통합 수용 근거를 공개하고 완료 close한다. #536·#8701 전체 로드맵은 OPEN을 유지한다.
7. devel에 검토 문서·asset이 실제 존재한 뒤 원 PR별 감사 댓글을 게시한다. 이미 완료된 조치는 반복하지 않는다.
8. 새로 만든 전용 upstream branch·clean local branch/worktree를 정리한다. 원 기여자의 fork branch와 다른 작업공간은 보존한다.
   실제 빌드 사용 종료를 확인한 뒤 primary의 `output/` 전체와 `target/pr-review/debug`를 삭제·확인한다.
   설치된 Flutter/Chrome/SDK 도구와 `local/crates-io-consumption`은 보존한다. 병합 뒤 제품 CI를 새로 시작하지 않는다.

이번 수용 범위는 명시적 Canvas2D preview와 Flutter 참조 host의 단계별 기능이다. 하드웨어 가속·consumer GPU 완료 fence·
장시간 표시·실물 OS IME/접근성 완료를 주장하지 않으며, 이전 Metal 실험 기반의 예외를 옮겨 사용하지 않는다.
