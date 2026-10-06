# #5749 상태 표시줄 쪽 표시가 문서 쪽번호를 따르게 한다

## 목표

한글과 같은 규칙으로, 상태 표시줄의 현재 쪽을 **문서가 매기는 쪽번호**(`쪽 > 새 번호로 시작`
반영)로 보여준다. 분모(전체)는 물리 쪽수를 유지한다.

## 문제

`main.ts` 의 `current-page-changed` 핸들러가 `${pageIdx + 1} / ${total} 쪽` 으로 **물리 순번**만
썼다. 앞 2쪽 뒤 1쪽부터 다시 시작하는 문서에서 한글은 세 번째 쪽을 `1쪽` 으로 보여주는데
rhwp-studio 는 `3 / 33 쪽` 이었다.

엔진은 이미 맞는 값을 갖고 있었다 — `PageContent.page_number` 가 NewNumber 를 반영하고,
쪽 하단 렌더도 `- 1 -` 로 다시 시작한다. `rhwp dump-pages --json` 도 `pageNumber` 를 낸다.
빠진 곳은 딱 하나, `get_page_info_native` 의 JSON 이었다. studio 쪽 타입
(`PageInfo.pageNumber?`)과 문서 비교(`compare/diff-engine.ts` 의 `pi.pageNumber ?? (page + 1)`)는
이미 이 필드를 기대하고 있었으므로, 필드를 채우자 그쪽 폴백도 함께 해소된다.

## 구현

1. `src/document_core/queries/rendering.rs` — `get_page_info_native` 응답에
   `"pageNumber"`(= `page_content.page_number`)를 추가한다. 기존 필드는 그대로 둔다.
2. `rhwp-studio/src/view/page-indicator.ts` (신규) — 표시 문자열 조립을 순수 함수로 분리한다.
   문서 쪽번호가 1 이상의 유한값일 때만 쓰고, 아니면 물리 순번으로 물러난다(구 WASM 대비).
3. `rhwp-studio/src/main.ts` — 쪽 정보를 한 번만 조회해 쪽번호와 구역 표시에 함께 쓴다.
4. `rhwp-studio/src/core/types.ts` — 이미 있던 `pageNumber?` 주석을 실제 계약에 맞게 고친다.

페이지네이션(쪽 수 계산)과 쪽 하단 쪽번호 렌더는 건드리지 않았다. 찾아가기(`edit:goto`)의
입력 해석도 이 변경 범위 밖이다.

## 검증

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --locked -- -D warnings` | 통과 |
| `cargo test --test regression_suite_013 issue_5749_page_info_page_number` | 1개 통과 |
| `cargo test --lib document_core::queries::rendering` | 22개 통과 |
| `node scripts/rust-unit-test-tiers.mjs --check` | 4225 tests (증가 없음) |
| `npx tsc --noEmit` (rhwp-studio) | 통과 |
| `npm test` (rhwp-studio) | 1039개 중 1038 통과 · 0 실패 · 1 skip |
| `npm run e2e:status-page-number` | 6개 단언 전부 PASS |
| 실문서 브라우저 실측 | 아래 |

실문서(교육부 보도자료 별첨 「2024학년도 대입전형시행계획 주요사항」, 로컬 코퍼스이므로
저장소에 올리지 않는다)를 studio 에서 열고 물리 쪽마다 상태 표시줄을 읽은 결과:

| 물리 쪽 | 문서 쪽번호 | 고치기 전 | 고친 뒤 |
| --- | --- | --- | --- |
| 1 | 1 | `1 / 33 쪽` | `1 / 33 쪽` |
| 2 | 2 | `2 / 33 쪽` | `2 / 33 쪽` |
| 3 | 1 | `3 / 33 쪽` | **`1 / 33 쪽`** |
| 4 | 2 | `4 / 33 쪽` | **`2 / 33 쪽`** |
| 33 | 31 | `33 / 33 쪽` | **`31 / 33 쪽`** |

같은 문서에서 확인된 전체 쪽수 차이(rhwp 33쪽 vs 한글 43쪽)는 조판 정확도 문제로 #5750 에
따로 등록했다. 이 변경의 범위가 아니다.
