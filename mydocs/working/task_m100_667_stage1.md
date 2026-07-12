# Task #667 Stage 1 — 구현 착수

## 확인

- GitHub Issue #667: `Task #664: ingest 스키마 진화 — passage_groups, boxed, page footer`
- 열린 PR 검색: `667 OR ingest OR passage OR boxed` 조건에서 충돌 후보 없음
- 현재 브랜치: `task/m100-667-ingest-schema-evolution`

## 현재 코드 상태

- `src/parser/ingest/schema.rs`
  - top-level은 `version`, `page_size`, `default_font`, `questions`
  - `StemBlock`은 `text`, `image`만 지원
- `src/document_core/builders/exam_paper.rs`
  - 질문별 stem/choice를 단순 문단으로 조립
  - image는 placeholder 텍스트로 출력
  - header/footer, shared passage, boxed block 매핑 없음
- `.claude/skills/rhwp-exam-ingest/SKILL.md`
  - 공유 지문은 `auto_number=false`와 직접 그룹 지시문으로 우회하도록 안내

## 구현 메모

- 새 필드는 모두 optional/default로 추가해 기존 v1 입력 호환을 유지한다.
- `<보기>` 박스는 첫 단계에서 paragraph border/fill 기반으로 구현한다.
- 실제 Vision ingest 자동화 품질은 이번 범위에 포함하지 않는다.

## 구현 결과

- `IngestDocument`에 `header_text`, `footer_text`, `form_label`, `passages`를 추가했다.
- `Question`에 `passage_ref`를 추가했다.
- `StemBlock`에 `Boxed { title, blocks }` 타입을 추가했다.
- `exam_paper` builder가 같은 passage를 첫 참조 위치에서 한 번만 출력하도록 했다.
- `boxed` block은 `border_fill_id=2` 문단 모양으로 출력한다.
- `header_text`/`footer_text`/`form_label`은 Header/Footer control로 매핑한다.
- `tools/rhwp-ingest/schema/sample_structured.json`을 추가했다.

## 검증

```bash
jq empty tools/rhwp-ingest/schema/ingest_schema_v1.json \
  tools/rhwp-ingest/schema/sample_minimal.json \
  tools/rhwp-ingest/schema/sample_structured.json
git diff --check
cargo fmt
env CARGO_INCREMENTAL=0 cargo test --lib ingest
env CARGO_INCREMENTAL=0 cargo test --lib exam_paper
cargo build --bin rhwp
target/debug/rhwp build-from-ingest tools/rhwp-ingest/schema/sample_structured.json \
  -o tmp/issue667-check/sample_structured.hwpx
target/debug/rhwp export-text tmp/issue667-check/sample_structured.hwpx \
  -o tmp/issue667-check/text
target/debug/rhwp dump tmp/issue667-check/sample_structured.hwpx \
  > tmp/issue667-check/sample_structured.dump.txt
target/debug/rhwp export-svg tmp/issue667-check/sample_structured.hwpx \
  -o tmp/issue667-check/svg
```

결과:

- ingest schema unit test 5개 통과
- exam_paper builder unit test 14개 통과
- structured 샘플 HWPX 생성 성공: 1페이지, 문제 2개, 문단 17개
- `export-text`, `dump`, `export-svg` smoke 통과
- 공유 지문 텍스트는 출력 텍스트에서 1회만 확인
- dump에서 `머리말(Both): "홀수형 국어 영역"`, `꼬리말(Both): "1/20"` 확인
- dump에서 `<보기>`와 보기 본문 문단의 `border_fill_id=2` 확인
