# PR #????: 커넥터 뮤테이터 3종 구역 패스스루 무효화 누락 수정 (#2698)

## 수정
`connector.rs`의 3개 뮤테이터가 IR 변경 후 `section.raw_stream = None`을 하지 않아 라운드트립 계약 위반. 형제 모듈(picture/shape/table 등)과 동일한 패턴으로 무효화 추가.

- `update_connector_subject_ids`: flag 패턴으로 무효화
- `recalculate_connector_routing`: closure 패턴으로 무효화
- `update_connectors_in_section`: 함수 말미에 무효화 추가
