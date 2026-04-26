# Feature 단계 보고서 — 텍스트/마크다운 내보내기 Stage 1

> 계획서: [feature_text_markdown_export_plan.md](../plans/feature_text_markdown_export_plan.md)

## 단계 목표

- CLI에서 문서 텍스트/마크다운 추출 경로를 추가하고,
- Markdown 이미지 처리 시 컨트롤 참조 실패 케이스를 `bin_data_id` 폴백으로 보강한다.

## 구현 결과

### 1) CLI 명령 추가 (`src/main.rs`)

- `export-text <file.hwp> [--output|-o] [--page|-p]`
- `export-markdown <file.hwp> [--output|-o] [--page|-p]`
- 도움말(`--help`)에 신규 명령/옵션 반영

### 2) 텍스트/마크다운 추출 API (`src/document_core/queries/rendering.rs`)

- `extract_page_text_native`
  - RenderTree의 `TextLine` 순회
  - `TextRun`, `FootnoteMarker`, `FormObject` 텍스트 결합
- `extract_page_markdown_with_images_native`
  - 일반 텍스트 라인 추출
  - 표(`Table`)를 Markdown table로 변환
  - 이미지를 `[[RHWP_IMAGE:n]]` 토큰으로 내보내고 참조 메타 반환
- `extract_page_markdown_native`
  - 이미지 메타 없이 Markdown 문자열만 반환

### 3) BinData 폴백 API (`src/document_core/commands/clipboard.rs`)

- `get_bin_data_image_data_native(bin_data_id)`
- `get_bin_data_image_mime_native(bin_data_id)`
- 공통 검증: `bin_data_id == 0`, 범위 초과 처리

### 4) Markdown 이미지 저장/치환 로직 (`src/main.rs`)

- 1차: `(section, para, control)` 기준으로 컨트롤 이미지 조회
- 실패 시: `bin_data_id` 기반 MIME/바이너리 조회로 폴백
- 저장 위치: `{output}/{stem}_assets/`
- 링크 치환: `![image n]({assets_dir}/{filename})`

## 확인한 제약

1. `page_count == 0`일 때 일부 오류 메시지에서 `page_count - 1` 계산 위험 존재
2. MIME 매핑 테이블 외 포맷은 `.bin` 확장자로 저장

## 테스트/실행 메모

- 기능 중심 수동 점검 기준으로 구현 완료
- 저장 포맷 및 토큰 치환 경로를 로그로 확인 가능
- 전체 회귀 테스트 실패 항목은 CFB 경로 네이밍 이슈와 연동되어 별도 추적 필요

## 다음 단계

1. `page_count == 0` 가드 패치
2. 이미지 MIME 확장자 매핑 확장
3. feature 전용 테스트 케이스(텍스트/마크다운 추출) 보강
