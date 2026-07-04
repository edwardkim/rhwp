# Task #1598 Stage 3 — 회귀 검증 (baseline/lib)

## 결과 (전부 PASS, 회귀 0)

| 검증 | 결과 |
|------|------|
| `issue_1598_ellipse_geometry_roundtrip` | 1 passed |
| `hwpx_roundtrip_baseline` (도형 포함 전수) | 4 passed |
| `issue_1392_shape_comment_roundtrip` | 2 passed |
| `issue_1403_pic_shape_caption_roundtrip` | 3 passed |
| `issue_1385_replace_export_roundtrip` | 1 passed |
| `opengov_corpus_snapshot` (36385226 행 추가) | 2 passed |
| **전체 `cargo test --lib`** | **1970 passed, 0 failed, 7 ignored** |
| `cargo clippy` (변경 코드) | warning 0 |
| `cargo fmt --check` (실 툴체인) | diff 0 |

ellipse/arc 지오메트리 추가가 polygon/curve/rect/picture/group/chart/ole 경로 및 IR diff
게이트에 무영향 확인. IR diff=0 유지(지오메트리는 IR-invisible 이라 diff 카운트 불변).

> 주: 직접 `rustfmt --edition 2021` 호출은 1275/1361(#1596 기존 코드)에 diff 보고하나,
> 저장소 `rust-toolchain.toml`/`rustfmt.toml` 기준 `cargo fmt --check` 는 clean. 제 신규
> 코드는 양쪽 모두 clean.
