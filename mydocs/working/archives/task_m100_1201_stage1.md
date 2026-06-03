# Task M100-1201 Stage 1 완료 보고 — manifest id map과 section idRef 수집

## 작업 범위

구현계획서 Stage 1 범위에 따라 HWPX masterpage 연결에 필요한 식별자 수집 경로만 추가했다.

## 변경 내용

### `src/parser/hwpx/content.rs`

- `PackageInfo`에 `master_page_items: Vec<PackageItem>` 필드를 추가했다.
- `content.hpf` manifest의 XML 항목 중 href가 `masterpage`를 포함하는 항목을 id/href/media-type과 함께 보존한다.
- 기존 `section_master_page_files`는 Stage 2 fallback 및 기존 테스트 호환을 위해 유지했다.
- 기존 manifest 순서 기반 테스트에 masterpage id/href 보존 검증을 추가했다.

### `src/parser/hwpx/section.rs`

- section XML의 `<hp:masterPage idRef="...">`를 문서 순서대로 수집하는 helper를 추가했다.

```rust
pub fn collect_hwpx_section_master_page_refs(xml: &str) -> Result<Vec<String>, HwpxError>
```

- 네임스페이스 접두사가 있는 `<hp:masterPage>`와 없는 `<masterPage>`를 모두 처리한다.
- `idRef`가 없는 masterpage root는 section 참조로 보지 않고 무시한다.
- 수집 helper의 정상 케이스와 root masterpage 무시 케이스 테스트를 추가했다.

## 검증

통과:

```text
cargo test --lib test_parse_content_hpf_master_pages_by_manifest_order
cargo test --lib test_collect_hwpx_section_master_page_refs
cargo fmt --all --check
```

## 확인 결과

- manifest의 `masterpage0 -> Contents/masterpage0.xml` 같은 id/href 관계를 보존할 수 있게 됐다.
- section XML의 `masterPage idRef`를 파싱할 수 있게 됐다.
- 아직 `parse_hwpx()`의 연결 로직은 변경하지 않았다. 현재 Stage 1은 수집 경로까지만 완료했다.

## 다음 단계

Stage 2에서는 `src/parser/hwpx/mod.rs`에서 다음 연결 흐름을 구현한다.

```text
section XML idRef -> PackageInfo.master_page_items id -> href -> parse_hwpx_master_page()
```

Stage 2 착수 전 작업지시자 승인이 필요하다.
