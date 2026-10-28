# Task #1591 — Stage 2 완료보고서

**단계**: 수정 설계·구현 (북마크 슬롯 편입)
**브랜치**: `local/task1591`

## 변경 내용

| # | 파일 | 변경 |
|---|------|------|
| C1 | `serializer/hwpx/section.rs` | 북마크 hoisting 루프(416-426) **제거** |
| C2 | `serializer/hwpx/section.rs` | `is_hwpx_inline_slot` 에 `Control::Bookmark` 추가 |
| C3 | `serializer/hwpx/section.rs` | `render_control_slot` 에 `Control::Bookmark` arm 추가(`<hp:ctrl><hp:bookmark/></hp:ctrl>`) |
| C4 | `serializer/hwpx/roundtrip.rs` | diff 비교는 종전대로 북마크 **제외**(보수적 결합 분리) — `is_hwpx_inline_slot(c) && !Bookmark` |

## 설계 — 보수적 결합 분리

`diff_documents` 가 `is_hwpx_inline_slot` 을 공유하므로, 북마크 슬롯 편입(C2)이 비교 의미까지
바꿔 `diff_documents_bookmark_not_compared_as_control` 테스트가 깨졌다. 직렬화기는 북마크를
정위치 방출하되 **diff 비교는 종전대로 북마크 제외**(게이트 의미 불변)하도록 C4 로 분리.
북마크 자체 보존 비교는 별도 과제로 남긴다(scope 최소화).

## 검증 (모두 GREEN)

| 검사 | 결과 |
|------|------|
| `task1591_bookmark_not_hoisted_before_slot` | PASS (`[tbl,bm]` 순서 보존) |
| bookmark 관련 6 테스트 | 전부 PASS |
| `cargo test --lib` | 1964 passed, 0 failed |
| `hwpx_roundtrip_baseline` | 4/4 |
| `cargo clippy --lib` | 무경고 |

## 다음 단계

Stage 3 — fidelity 전수 통제 비교(롤백 가드): C1(36384689·36385445) 해소 + 악화 0 확인.
악화 ≥1 시 전량 롤백.
