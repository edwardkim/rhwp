# Task #666 Stage 1 — build-from-ingest 최소 경로 복구

## 목적

#666 은 기존 visual sweep 도구를 새로 만드는 작업이 아니라, Neumann ingest pipeline 의
roundtrip 검증 이슈다. 먼저 현재 devel 에서 `rhwp build-from-ingest` 자체가 동작하는지
확인했다.

## 재현된 문제

수정 전 명령:

```bash
target/debug/rhwp build-from-ingest tools/rhwp-ingest/schema/sample_minimal.json \
  -o tmp/issue666-check/sample_minimal.hwpx
```

결과:

```text
오류: HWPX 직렬화 실패 - XML 쓰기 실패: 미등록 ID 참조 발견: charPrIDRef: [0]
```

## 원인

`build_exam_paper()` 가 `Document::default()` 로 문서를 만든 뒤 문단에는
`char_shape_id=0`, `para_shape_id=0`, `style_id=0` 을 참조시켰지만,
HWPX serializer 가 header 에 등록할 기본 `DocInfo` pool 을 만들지 않았다.

## 수정

`src/document_core/builders/exam_paper.rs` 에서 ingest 문서 속성 기반으로 최소 유효
DocInfo 를 초기화했다.

- `default_font` 를 7개 언어 font face group 에 등록
- `BorderFill`, `TabDef`, `CharShape`, `ParaShape`, `Style` 기본 항목 등록
- `CharShape`/`ParaShape`/쪽 테두리의 `borderFillIDRef` 를 등록된 1-based id `1` 로 설정
- `page_size` 를 section `PageDef` 에 반영

## 검증

```bash
cargo test document_core::builders::exam_paper --quiet
```

- 결과: 11 passed

```bash
cargo build --bin rhwp --quiet
target/debug/rhwp build-from-ingest tools/rhwp-ingest/schema/sample_minimal.json \
  -o tmp/issue666-check/sample_minimal.hwpx
target/debug/rhwp export-text tmp/issue666-check/sample_minimal.hwpx
target/debug/rhwp dump tmp/issue666-check/sample_minimal.hwpx
target/debug/rhwp export-svg tmp/issue666-check/sample_minimal.hwpx -o tmp/issue666-check/svg
```

- `build-from-ingest`: 성공, HWPX 6439 bytes, 문제 3개, 문단 21개
- `export-text`: 성공, 문제 stem 과 ①~⑤ 선택지 텍스트 보존 확인
- `dump`: 성공
- `export-svg`: 성공

```bash
cargo fmt --check
git diff --check
```

- 결과: 통과

## 판단

- 기존 visual sweep 은 #666 의 시각 검증 보조 수단으로 재사용한다.
- #666 은 visual sweep 존재만으로 close 할 수 없다. ingest JSON 에서 HWPX 를 생성하고
  텍스트/구조 라운드트립을 확인하는 별도 의미가 남아 있다.
- 다만 4종 시험지 전체 검증은 실제 page PNG/Vision 분석으로 `ingest.json` 산출물이
  필요하므로, 본 단계는 `build-from-ingest` 기반 명령 복구까지로 한정한다.
