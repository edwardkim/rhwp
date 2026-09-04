# Stage 4 완료보고서 — task_m100_2050

- **이슈**: #2050  **브랜치**: `local/task2050`

## 작업: 목차 + 북마크 + 쪽번호 이동

### `viewer.ts` 추가
- `navEmpty(text)`: 빈 패널 안내 요소.
- `navigateToPosition(section, para)`: `getPageOfPosition` → `{ok, page}` 파싱 → `scrollToPage(page)`.
- `buildOutline()`: `getStructure("auto")` → `roots` 트리 렌더. 노드 클릭 → 위치 이동. `level` 기반 들여쓰기. 빈 트리 시 "목차 정보가 없습니다".
- `buildBookmarks()`: `getBookmarks()` → 목록 렌더. 클릭 → 위치 이동. 빈 목록 시 "북마크가 없습니다".
- `buildSidebar()`가 썸네일/목차/북마크 모두 생성하도록 확장.
- **하단 쪽번호 이동**(추가 요청): `#stb-page` 클릭 → 인라인 number input → Enter 시 해당 페이지로. `pageInputActive` 가드로 입력 중 상태바 덮어쓰기 방지.

### 목차 클릭 이동
- 사용자 추가 요청("목차 클릭 → 페이지 이동")은 `navigateToPosition`으로 구현됨(위 buildOutline 노드 클릭).

## WASM 의존성 (Stage 2 연계)
- 이 단계는 `getStructure` WASM 바인딩(Stage 1) + pkg 재빌드(Stage 2) 완료에 의존.
- **주의**: WASM 빌드 중 다른 Claude 세션이 동시에 Docker 빌드를 돌려 공유 `target/wasm32`·`pkg`가 반복 오염됨. 경합 제거 후 네이티브 단독 빌드(작업지시자 지시: Docker 미사용)로 성공. `getStructure` pkg 노출 확인.

## 검증
- `npm run compile`(webpack) 성공, 타입 체크 통과.
- 산출 `dist/webview/viewer.js`가 `getStructure`/`getBookmarks`/`getPageOfPosition` 참조 확인.

## 상태: ✅ 완료
