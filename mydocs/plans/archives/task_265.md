# Task 265 수행 계획서: 쪽번호 감추기 기능 구현

## 현상

- samples/style-02.hwp: 첫 페이지(커버)에 쪽번호가 표시됨
- 한컴에서는 `[감추기]` 컨트롤(`pghd`)로 첫 페이지 쪽번호를 숨김

## 구현 계획

### 1단계: 파서 수정
- `CTRL_PAGE_HIDE` 태그 ID 수정: `pghi` → `pghd` (스펙 오류)
- PageHide 컨트롤이 정상 파싱되도록 수정

### 2단계: pagination 연동
- `PageContent`에 `page_hide: Option<PageHide>` 필드 추가
- `collect_header_footer_controls`에서 PageHide 수집 (문단 인덱스 포함)
- `finalize_pages`에서 문단 위치 기반으로 해당 페이지에만 PageHide 할당

### 3단계: layout 적용
- `build_page_number`에서 `page_hide.hide_page_num` 체크
- true이면 쪽번호 렌더링 건너뜀

## 참조 파일

| 파일 | 변경 |
|------|------|
| src/parser/tags.rs | CTRL_PAGE_HIDE: `pghi` → `pghd` |
| src/renderer/pagination.rs | PageContent에 page_hide 필드 추가 |
| src/renderer/pagination/engine.rs | PageHide 수집 + 문단 기반 페이지 매칭 |
| src/renderer/pagination/state.rs | PageContent 초기화 |
| src/renderer/layout.rs | build_page_number에서 감추기 체크 |
| src/main.rs | dump에 page_num 필드 추가 |
