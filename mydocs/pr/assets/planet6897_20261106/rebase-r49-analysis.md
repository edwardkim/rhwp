# R49 리베이스와 충돌 처리 보고서

- 이전 head: `cc3fa6f9205672f745ef62945a1304cb802d3890`. 선행 보정·실제 선택4쪽 증거·생성 JS를 이 커밋에 보존했다.
- 이전 base: `08914c0ed00119db2f26b4fa7ea8eb0495b76cfd`.
- 새 `upstream/devel`: `1b78dc23fd8add8d2bc61958be5533cbeafdbd20`. PR #8768·#8771의 7개 커밋을 반영했다.
- 리베이스 결과와 기본 작업 트리 반영 head: `4969ae2396101079b89d1780bef72d1e1adb8a58`.
- `git rebase --onto <새 base> <이전 base>`의 실제 최종 종료 코드는 **0**이다.
- **153개 전체 커밋의 순서·저자·이메일·저자 시각·전체 메시지**가 보존됐으며 코드 충돌은 **0건**이다.
- 원 contributor **27 PR / 56개 고유 커밋**의 출처와 적용 순서를 보존했다.

[153개 전체 대응표](rebase-r49-all-commit-map.tsv) · [원 기여56개 대응표](contributor-commit-map-r49.tsv).

## 실행과 작업 보존

진행 중인 고정 producer 검증의 기본 작업 트리가 중간 checkout으로 흔들리지 않도록
`output/pr-review/planet6897-integration-20261106/rebase-r49-detached`에서 detached rebase를 수행했다.
완료 후 기본 작업 트리가 clean이고 전체 커밋 보존과 net diff가 새 base의 변경 경로 안에 있음을
확인해 `git reset --keep`으로 결과만 반영했다. 사용자 변경을 stash·삭제·되돌리지 않았고
원격 push도 하지 않았다. 이전 head는 전용 backup ref로 보존한다.

최초 시도는 새 sparse 작업 트리의 index가 초기화되지 않아 checkout 단계에서 종료 코드1로
중단됐다. 충돌 파일은 없었고 코드 replay에 진입하기 전의 자체 준비 오류다. 이번 작업이 만든
새 작업 트리만 원 HEAD에서 materialize해 index를 초기화한 뒤 재개했다. 이 중단과 코드 충돌을
혼동하지 않는다. 재개 후 153/153을 모두 적용했다.

## 들어온 기능과 이후 검증

새 변경의 제품 경로는 `src/paint/builder.rs`, `src/renderer/render_tree.rs`,
`src/renderer/web_canvas.rs`, `rhwp-studio/vite.config.ts`와 신규 샘플 전달 검사다.
Legacy/Layer 본문 clip 공유와 실제 public 샘플 전달의 보존을 새 base·후보에서 확인한다.
기존 글꼴·수식 하락 보정과 원 PR #8389/#8397의 기존 기준 보존도 유지한다.
리베이스 전 R47/R48의 실제 로그·TSV는 해당 고정 producer의 과거 증거이며 새 head의
재검증 완료로 표시하지 않는다. 현재 새 head의 기능 검증과 PR 준비는 계속 진행 중이다.

## 실제 로그

- `output/pr-review/planet6897-integration-20261106/logs/rebase-r49-session.log`: 최초 준비 중단과 index 보정, 최종153개 적용·종료0.
- `output/pr-review/planet6897-integration-20261106/logs/rebase-r49-adopt.log`: 기본 작업 트리 반영·종료0.
- `output/pr-review/planet6897-integration-20261106/rebase-r49-before.json`: 이전 head/base·153개 메타데이터.
- `output/pr-review/planet6897-integration-20261106/rebase-r49-first-setup-stop.json`: 코드 충돌 없는 최초 준비 중단.
- `output/pr-review/planet6897-integration-20261106/rebase-r49-result.json`: 실제 결과·보존 검사·net 변경 경로.

종료 정리 전에 필수 최종 검증·증적을 정식 자산 경로에 보존한다. 임시 로그 전체를 Git에 넣지 않는다.
