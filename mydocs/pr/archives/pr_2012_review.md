# PR #2012 리뷰

## 메타

| 항목 | 내용 |
|------|------|
| PR | #2012 |
| 제목 | Studio 대형 문서 입력 랙 완화 |
| 작성자 | jangster77 |
| base | `devel` |
| head | `task_m100_2010_autosave_interval` |
| 관련 이슈 | #2010 |
| 문서 작성 시점 참고값 | mergeable: MERGEABLE, mergeStateStatus: BLOCKED(CI 진행 중) |

## 변경 범위

- `rhwp-studio` 자동저장 설정
  - 사용자 설정에 복구용 자동저장 사용 여부, 복구 저장 간격, idle 자동저장 사용 여부, idle 지연 시간을 추가했다.
  - 환경설정 UI에서 한컴식 자동저장 간격을 명시적으로 조정할 수 있게 했다.
  - 자동저장 상태를 Studio 상태창에 표시해 저장 중/완료/실패 상태를 사용자가 볼 수 있게 했다.
- 자동저장 스케줄러
  - 매 수정 직후 export가 반복되는 구조를 idle timer와 recovery interval timer로 분리했다.
  - 수동 저장, 다른 이름 저장, 인쇄 진입 시에는 pending pagination과 자동저장 상태가 정리되도록 했다.
- 대형 표 입력 성능
  - 표 셀 내부 단순 텍스트 입력은 page-local command 경로를 먼저 사용한다.
  - `rhwp_apply_text_edit_page_local` WASM API와 Studio bridge/command 경로를 추가했다.
  - mouse/table hover 경로의 bbox 캐시를 현재 page hint와 함께 재사용하도록 정리했다.
- 대형 문서 지연 페이지네이션
  - 30쪽 초과 문서에서는 입력 직후 background flush를 자동 실행하지 않고 save/print 등 명시 동작에서 flush한다.
  - 30쪽 이하 문서는 idle 10초 후 기존처럼 flush하여 일반 문서의 즉시 pagination 정합성을 유지한다.
- 운영 기록
  - `mydocs/working/task_m100_2010_stage1.md`
  - `mydocs/working/task_m100_2010_stage2.md`
  - `mydocs/working/task_m100_2010_stage3.md`
  - `mydocs/orders/20260707.md`

## 렌더 영향 및 시각 검증 판정

이 PR은 렌더링 결과를 기준 PDF와 맞추는 수정이 아니라 Studio 편집 중 상호작용 지연을 줄이는 성능/UX PR이다.
WASM API와 command 경로가 추가되지만 사용자-visible 수용 기준은 입력 직후 긴 block task가 사라지는지이다.

따라서 `mydocs/manual/visual_sweep_guide.md` 기준의 PDF/SVG visual sweep은 merge blocker로 보지 않는다.
대표 PNG asset도 만들지 않는다. 대신 실제 115쪽 샘플을 Studio에서 열고 Chrome 계측으로 입력 경로를 확인했다.

확인 샘플:

- `samples/issue1949_giant_cell_nested_tables_perf.hwp`
- 115쪽 대형 문서, 표 셀 입력 경로

핵심 계측 결과:

- headless 115쪽 샘플: 입력 40ms, `insertTextInCellDeferredPagination` 0.9ms, 2.2초 관찰 중 `flushDeferredPagination` 0회, long task 0회
- 실제 Chrome 탭: 입력 156ms, `insertTextInCellDeferredPagination` 8.3ms, 2.2초 관찰 중 `flushDeferredPagination` 0회, long task 78ms 1회
- 기존 재현에서 보였던 입력 직후 4~6초 `flushDeferredPagination` block은 재현되지 않았다.

## 로컬 검증

- `git diff --check` 통과
- `wasm-pack build --target web --out-dir pkg` 통과
- `npm run build` (`rhwp-studio`) 통과
- `npm test` (`rhwp-studio`) 171개 통과
- `cargo test test_insert_text_in_cell` 통과
- 실제 Chrome 탭에서 115쪽 샘플 표 셀 입력 계측 통과

## 리스크

- 30쪽 초과 대형 문서는 편집 직후 전체 pagination flush가 지연되므로, 저장/인쇄 전 명시 flush 경로가 계속 유지되어야 한다.
- autosave interval 설정은 복구용 draft 저장 정책을 조정하는 것이며, 일반 저장 파일을 자동으로 덮어쓰는 기능은 아니다.
- page-local fast path는 단순 텍스트 입력에 한정하며, 복잡한 command는 기존 전체 command 경로를 사용한다.

## 결론

#2010의 핵심 요구인 대형 문서 편집 중 과도한 자동저장/export와 페이지네이션 flush로 인한 입력 랙 완화에 부합한다.
PR 내용과 로컬 검증 결과 기준 merge 후보로 판단한다.

merge 전 최종 조건:

- PR head 최신 커밋 기준 GitHub Actions 통과
- `devel` 기준 merge 가능 상태 확인
- 작업지시자 승인
