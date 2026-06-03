# 구현 계획서 — Task M100 #416

## 이슈

[#416](https://github.com/edwardkim/rhwp/issues/416) — find_bin_data 가드 결함 (Task #195 회귀, 페이지 배경 이미지 잘못 표시)

## 단일 진실 소스 — `src/renderer/layout/utils.rs`

영향 라인 (현재 기준):

| 라인 | 코드 |
|------|------|
| 13-25 | `pub(crate) fn find_bin_data(...)` 함수 본체 |

본 task 는 함수 본체만 수정 + 단위 테스트 추가 + 트러블슈팅 문서 갱신.

## 호출 지점 (변경 없음, 자동 적용)

총 11+ 곳 — 본 헬퍼 경유라 시그니처 변경 없으므로 자동 정정:

| 파일 | 라인 |
|------|------|
| `src/renderer/layout.rs` | 647, 2556 |
| `src/renderer/layout/table_layout.rs` | 304 |
| `src/renderer/layout/table_cell_content.rs` | 630 |
| `src/renderer/layout/picture_footnote.rs` | 81, 278 |
| `src/renderer/layout/shape_layout.rs` | 954, 986, 1114 |
| `src/renderer/layout/paragraph_layout.rs` | 1695, 1945, 2028 |

## Stage 1 — `find_bin_data` 함수 정정 + 단위 테스트

### 변경 1 — 함수 본체

```rust
/// bin_data_id(1-indexed 순번)로 BinDataContent를 찾는다.
/// bin_data_id는 doc_info의 BinData 레코드 순번(1부터 시작)이며,
/// BinDataContent 배열도 같은 순서로 저장되어 있다.
///
/// HWPX 차트는 sparse id (60000+N) 를 사용하므로 인덱스 범위 밖일 때만 id 직접 검색.
pub(crate) fn find_bin_data<'a>(bin_data_content: &'a [BinDataContent], bin_data_id: u16) -> Option<&'a BinDataContent> {
    if bin_data_id == 0 {
        return None;
    }
    // 1-indexed 순번으로 BinDataContent 배열 접근 (storage_id 가드 제거 — 트러블슈팅 #2026-02-17 정정 동작 복원)
    if let Some(c) = bin_data_content.get((bin_data_id - 1) as usize) {
        return Some(c);
    }
    // 인덱스 범위 밖 (HWPX 차트 sparse id 60000+N 등) — id 직접 검색 (Task #195 의도 보존)
    bin_data_content.iter().find(|c| c.id == bin_data_id)
}
```

### 변경 2 — 단위 테스트 추가

`src/renderer/layout/utils.rs` 끝에 추가:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::bin_data::BinDataContent;

    fn mk(id: u16, ext: &str) -> BinDataContent {
        BinDataContent { id, data: vec![], extension: ext.to_string() }
    }

    #[test]
    fn find_bin_data_returns_none_for_zero() {
        let v = vec![mk(1, "png")];
        assert!(find_bin_data(&v, 0).is_none());
    }

    #[test]
    fn find_bin_data_indexed_match_storage_id_differs() {
        // hwpspec.hwp 패턴 — bin_data_id=1 이 storage_id=12 를 가리킴
        let v = vec![
            mk(12, "png"),  // index 0 (bin_data_id=1)
            mk(1, "bmp"),   // index 1 (bin_data_id=2)
            mk(2, "bmp"),   // index 2 (bin_data_id=3)
        ];
        // bin_data_id=1 → 인덱스 0 의 BIN000C.png 매칭 (storage_id=12)
        let c = find_bin_data(&v, 1).expect("매칭");
        assert_eq!(c.id, 12);
        assert_eq!(c.extension, "png");
    }

    #[test]
    fn find_bin_data_indexed_match_storage_id_matches() {
        // 일반적인 케이스 — storage_id 가 인덱스와 일치
        let v = vec![mk(1, "jpg"), mk(2, "png"), mk(3, "bmp")];
        for i in 1..=3u16 {
            let c = find_bin_data(&v, i).expect("매칭");
            assert_eq!(c.id, i);
        }
    }

    #[test]
    fn find_bin_data_sparse_id_for_hwpx_chart() {
        // HWPX 차트 — sparse id 60000+N
        let v = vec![
            mk(1, "png"),
            mk(2, "png"),
            mk(60001, "ooxml_chart"),  // 차트 1
            mk(60002, "ooxml_chart"),  // 차트 2
        ];
        // bin_data_id=60001 → 인덱스 범위 (60001-1=60000) 밖 → fallback id 직접 검색
        let c = find_bin_data(&v, 60001).expect("차트 매칭");
        assert_eq!(c.id, 60001);
        assert_eq!(c.extension, "ooxml_chart");
    }

    #[test]
    fn find_bin_data_out_of_range_returns_none() {
        let v = vec![mk(1, "png"), mk(2, "png")];
        assert!(find_bin_data(&v, 99).is_none());
    }
}
```

### Stage 1 검증

- `cargo test --lib utils::tests` 통과
- 빌드 (`cargo build --lib`) 통과
- clippy (`cargo clippy --lib -- -D warnings`) 통과

## Stage 2 — 회귀 테스트 (실제 hwp 샘플)

### 변경 3 — `hwpspec.hwp` 페이지 배경 회귀 테스트

`src/wasm_api/tests.rs` 또는 새 통합 테스트 파일 (`tests/issue_416_page_bg_bin_data.rs`) 에 추가:

```rust
//! Issue #416: hwpspec.hwp 1 페이지 페이지 배경의 BinData 매칭 검증
//! find_bin_data 가드 결함 회귀 방지

use std::path::Path;

#[test]
fn issue_416_hwpspec_page_bg_bin_data_matches_correct_image() {
    let path = Path::new("samples/hwpspec.hwp");
    if !path.exists() { return; }  // CI 환경에서 샘플 없으면 skip

    let bytes = std::fs::read(path).expect("hwpspec.hwp 로드");
    let doc = rhwp::parser::parse(&bytes).expect("파싱");

    // 첫 번째 BIN_DATA 항목 (인덱스 0, bin_data_id=1) 의 storage_id 와 데이터 크기 검증
    let first = doc.bin_data_content.first().expect("BIN_DATA 존재");
    // hwpspec.hwp 의 첫 BIN_DATA 는 storage_id=12 (BIN000C.png), 페이지 표지 이미지
    assert_eq!(first.id, 12, "첫 BIN_DATA storage_id 가 12 (BIN000C) 이어야 함");
    assert!(first.data.len() > 100, "페이지 표지는 1137 bytes 이상이어야 함");
    assert_eq!(first.extension, "png");

    // SVG export 시 첫 페이지에 정확한 PNG 데이터가 포함되는지 확인
    let svg = doc.export_page_svg(0, &Default::default()).expect("SVG 출력");
    // 16x13 작은 PNG (회귀 시 들어갔던 이미지) 이 아닌, 정상 페이지 표지가 들어가야 함
    assert!(!svg.contains(r#"width="16" height="13""#),
        "회귀: 16x13 픽셀 잘못된 PNG 가 페이지 배경에 들어감");
}
```

(실제 API 시그니처는 단계 진행 중에 점검 — 위는 의사코드 형태)

### 변경 4 — `hwpspec.hwp` SVG 시각 검증 (수동)

```bash
cargo run --release --bin rhwp -- export-svg samples/hwpspec.hwp -p 0 --debug-overlay -o output/svg/issue-416/
```

작업지시자가 `output/svg/issue-416/hwpspec_001.svg` 시각 확인.

### 변경 5 — HWPX 차트 회귀 점검

본 저장소 샘플에 HWPX 차트 없으므로:
- Stage 1 의 단위 테스트 (`find_bin_data_sparse_id_for_hwpx_chart`) 로 sparse id 경로 검증
- 추가 시 작업지시자가 차트 포함 hwpx 보유 시 별도 회귀 테스트 후속

### Stage 2 검증

- 회귀 테스트 통과
- 시각 검증 — 페이지 표지 이미지 정상 표시
- svg_snapshot 6/6 통과 (다른 샘플 무회귀)

## Stage 3 — 자동 검증 / 시각 검증 / 트러블슈팅 갱신

### 자동 검증

| 검증 | 명령 |
|------|------|
| cargo lib test | `cargo test --lib` (1016+ → 1020+ 으로 증가, 무회귀) |
| svg_snapshot | `cargo test --test svg_snapshot` (6/6) |
| clippy | `cargo clippy --lib -- -D warnings` |
| 신규 회귀 테스트 | `cargo test --test issue_416_page_bg_bin_data` (또는 lib 안) |

### 시각 검증

작업지시자 환경에서 `samples/hwpspec.hwp` 1 페이지를 dev server / SVG 로 확인:
- 페이지 표지가 정상 PNG (159×247.6mm) 로 표시
- 16×13 작은 이미지 늘려진 형태가 아님

### 변경 6 — 트러블슈팅 문서 갱신

`mydocs/troubleshootings/bin_data_id_index_mapping.md` 끝에 이력 추가:

```markdown
## 회귀 이력

### 2026-04-20 — Task #195 fallback 도입 (회귀 origin)

커밋 `5c72f48` 에서 HWPX 차트 sparse id (60000+N) 처리를 위해 `find_bin_data` 에 fallback 추가.
이때 가드 `c.id == bin_data_id` 가 일반 HWP 그림의 정상 케이스도 거르는 부작용 발생.

- 결함: 인덱스로 접근한 항목의 `c.id` (=storage_id) 가 `bin_data_id` (=인덱스) 와 다르면 fallback 으로 빠짐 → storage_id 검색 (본 트러블슈팅이 정정한 그 결함 패턴)
- 영향: storage_id ≠ 인덱스인 모든 hwp (예: hwpspec.hwp 의 페이지 표지)

### 2026-04-28 — Task #416 재정정

- 가드 제거: 인덱스 범위 안이면 무조건 인덱스 매칭
- 인덱스 범위 밖 (HWPX 차트 sparse id 60000+N) → id 직접 검색 fallback 보존
- 단위 테스트 4 개 추가 (인덱스 매칭 / sparse id / 범위 밖 / 0)

## 교훈

- HWP 그림의 `bin_data_id` 와 HWPX 차트의 sparse id 는 **다른 매칭 방식** 이 필요
- fallback 추가 시 가드는 신중히 — 정상 케이스를 거르지 않도록
- 동일 결함이 재발할 가능성에 대비해 단위 테스트로 회귀 방지
```

### Stage 3 검증

- 트러블슈팅 문서 정확히 갱신
- 모든 자동 검증 통과
- 작업지시자 시각 판정 통과

## 산출물

| 단계 | 산출물 |
|------|--------|
| Stage 1 | `mydocs/working/task_m100_416_stage1.md` |
| Stage 2 | `mydocs/working/task_m100_416_stage2.md` |
| Stage 3 | `mydocs/working/task_m100_416_stage3.md` |
| 최종 | `mydocs/report/task_m100_416_report.md` |
| 트러블슈팅 갱신 | `mydocs/troubleshootings/bin_data_id_index_mapping.md` |

## 위험 / 주의

### 1. HWPX 차트 회귀 위험

- 본 저장소에 차트 포함 HWPX 샘플 없음 → 단위 테스트로만 검증
- 작업지시자가 차트 hwpx 보유 시 별도 시각 검증 권장

### 2. 다른 hwp 샘플의 회귀 위험

- 대부분의 hwp 는 storage_id == 인덱스 → 가드가 통과해 왔음 → 본 정정으로 인덱스 매칭만 사용해도 동일 결과
- svg_snapshot 6 샘플 모두 무회귀 점검

### 3. 회귀 테스트의 견고성

- `hwpspec.hwp` 가 `samples/` 에 없는 환경에선 skip → CI 안정성 확보
- 단위 테스트 (BinDataContent 직접 생성) 는 환경 의존성 없음

## 다음 단계

본 구현 계획서 승인 → Stage 1 진행.
