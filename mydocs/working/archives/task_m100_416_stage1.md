# Stage 1 완료 보고서 — Task M100 #416

## 작업 내용

`src/renderer/layout/utils.rs` 의 `find_bin_data` 함수 본체 정정 + 단위 테스트 5 개 추가.

### 변경 1 — `find_bin_data` 본체

```diff
 pub(crate) fn find_bin_data<'a>(bin_data_content: &'a [BinDataContent], bin_data_id: u16) -> Option<&'a BinDataContent> {
     if bin_data_id == 0 {
         return None;
     }
-    // 1-indexed 순번으로 먼저 조회 (기존 동작 유지)
+    // 1-indexed 순번으로 BinDataContent 배열 접근
     if let Some(c) = bin_data_content.get((bin_data_id - 1) as usize) {
-        if c.id == bin_data_id {
-            return Some(c);
-        }
+        return Some(c);
     }
-    // 실패 시 id 필드로 직접 검색 (HWPX 차트처럼 sparse id 사용 시)
+    // 인덱스 범위 밖 (HWPX 차트 sparse id 60000+N 등) — id 직접 검색
     bin_data_content.iter().find(|c| c.id == bin_data_id)
 }
```

함수 docstring 에도 정정 사유 추가 (트러블슈팅 문서 참조 + 가드를 사용하지 않는 이유).

### 변경 2 — 단위 테스트 5 개 추가

`utils.rs` 끝에 `#[cfg(test)] mod tests` 추가:

| 테스트 | 검증 |
|---|---|
| `find_bin_data_returns_none_for_zero` | bin_data_id=0 → None |
| `find_bin_data_indexed_match_storage_id_differs` | **hwpspec.hwp 패턴** — id=12 가 bin_data_id=1 매칭 (회귀 방지) |
| `find_bin_data_indexed_match_storage_id_matches` | 일반 케이스 — storage_id == 인덱스 |
| `find_bin_data_sparse_id_for_hwpx_chart` | HWPX 차트 60001/60002 sparse id |
| `find_bin_data_out_of_range_returns_none` | 인덱스 밖 + 일치 id 없음 → None |

## 검증

| 항목 | 결과 |
|------|------|
| 단위 테스트 (`cargo test --lib renderer::layout::utils::`) | ✅ **5/5 passed**, 0 failed |
| `cargo build --lib` | ✅ 통과 |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |

```
running 5 tests
test renderer::layout::utils::tests::find_bin_data_indexed_match_storage_id_matches ... ok
test renderer::layout::utils::tests::find_bin_data_indexed_match_storage_id_differs ... ok
test renderer::layout::utils::tests::find_bin_data_out_of_range_returns_none ... ok
test renderer::layout::utils::tests::find_bin_data_sparse_id_for_hwpx_chart ... ok
test renderer::layout::utils::tests::find_bin_data_returns_none_for_zero ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1017 filtered out
```

## 영향

- 활성 코드의 `find_bin_data` 가 가드 없이 인덱스 매칭 → hwpspec.hwp 의 storage_id ≠ 인덱스 케이스 정상 매칭
- HWPX 차트 sparse id (60000+N) 는 인덱스 범위 밖이므로 fallback 으로 정상 처리
- 11+ 호출 지점은 헬퍼 경유라 자동 적용 (시그니처 변경 없음)

## 다음 단계

Stage 2 — 회귀 테스트 (`hwpspec.hwp` 페이지 배경 검증 + 시각 검증).

## 산출물

- 변경 파일: `src/renderer/layout/utils.rs`
- 본 보고서: `mydocs/working/task_m100_416_stage1.md`
