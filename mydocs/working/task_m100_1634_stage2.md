# Task M100 #1634 stage2 — 표 밖 전치 붙여넣기

## 문제

1차 구현은 `table:transpose-paste`가 표 내부 커서에서만 동작한다. 사용자가 제공한 네 번째 참고 그림은
원본 표 범위를 유지하고, 표 밖 대상 위치에 전치된 결과 영역을 새로 만드는 흐름에 가깝다.

현재 동작:

- 선택 셀 범위를 전치 버퍼에 복사할 수 있다.
- 붙여넣기는 기존 표 내부 대상 셀에서만 가능하다.
- 표 밖 문단 커서에서는 전치 붙여넣기 결과가 생성되지 않는다.

## 목표

- 표 밖 일반 문단 커서에서도 `셀 전치 붙여넣기`를 실행할 수 있게 한다.
- 표 밖 붙여넣기는 버퍼 크기 기준으로 `source_cols × source_rows` 새 표를 만들고 내용을 채운다.
- 표 내부 붙여넣기는 1차 구현처럼 기존 표 대상 셀에 덮어쓴다.
- 원본 표는 유지한다.

## 구현 방침

1. Rust native API에 전치 버퍼를 새 표로 붙여넣는 메서드를 추가한다.
   - 입력: `section_idx`, `para_idx`, `char_offset`
   - 처리: 대상 크기로 표 생성 후 `(0,0)`부터 전치 데이터 채움
   - 반환: 기존 표 만들기 결과와 전치 크기 정보
2. WASM/WasmBridge에 새 메서드를 노출한다.
3. Studio `table:transpose-paste` 활성 조건을 `hasDocument && hasTableTransposeClipboard`로 확장한다.
4. 실행 시 커서가 표 내부면 기존 메서드, 표 밖이면 새 표 생성 메서드를 호출한다.
5. 전용 native API 테스트와 Studio 빌드/메뉴 테스트를 갱신한다.

## 검증

- `cargo test transpose --lib`
- `node --test tests/menu-shortcut-labels.test.ts`
- `npm run build`
- `git diff --check`
