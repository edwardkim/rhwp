# PR #1909 Review — task 667: ingest 스키마 확장

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1909 |
| 작성자 | `jangster77` |
| base | `devel` |
| head | `task_m100_667_ingest_schema_evolution` |
| 관련 이슈 | https://github.com/edwardkim/rhwp/issues/667 |
| 문서 작성 시점 규모 | 8 files, +718/-40 |
| 문서 작성 시점 참고 SHA | `992fbc2d24cba952c965e24cf2a072bee965a235` |

`draft`, `mergeable`, CI 상태는 merge 전 최신 PR head 기준으로 다시 확인한다.

## 관련 이슈 요약

#667은 `rhwp-exam-ingest`/`build-from-ingest` 입력 스키마가 시험지의 공유 지문, `<보기>` 박스,
반복 머리말/꼬리말 정보를 구조적으로 표현하지 못하는 문제를 다룬다.

## 변경 범위

- `IngestDocument`
  - `header_text`, `footer_text`, `form_label`, `passages` 추가
- `Question`
  - `passage_ref` 추가
- `StemBlock`
  - `boxed` 블록 추가
- `exam_paper` builder
  - 공유 지문을 첫 참조 위치에 한 번만 출력
  - `boxed` 블록을 별도 paragraph border/fill 스타일로 출력
  - header/footer metadata를 Header/Footer control로 출력
- 문서/샘플
  - `.claude/skills/rhwp-exam-ingest/SKILL.md`
  - `mydocs/manual/cli_commands.md`
  - `tools/rhwp-ingest/schema/ingest_schema_v1.json`
  - `tools/rhwp-ingest/schema/sample_structured.json`

## 렌더 영향 및 visual sweep 판정

이 PR은 원본 PDF 대비 렌더러 정합을 수정하는 PR이 아니라 `build-from-ingest`의 중간 JSON 스키마와
시험지 builder 매핑을 확장하는 PR이다. renderer/layout/typeset/paint 경로는 수정하지 않았다.

따라서 한컴 기준 PDF visual sweep은 수행하지 않았다. 대신 structured ingest 샘플을 실제 HWPX로 생성한 뒤
`export-text`, `dump`, `export-svg` smoke로 공유 지문/boxed/header/footer 구조가 출력되는지 검증했다.

## 로컬 검증

```bash
jq empty tools/rhwp-ingest/schema/ingest_schema_v1.json \
  tools/rhwp-ingest/schema/sample_minimal.json \
  tools/rhwp-ingest/schema/sample_structured.json
git diff --check upstream/devel...HEAD
cargo fmt --check
env CARGO_INCREMENTAL=0 cargo test --lib ingest
env CARGO_INCREMENTAL=0 cargo test --lib exam_paper
env CARGO_INCREMENTAL=0 cargo clippy --lib -- -D warnings
cargo build --bin rhwp
target/debug/rhwp build-from-ingest tools/rhwp-ingest/schema/sample_structured.json \
  -o tmp/issue667-prcheck/sample_structured.hwpx
target/debug/rhwp export-text tmp/issue667-prcheck/sample_structured.hwpx \
  -o tmp/issue667-prcheck/text
target/debug/rhwp dump tmp/issue667-prcheck/sample_structured.hwpx \
  > tmp/issue667-prcheck/sample_structured.dump.txt
target/debug/rhwp export-svg tmp/issue667-prcheck/sample_structured.hwpx \
  -o tmp/issue667-prcheck/svg
```

결과:

- JSON 유효성 통과
- diff check 통과
- fmt check 통과
- ingest unit test 5개 통과
- exam_paper builder unit test 14개 통과
- `clippy --lib -D warnings` 통과
- `cargo build --bin rhwp` 통과
- structured 샘플 HWPX 생성 성공: 1페이지, 문제 2개, 문단 17개
- 공유 지문 텍스트는 `export-text` 결과에서 1회만 출력됨
- dump에서 `<보기>`/보기 본문 문단의 `border_fill_id=2` 확인
- dump에서 `꼬리말(Both): "1/20"` 확인

## 리스크와 범위 밖

- 이미지/Picture 직렬화 완성은 이 PR 범위가 아니다.
- 복잡한 Table/Frame 기반 보기 박스 정밀 복원은 후속 범위다.
- 원본 PDF와의 시각 정합을 보장하는 PR이 아니다.
- `build-from-ingest`의 LineSeg 사전 계산 보강은 이슈 코멘트에 언급된 후속 개선 후보지만,
  이번 PR에서는 schema/IR 매핑 범위로 한정했다.

## 최종 권고

PR head 최신 커밋 기준 GitHub Actions가 통과하면 merge 후보로 판단한다.
옵션 1 self PR 경로이므로 review 문서와 오늘할일을 PR head에 포함한다.

