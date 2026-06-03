# Task 240 - 4단계 완료 보고서: 명령·메뉴·단축키 연결 및 테스트

## 완료 항목

### insert.ts
- `insert:bookmark` 명령 추가 (BookmarkDialog 싱글톤 패턴)
- `canExecute: (ctx) => ctx.hasDocument`

### index.html
- 메뉴 > 입력 > 책갈피 항목 추가 (`data-cmd="insert:bookmark"`, 단축키 `Ctrl+K,B` 표시)

### input-handler-keyboard.ts
- **코드 단축키 (Ctrl+K,B) 지원 추가**
  - `chordMapK` 매핑 테이블: `b` → `insert:bookmark` (한글 ㅠ 포함)
  - `onKeyDown()` 시작부에 `_pendingChordK` 상태 체크 로직
  - `handleCtrlKey()`에서 Ctrl+K 감지 시 `_pendingChordK = true` 설정
- **F11 책갈피 타입 처리**
  - `handleF11()`에서 `result.type === 'bookmark'` 분기 추가
  - 책갈피 위치로 커서 이동 후 `insert:bookmark` 명령 디스패치 → 대화상자 열기

### goto-dialog.ts
- **찾아가기 대화상자에 책갈피 탭 추가**
  - 탭 바: 쪽 | 책갈피
  - 쪽 탭: 기존 쪽 번호 입력 기능 유지
  - 책갈피 탭: 문서 내 책갈피 목록 표시, 이름순 정렬
  - 항목 클릭으로 선택, 더블 클릭 또는 확인 버튼으로 해당 위치 이동
  - `constructor(services, tab?)` — 'bookmark' 탭으로 직접 열기 지원

### dialogs.css
- 찾아가기 대화상자 탭 바/책갈피 목록 스타일 추가 (`goto-tab-*`, `goto-bookmark-*`)

### body_text.rs (버그 수정)
- **CTRL_DATA에서 책갈피 이름 추출 누락 수정**
  - HWP 스펙: 책갈피 이름은 HWPTAG_CTRL_DATA의 ParameterSet에 저장됨
  - 기존 코드는 Field 컨트롤에 대해서만 CTRL_DATA를 처리
  - `Control::Bookmark`에 대해서도 `parse_ctrl_data_field_name()` 호출 추가
  - hwplib(Java) 크로스 체크로 발견: `ForControlBookmark` → `ForCtrlData` → `ForParameterSet`
  - synam-001.hwp 검증: `""` → `"[별지8] 위임장"` 정상 추출

### bookmark-dialog.ts / goto-dialog.ts
- 빈 이름 책갈피 `(이름 없음)` 표시 처리

## 검증
- TypeScript 컴파일 오류 없음
- WASM 빌드 성공
- Rust 테스트 716개 통과
- synam-001.hwp 책갈피 목록 표시 및 이동 동작 확인
