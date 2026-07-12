# PR #2040 처리 계획

작성일: 2026-07-08
대상 PR: https://github.com/edwardkim/rhwp/pull/2040

## 1. 커밋 구성

| SHA | 작성자 | 내용 |
| --- | --- | --- |
| `60737d87f41468cb6c30f76b047a1a093b294cf9` | `lpaiu-cs` | Task #2038: BinData storage id 를 최댓값+1 로 채번 |
| `a25709545ac8f1f102dc676bbf813559e845a7cc` | `lpaiu-cs` | `devel` merge commit |
| `b12f99f7b4a47fe2be36e15f9a5803d2d4ef73a3` | `jangster77` | PR #2040 merge commit |

## 2. 처리 단계

1. PR 메타와 관련 이슈 #2038 확인.
2. reviewer `jangster77` 등록.
3. PR head 를 `local/pr2040` 으로 fetch.
4. `pr2040-merge-test` 브랜치에서 최신 `upstream/devel` 포함 여부와 충돌 여부 확인.
5. 코드 리뷰 및 로컬 검증 실행.
6. GitHub Actions 완료 확인.
7. 작업지시자 옵션 2 승인에 따라 원 코드 PR merge.
8. review 문서를 archive 경로로 이동해 docs-only 후속 PR 로 보존.
9. docs-only PR merge 후 #2038 close/comment 및 PR 감사 코멘트 처리.

## 3. 검증 계획 및 결과

완료:

- cargo 검증 전 `/Users/tsjang/rhwp/target` 하위 항목 삭제.
- merge simulation — `upstream/devel` 이 PR head 에 이미 포함되어 있어 `Already up to date`.
- `CARGO_INCREMENTAL=0 cargo test bindata_storage_id_collision_tests -- --nocapture` — 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` — 통과.
- `cargo fmt --check` — 통과.
- `git diff --check` — 통과.
- `CARGO_INCREMENTAL=0 cargo clippy --release --lib -- -D warnings` — 통과.
- 최신 head 기준 GitHub Actions 전체 완료.

생략:

- visual sweep / MCP 기준 PDF — 렌더 출력 변경이 아니라 BinData storage id 채번과 저장 왕복 보존 변경이다.
- `wasm-pack build --target web --out-dir pkg` — PR head 의 GitHub Actions 에서 WASM Build 가 skipped 이며, 이번 변경은 WASM 산출물 갱신 PR 이 아니다.

대기:

- docs-only 후속 PR merge 및 후속 코멘트.

## 4. 코멘트 초안

docs-only PR merge 후 게시한다.

```markdown
@lpaiu-cs 감사합니다. 첫 기여에 이어 storage id 와 위치 id 의 의미 차이를 정확히 짚어주셨습니다.

검토 결과 #2038 의 주장은 맞다고 판단했습니다. 기존 `insert_picture_native()` / HTML 이미지 등록 경로는 `len() + 1` 값을 storage id 와 `ImageAttr.bin_data_id` 에 함께 사용했기 때문에, storage id 에 구멍이 있는 문서에서 기존 `BIN%04X` stream 과 충돌할 수 있었습니다. 이번 PR 처럼 storage id 는 기존 최댓값+1 로, `ImageAttr.bin_data_id` 는 위치 id 로 유지하는 방향이 현재 renderer lookup 계약과 맞습니다.

로컬에서 다음을 확인했습니다.

- `CARGO_INCREMENTAL=0 cargo test bindata_storage_id_collision_tests -- --nocapture`: passed
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: passed
- `cargo fmt --check`: passed
- `git diff --check`: passed
- `CARGO_INCREMENTAL=0 cargo clippy --release --lib -- -D warnings`: passed

GitHub Actions 최신 head 통과 확인 후 `b12f99f7b4a47fe2be36e15f9a5803d2d4ef73a3` 로 merge 했습니다. 관련 이슈 https://github.com/edwardkim/rhwp/issues/2038 도 후속 코멘트와 함께 정리하겠습니다.
```

## 5. merge 후 후속

- 원 코드 PR merge 완료: `b12f99f7b4a47fe2be36e15f9a5803d2d4ef73a3`.
- `devel` fast-forward sync 완료.
- review 문서는 `mydocs/pr/archives/` 로 이동.
- docs-only 후속 PR merge 후 #2038 에 merge commit, 로컬 검증, CI 결과를 후속 코멘트로 기록하고 close 한다.
- docs-only 후속 PR merge 후 PR #2040 에 first-time contributor 감사 코멘트를 남긴다.
- `local/pr2040`, `pr2040-merge-test` 등 로컬 검토 브랜치 정리.
