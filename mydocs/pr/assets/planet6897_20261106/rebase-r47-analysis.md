# R47 리베이스와 기능 보존 계획

- 이전 head: `53dd51a27bd891190a39429899c79ba300d375e0`, 이전 base: `7f76cee60e38fceb63feacaff706cf984b19c427`.
- 새 base: `08914c0ed00119db2f26b4fa7ea8eb0495b76cfd`, 리베이스 직후 head: `109b3e855ac9752ddf4406cd6a4fa81e106fc913`.
- 실제 제품·기록 151커밋의 순서·author·이메일·author 시각·제목을 보존했다.
  [전체 대응표](rebase-r47-all-commit-map.tsv), [원 기여 56커밋](contributor-commit-map-r47.tsv).
- 코드 충돌은 0건이다. 31번째 pick을 시작할 때 읽기 `git status`의 index refresh와
  경합해 한 번 중단됐다. 활성 lock을 삭제하지 않았으며 lock이 사라진 뒤 reschedule된
  pick을 그대로 재개했다. 중복 커밋·skip·원 contributor head 수정은 없다.
- 임시 굵기 계측 코드는 리베이스 전에 복구했고 커밋하지 않았다. Native family 소유 보정의
  파일 바이트는 리베이스 전후 동일하다. Git 자동 GC는 이 작업의 fetch가 시작한
  유지보수만 중단했으며 Git 객체·pack은 수동 삭제하지 않았다.

새 base는 PR #8269의 메모 API·앵커 삭제 복구·HWP/HWPX 메모/구역 끝 저장과 릴리스
정비를 포함한다. renderer/paint 변경은 없다. 겹친 4파일은 parser/control,
parser/hwpx/section, wasm_api, CLI 매뉴얼이며 모두 자동 적용됐다.
메모 API·왕복·원문 보존·Studio 삭제 Undo의 기존 검사를 새 head에서 실행하고, 현재
보정과 base의 조판 근거를 함께 대조한다. 충돌이 없었다는 사실만으로 완료로 판정하지 않는다.

R46f의 완료 자료와 중단된 전쪽 실행은 **변경 전 증거**로 보존한다. 새 source의
Native family 소유 회귀와 실제 Print 24쪽을 먼저 확인하고, 보정이 확인된 뒤
필수 검증·Native/fresh WASM 전체 TSV·개별 PR 주장 페이지 판독을 완료한다.
이 문서 작성 시점에는 R47 제품 빌드·재검증이 아직 완료 전이다.

작업 로그는 `output/pr-review/planet6897-integration-20261106/logs/rebase-r47-session.log`에 있다.
