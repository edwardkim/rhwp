# Task #477 Stage 3 — 광범위 회귀 검증

## 검증 게이트

| 검증 | 결과 |
|------|------|
| `cargo test --lib` | **1078 passed** (이전 1075 + 신규 단위 테스트 3 추가) ✅ |
| `cargo test --test svg_snapshot` | 6/6 ✅ |
| `cargo test --test issue_418` | 1/1 ✅ |
| `cargo clippy --lib -- -D warnings` | 0건 ✅ |

## 단위 테스트 추가 / 갱신

`src/renderer/svg/tests.rs`:

| 테스트 | 의도 | 결과 |
|--------|------|------|
| `test_compute_image_crop_src_kwater_pi31` (신규) | 케이스 A — image = crop 후 image | ✅ 12660/75 = 168.8 |
| `test_compute_image_crop_src_exam_kor_header` (갱신) | 케이스 B — image = 원본, Task #430 의도 보존 | ✅ 102366/75 = 1364.88 |
| `test_compute_image_crop_src_no_crop_full_image` (갱신) | 전체 영역 | ✅ |
| `test_compute_image_crop_src_offset_top_left` (갱신) | 양방향 crop offset | ✅ |
| `test_compute_image_crop_src_fallback_when_original_size_missing` (갱신) | original_size_hu 미공급 시 폴백 | ✅ |

## SVG byte 회귀 검증

룰 정정으로 SVG 출력 변화:
- **k-water-rfp 1쪽**: md5 `c2653b6...` → `071f31d...` (회귀 정정)
- **k-water-rfp 16쪽**: 외부 SVG 폭 623.61 → 606.16 (셀 폭 클램프 정정)

다른 샘플 (synam-001, exam_kor 등) 의 byte 변화는 단위 테스트로 정합 확인 (exam_kor 헤더 케이스 보존).

## 다음 단계

- **Stage 4**: 작업지시자 시각 검증 — 통과 ✅
- **Stage 5**: 최종 결과보고서 + 오늘할일 갱신
