# PR #2040 리뷰 기록

작성일: 2026-07-08
대상 PR: https://github.com/edwardkim/rhwp/pull/2040
관련 이슈: https://github.com/edwardkim/rhwp/issues/2038

## 1. 메타

| 항목 | 내용 |
| --- | --- |
| 제목 | Task #2038: BinData storage id 를 최댓값+1 로 채번 (위치와 분리) |
| 작성자 | `lpaiu-cs` |
| 작성자 상태 | GitHub 기준 `FIRST_TIME_CONTRIBUTOR` |
| base | `devel` |
| head | `lpaiu-cs:fix/issue-2038` |
| head SHA | 문서 작성 시점 참고값: `a25709545ac8f1f102dc676bbf813559e845a7cc` |
| 규모 | 3 files, +208/-6 |
| maintainer can modify | true |
| reviewer assign | `jangster77` 요청 등록 완료 |
| merge 결과 | `b12f99f7b4a47fe2be36e15f9a5803d2d4ef73a3` 로 merge 완료 |

## 2. 변경 범위

- `src/model/document.rs`
  - `Document::next_bin_data_storage_id()` 를 추가해 `bin_data_content.id` 와 `doc_info.bin_data_list.storage_id` 양쪽의 최댓값을 기준으로 다음 storage id 를 채번한다.

- `src/document_core/commands/object_ops/picture.rs`
  - `insert_picture_native()` 에서 `ImageAttr.bin_data_id` 용 위치 id 와 `BinDataContent.id` / `BinData.storage_id` 용 storage id 를 분리한다.
  - storage id 구멍이 있는 문서를 구성해 그림 삽입 후 id 유일성과 HWP 저장 왕복 보존을 확인하는 회귀 테스트 2건을 추가한다.

- `src/document_core/html_table_import.rs`
  - HTML 이미지 등록에서도 신규 storage id 를 `next_bin_data_storage_id()` 로 채번하고, 그림 속성의 `bin_data_id` 는 기존 위치 id 의미를 유지한다.

## 3. 관련 이슈 판단

#2038 의 핵심 주장은 맞다.

기존 `devel` 의 `insert_picture_native()` 는 `self.document.bin_data_content.len() + 1` 값을 신규 `BinDataContent.id`, `BinData.storage_id`, `ImageAttr.bin_data_id` 에 함께 사용했다. 그러나 `mydocs/troubleshootings/bin_data_id_index_mapping.md` 와 `renderer::layout::utils::find_bin_data` 계약상 `ImageAttr.bin_data_id` 는 1-based 위치 의미를 우선하며, 저장 스트림 이름에 쓰이는 storage id 와 항상 같다고 볼 수 없다.

따라서 기존 문서가 storage id 2만 갖고 storage id 1이 비어 있는 경우, 기존 구현은 신규 그림의 storage id 를 다시 2로 배정해 `BIN0002` 스트림 충돌을 만들 수 있다. PR 의 수정은 storage id 를 기존 최댓값+1 로 분리 채번하면서 `ImageAttr.bin_data_id` 의 위치 의미를 유지하므로 이슈의 원인과 직접 맞닿아 있다.

## 4. 렌더 영향 및 visual sweep

- 변경 대상은 그림 삽입/HTML 이미지 등록/저장 왕복의 BinData id 채번 경로다.
- 페이지 배치, SVG/PDF renderer, glyph/text layout 경로를 직접 변경하지 않는다.
- 이번 PR 의 핵심 검증은 저장소 내부 id 충돌 방지와 HWP 저장 왕복 보존이므로 MCP 기준 PDF 산출이나 visual sweep 은 필수 검증으로 보지 않는다.

## 5. 코드 리뷰 결과

블로커 발견 없음.

확인한 근거:

- `ImageAttr.bin_data_id` 는 PR 후에도 신규 `bin_data_content` 위치에 맞춘 `len() + 1` 값을 유지한다.
- `BinDataContent.id` 와 `BinData.storage_id` 는 문서 전체에 이미 존재하는 storage id 최댓값 이후로 채번되어 storage stream 이름 충돌을 피한다.
- `html_table_import` 의 이미지 등록 경로도 같은 helper 를 사용하므로 `insert_picture_native` 만 고치는 반쪽 수정이 아니다.
- 신규 테스트가 id 유일성뿐 아니라 `export_hwp` 후 재로드 이미지 데이터 보존까지 확인한다.

비차단 메모:

- `next_bin_data_storage_id()` 는 `u16::MAX` 에서 `saturating_add(1)` 로 같은 값을 반환할 수 있다. 현실적인 BinData 개수 한계에서는 문제가 될 가능성이 낮고, 이번 PR 의 storage id hole 충돌 수정 범위를 막을 사안은 아니라고 판단했다.

## 6. 로컬 검증

브랜치: `pr2040-merge-test`

- cargo 검증 전 `/Users/tsjang/rhwp/target` 하위 항목 삭제 완료.
- `git merge upstream/devel --no-commit --no-ff`
  - 결과: `Already up to date`
- `CARGO_INCREMENTAL=0 cargo test bindata_storage_id_collision_tests -- --nocapture`
  - 결과: 통과.
  - `insert_picture_storage_id_skips_existing_ids`: ok.
  - `insert_into_storage_hole_doc_survives_hwp_save_roundtrip`: ok.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
  - 결과: 통과.
- `cargo fmt --check`
  - 결과: 통과.
- `git diff --check`
  - 결과: 통과.
- `CARGO_INCREMENTAL=0 cargo clippy --release --lib -- -D warnings`
  - 결과: 통과.

WASM 별도 검증:

- Rust core 저장/삽입 경로 변경이지만 WASM 패키지 산출물 갱신을 수반하는 PR 이 아니고, GitHub Actions 의 WASM Build 도 preflight 조건상 skipped 였다. 이번 리뷰에서는 cargo release-test/clippy 와 PR 내 저장 왕복 테스트를 주요 검증으로 삼았다.

## 7. GitHub Actions

최신 head `a25709545ac8f1f102dc676bbf813559e845a7cc` 기준 최종 확인:

- `CI preflight`: success.
- `CodeQL preflight`: success.
- `WASM Build`: skipped.
- `Analyze (javascript-typescript)`: success.
- `Analyze (python)`: success.
- `Analyze (rust)`: success.
- `Build default-feature tests`: success.
- `Native Skia tests`: success.
- `Build & Test`: success.
- `CodeQL`: success.

PR 상태: `MERGEABLE` / `CLEAN`.

## 8. 최종 권고

최종 판단: **merge 완료**.

처리 결과:

- 작업지시자 옵션 2 지시에 따라 원 코드 PR 을 먼저 merge 했다.
- PR #2040 은 `b12f99f7b4a47fe2be36e15f9a5803d2d4ef73a3` 로 `devel` 에 반영됐다.
- PR 본문이 closing keyword 를 사용하지 않아 #2038 은 merge 직후 open 상태다.
- review 문서는 별도 docs-only 후속 PR 로 보존한 뒤 #2038 close/comment 및 PR 감사 코멘트를 처리한다.
