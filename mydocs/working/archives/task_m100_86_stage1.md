# Task #86 단계1 완료 보고서: WASM API + 라이브러리 썸네일 추출 함수

## 수행 내용

전체 문서 파싱 없이 CFB의 PrvImage 스트림만 경량 추출하는 API를 구현하였다.

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/parser/mod.rs` | `extract_thumbnail_only()` 함수 + `ThumbnailResult` 구조체 |
| `src/model/document.rs` | `PreviewImageFormat::Png` 변형 추가 |
| `src/wasm_api.rs` | `extractThumbnail(data)` JS API 노출 |
| `src/wasm_api/tests.rs` | 썸네일 추출 테스트 2개 추가 |

## 발견 사항

- 한컴 오피스는 PrvImage에 **PNG** 포맷을 사용함 (기존 문서에는 BMP/GIF로 기재)
- biz_plan.hwp의 PrvImage: **PNG 724x1024, 12,097 bytes**
- BMP → PNG 변환이 불필요하여 구현이 단순화됨

## 검증

- `cargo build` 성공
- `cargo test` 785 passed, 0 failed (+2 신규)
- biz_plan.hwp: PNG 724x1024 추출 확인
- 잘못된/빈 데이터: None 반환 확인 (패닉 없음)

## 커밋

`2edbbca` Task #86 단계1: WASM API + 라이브러리 썸네일 추출 함수
