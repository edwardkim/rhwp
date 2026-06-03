# Feature 최종 보고서 — 텍스트/마크다운 내보내기

## 개요

이번 기능은 HWP 문서를 페이지 단위 텍스트/마크다운으로 추출하는 CLI 경로를 추가하고, Markdown 이미지 처리 시 컨트롤 참조 실패를 `bin_data_id` 폴백으로 복원하도록 확장했다.

## 변경 사항 요약

### 코드

- `src/main.rs`
  - 신규 명령: `export-text`, `export-markdown`
  - 옵션 파서 및 출력 디렉터리/파일 생성
  - Markdown 이미지 토큰 치환 + 에셋 파일 저장
- `src/document_core/queries/rendering.rs`
  - 텍스트 라인 추출 API
  - Markdown(+표/+이미지토큰) 추출 API
- `src/document_core/commands/clipboard.rs`
  - BinData ID 기반 이미지 데이터/MIME 조회 API

### 산출물 동작

1. 텍스트 내보내기: 페이지별 `.txt`
2. 마크다운 내보내기: 페이지별 `.md`
3. 이미지 포함 문서: `{stem}_assets` 디렉터리 + 마크다운 이미지 링크
4. 컨트롤 좌표 부재 케이스: BinData 폴백으로 이미지 저장 시도

## 설계 포인트

1. 추출 로직은 `DocumentCore` 쿼리 계층에 두고, CLI는 I/O orchestration만 수행
2. 이미지 추출은 "컨트롤 우선 + BinData 폴백" 2단계 전략 적용
3. 실패 시 중단보다 경고 출력 후 계속 진행(페이지 내 부분 성공 허용)

## 알려진 이슈

1. `page_count == 0` 문서에서 일부 에러 메시지 포맷 계산 위험(`page_count - 1`)
2. 확장자 매핑 미정 MIME은 `.bin` 저장
3. 현재 테스트 실패 목록은 CFB 이름 규칙 이슈(`Object name cannot contain \\`)와 연관되어 본 기능과 직접 원인은 분리됨

## 후속 작업 제안

1. 빈 문서 가드 추가 및 사용자 메시지 개선
2. `export-text`/`export-markdown` 전용 자동화 테스트 추가
3. Markdown 렌더 품질 개선(헤딩/목록/각주 표현)
4. MIME 확장자 매핑 확대(예: tif, heic 등)

## 관련 문서

- 계획: [task_m100_237.md](../plans/task_m100_237.md)
- 단계: [task_m100_237_stage1.md](../working/task_m100_237_stage1.md)
