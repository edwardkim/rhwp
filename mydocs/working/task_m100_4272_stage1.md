# Task M100 #4272 Stage 1 — 중첩 셀 텍스트 선택 하이라이트

- 이슈: [#4272](https://github.com/edwardkim/rhwp/issues/4272)
- 기준 브랜치: `upstream/devel`
- 기준 commit: `828eabc19a4953a684e05d523a614256dae28b26`
- 작업 브랜치: `fix/issue-4272-nested-cell-text-selection`
- 작성일: 2026-08-09 KST
- 상태: 구현 및 focused 로컬 검증 완료, 후보 커밋 전

## 목표

rhwp-studio에서 중첩 표 안쪽 셀의 텍스트를 마우스로 드래그할 때 논리 선택뿐 아니라 Canvas
선택 하이라이트도 전체 `cellPath`를 기준으로 표시되게 한다. 깊이 1 셀과 본문 선택의 기존
계약은 유지하고, 브라우저 hot path에 문서 전체 순회나 재조판을 추가하지 않는다.

## RED 재현과 원인

- 샘플: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 위치: 물리 5쪽의 `23,504`
- 실제 경로:
  `[(control=1, cell=0, para=0), (control=2, cell=0, para=12), (control=0, cell=50, para=0)]`
- 수정 전에는 깊이 3 논리 selection이 생성되어도 선택 하이라이트가 0개였다.
- Studio의 `InputHandler.updateSelection()`이 중첩 셀 선택에도 바깥 셀의 평면
  `(parentParaIndex, controlIndex, cellIndex)`를 전달했다. Rust의
  `flat_cell_ctx_matches()`는 오매칭 방지를 위해 깊이 1만 허용하므로 안쪽 TextRun이 선택 rect
  대상에서 제외됐다.

## 반영 내용

- `src/document_core/queries/cursor_nav.rs`
  - 평면 셀과 전체 경로 셀을 구분하는 선택 대상을 도입하고 기존 선택 계산을 공용화했다.
  - 중첩 셀은 전체 경로를 정확히 비교하되 마지막 `cellParaIndex`만 현재 선택 문단으로 바꿔
    다문단 선택을 처리한다.
  - 기존 page hint, page tree cache와 정확성 fallback을 그대로 재사용한다.
- `src/wasm_api.rs`
  - `getSelectionRectsInCellByPath`와 page hint options 변형
    `getSelectionRectsInCellByPathEx`를 추가했다.
- `rhwp-studio`
  - 시작·끝 위치의 전체 셀 컨테이너 경로가 같은지 판정한다.
  - 깊이 2 이상은 경로 API, 깊이 1은 기존 평면 API로 라우팅한다.
  - 경로 API의 hinted/positional 호환 dispatch와 단위 테스트를 추가했다.
- 회귀 래칫
  - 실제 샘플 기반 Rust 통합 테스트와 경로 매처 단위 테스트를 추가했다.
  - 실제 마우스 드래그 CDP E2E, npm 배선과 E2E 매니페스트 항목을 추가했다.

## 성능 경계

- 선택 이벤트마다 문서 전체 탐색이나 페이지 재조판을 새로 수행하지 않는다.
- 기존 선택 rect 계산의 후보 페이지 계획과 렌더 트리 캐시를 사용한다.
- RenderNode 순회 중에는 기존 `CellContext.path`와 입력 slice를 비교하며 노드별 경로 할당을 하지
  않는다. IR 문단 조회용 경로 복제는 선택 문단당 한 번이다.
- 호스트 CDP의 16-step 마우스 드래그 관측값은 path API 17회, 합계 약 4.0ms였다. 이 수치는 해당
  실행의 진단값이며 장비 독립 성능 기준이나 merge threshold로 사용하지 않는다.

## 검증

| 검증 | 결과 |
|---|---|
| 실제 샘플 Rust 통합 테스트 `issue_4272_nested_cell_text_selection` | 1/1 통과 |
| Rust 전체 경로 매처 단위 테스트 필터 `path_matcher` | 2/2 통과 |
| Studio focused `selection-page-hints.test.ts` | 1/1 통과 |
| Studio 전체 `npm test` (샌드박스 밖에서 실행) | 816/816 통과 |
| TypeScript `tsc --noEmit` | 통과 |
| Markdown 상대 링크 검사(변경 문서 4개) | 통과 |
| E2E 매니페스트 양방향 검사(91/91) | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |
| Docker WASM `docker compose --env-file .env.docker run --rm wasm` | 통과 |
| 호스트 Chrome CDP #4272 실제 마우스 드래그 E2E | 통과 |
| 호스트 Chrome CDP #4252 인접 표 객체 선택 E2E | 통과 |

Studio 전체 테스트는 `spawnSync()` 자식 프로세스를 막는 샌드박스 `EPERM` 오탐을 피하도록 처음부터
샌드박스 밖에서 실행했다.

## WASM 산출물

- 위치: `pkg/rhwp_bg.wasm` (gitignore 대상, 실행 중인 dev 서버가 참조하는 저장소 루트 `pkg/`)
- 크기: 7,709,062 bytes
- SHA-256: `80ea461e3a94a0e09d868dd1f4992ffec498a9c3c33688fc76b4352e2f2f2dff`
- 생성된 `pkg/rhwp.js`, `pkg/rhwp.d.ts`, `pkg/rhwp_bg.wasm.d.ts`에서 두 경로 API export를 확인했다.

프로젝트 표준은 Docker WASM 빌드다. 앞서 시도한 host `wasm-pack`은 샌드박스의 읽기 전용
`wasm-bindgen` 임시 경로에서 packaging이 실패했으며, 표준 빌드나 검증 결과로 인정하지 않는다.

## CDP 시각 증적

- [선택 상태 JSON](../../output/4272/nested-cell-text-selection.json)
- [선택 화면 PNG](../../output/4272/nested-cell-text-selection.png)
- [HTML 보고서](../../output/e2e/issue-4272-nested-cell-text-selection-report.html)

호스트 Chrome CDP `http://localhost:19222`에 연결해 현재 `127.0.0.1:7700` dev 서버를 검증했다.
관측 결과는 `23,504` 전체 offset `0 -> 6`, 깊이 3 경로, 논리 selection 존재, 하이라이트 1개,
브라우저 warning/error 0건이다.

## 절차 교정 기록

- 최초 구현은 별도 worktree의 올바른 task branch에서 진행됐지만 VS Code가 보고 있는 메인
  작업공간은 `devel`에 남아 있었다. WIP를 stash 안전본으로 보존한 뒤 메인 작업공간을
  `fix/issue-4272-nested-cell-text-selection`로 전환하고 동일 변경을 복원했다.
- 이후 Docker WASM, focused 테스트와 CDP E2E는 모두 메인 작업공간의 현재 task branch에서 다시
  실행했다. 안전본 `stash@{0}`은 후보 커밋이 확정될 때까지 유지한다.

## 다음 승인 게이트

Stage 1 후보는 준비됐고 신규 스크립트를 Git index에서 추적한 상태로 E2E 매니페스트 검사도
통과했다. 후보 커밋 뒤 전체 PR 검증 게이트, 원격 push, PR 생성, 이슈 comment·close는 이후 각
절차의 승인 경계를 따른다.
