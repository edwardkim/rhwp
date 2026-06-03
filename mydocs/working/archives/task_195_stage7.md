# Task #195 단계 7 완료보고서 — 내부 CFB 파싱

> 구현계획서: [task_195_impl.md](../plans/task_195_impl.md)
> 단계: 7 / 8

## 작업 결과

### 신규 파일

- `src/parser/ole_container.rs` (170줄)
  - `OleContainer` 구조체: `preview_emf` / `ooxml_chart` / `raw_contents`
  - `parse_ole_container(&[u8]) -> Option<OleContainer>` 공개 API
  - `strip_ole_presentation_header` — OLE Presentation Stream 헤더 스킵 후 EMF 바이트 반환 (EMR_HEADER + " EMF" 매직 스캔)
  - `cfb` crate 재사용 (기존 Cargo.toml 존재)

### 수정 파일

- `src/parser/mod.rs`: `pub mod ole_container` 선언

### 검증 (통합)

1.hwp 2개 OLE 각각 호출 시:
```
bin_id=1 → preview_emf=Some / ooxml_chart=Some / raw_contents=Some
bin_id=2 → preview_emf=Some / ooxml_chart=Some / raw_contents=Some
```
3종 스트림 모두 정상 추출.

### 단위 테스트 (4건)
- `test_strip_no_emf_magic` — 매직 없는 데이터 → None
- `test_strip_emf_at_offset` — OLE 헤더 뒤 EMR_HEADER 스캔 성공
- `test_parse_empty_bytes` / `test_parse_non_cfb` — 방어적 반환

## 테스트 결과
- `cargo build --release` OK
- `cargo test --release --lib` 882 passed (기존 878 + 신규 4)

## 커밋 대상
- src/parser/ole_container.rs (신규)
- src/parser/mod.rs (pub mod 선언)
- mydocs/working/task_195_stage7.md
