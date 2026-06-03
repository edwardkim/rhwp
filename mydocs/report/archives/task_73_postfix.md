# 타스크 73 후속 수정 — 최종 결과 보고서

## 개요

타스크 73(문단 부호 표시 기능) 완료 후 검증 과정에서 발견된 3건의 렌더링 버그를 수정하였다.

## 수정 내역

### 1. 빈 문단 ↵ 기호 위치 수정

- **문제**: 빈 문단(텍스트 없는 문단)에서 ↵ 기호가 오른쪽 끝에 표시됨
- **원인**: 빈 문단의 TextRun bbox 폭이 `col_area.width`(전체 컬럼 폭)로 설정되어, `bbox.x + bbox.width`가 오른쪽 끝을 가리킴
- **수정**: `run.text.is_empty()` 체크 → 빈 텍스트이면 `node.bbox.x`(왼쪽 시작) 위치에 기호 배치
- **수정 파일**: `svg.rs`, `web_canvas.rs`, `html.rs` (3개 렌더러 동일 패턴)
- **커밋**: `80314cb`

### 2. TextBox 내 인라인 이미지 위치 수정

- **문제**: `samples/20250130-hongbo.hwp` 2페이지 하단 사각형 내 그림 2개가 사각형 아래에 렌더링됨
- **원인**: `layout_textbox_content()`에서 `inline_y`를 `para_y`(텍스트 레이아웃 후 위치)로 초기화하여, 빈 문단의 기본 줄 높이만큼 이미지가 아래로 밀림
- **수정**: `inline_y` 초기화를 `inner_area.y`(텍스트 영역 시작)로 변경
- **수정 파일**: `layout.rs`
- **커밋**: `c7d4bc9`

### 3. 도형 테두리 선 종류 '없음' 처리

- **문제**: HWP에서 선 종류를 "없음"으로 설정한 사각형에 테두리가 렌더링됨
- **원인**: `drawing_to_shape_style()`에서 `border.width > 0`만 확인하고, `attr` 비트 0-5(선 종류)를 확인하지 않음. 도형의 attr 비트 0-5에서 0은 "없음"을 의미 (표 테두리와 달리 0이 실선이 아님)
- **수정**: `border.attr & 0x3F == 0`이면 stroke 미적용
- **수정 파일**: `layout.rs`
- **부가**: `main.rs` info 커맨드에 사각형 border 디버깅 정보 출력 추가
- **커밋**: `69d28bc`

## 검증

- 488개 Rust 테스트 통과
- WASM 빌드 성공
- 웹 브라우저에서 `20250130-hongbo.hwp` 로드하여 3건 모두 수정 확인
- SVG 내보내기로 사각형 테두리 미렌더링 확인

## 수정 파일 목록

| 파일 | 변경 내용 |
|------|-----------|
| `src/renderer/svg.rs` | 빈 문단 ↵ 기호 위치 수정 |
| `src/renderer/web_canvas.rs` | 빈 문단 ↵ 기호 위치 수정 |
| `src/renderer/html.rs` | 빈 문단 ↵ 기호 위치 수정 |
| `src/renderer/layout.rs` | TextBox inline_y 초기화 수정 + 도형 테두리 선 종류 체크 |
| `src/main.rs` | info 커맨드 사각형 border 정보 출력 |
