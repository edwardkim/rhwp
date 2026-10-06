# #5740 rhwp-studio 큰 문서 로딩 중 대기 커서

## 목표

큰 문서를 여는 동안 편집 영역이 빈 화면으로 오래 남는 구간에서, 마우스 커서로 "처리 중"을
알린다. 로딩이 끝나거나 실패해도 커서는 반드시 되돌아가야 한다.

## 문제

10MB 급 `.hwp` 를 열면 파싱·쪽 계산 동안 상태바 텍스트만 바뀌고 커서는 평소 그대로였다.
앱이 일하는 중인지 멈춘 것인지 구분되지 않아 사용자가 빈 화면을 클릭하거나 다시 열기를
시도하게 된다.

## 구현

1. `src/view/busy-cursor.ts` 를 새로 두고 루트 클래스 `rhwp-busy` 토글만 담당하게 한다.
   로딩 경로가 겹쳐 불릴 수 있어(`loadFile` → `loadBytes`) 깊이를 세고, 가장 바깥 처리가
   끝날 때만 되돌린다. `withBusyCursor` 의 `finally` 가 실패 경로도 닫는다.
   루트를 인자로 받아 전역 `document` 없이 검증한다.
2. `src/style.css` 에 `:root.rhwp-busy, :root.rhwp-busy * { cursor: wait !important; }` 를 둔다.
   편집 영역이 `style.cursor` 를 인라인으로 넣으므로(`input-handler-mouse.ts`) `!important`
   없이는 지지 않는다.
3. `main.ts` 의 문서 열기 경로를 감싼다.
   - `loadBytes` — 바이트로 여는 모든 경로(파일 열기 · `?url=` · 자동저장 복구 · 호스트 API)의
     공통 깔때기.
   - `loadFile` — 파일 읽기(`arrayBuffer`)도 큰 파일에서는 시간이 걸린다. 단, 저장 여부 확인
     모달(`canReplaceCurrentDocument`) 다음부터 든다 — 사용자가 답해야 하는 동안은 평소 커서다.
   - `createNewDocument` — 새 문서 생성도 같은 처리 구간이다.

빈 화면 자체를 없애는 오버레이·스피너는 이 변경의 범위 밖이다(별건).

## 검증

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 (Rust 변경 없음) |
| `npx tsc --noEmit -p tsconfig.json` (rhwp-studio) | 통과 |
| `npm test` (rhwp-studio) | 1038개 중 1037 통과 · 0 실패 · 1 skip |
| `npm run e2e:loading-busy-cursor` | 5개 단언 전부 PASS |

e2e 실측(헤드리스 Chrome, dev 서버, 샘플 `samples/2025 행정업무운영 편람(최종).hwp` 10MB):

- 로딩 전: `rhwp-busy` 없음.
- 실제 열기 경로(`#file-input` change → `loadFile` → `loadBytes`)를 태우고 프레임마다 샘플링:
  **39프레임 중 37프레임**에서 루트에 `rhwp-busy` 가 걸려 있고 `getComputedStyle(body).cursor`
  가 `wait` 로 계산됐다.
- 로딩 후: `rhwp-busy` 해제, 커서 `auto` 로 복귀.

단위 계약 테스트 `rhwp-studio/tests/busy-cursor.test.ts` 4건 — 처리 중 걸림/끝나면 복구,
겹침 시 클래스 조작 1회, 실패해도 복구, CSS 가 인라인 커서를 덮는 `!important` 규칙 유지.
