# Task #370 Stage 1 완료 보고서

## 변경 사항

`src/renderer/mod.rs`의 한글 세리프 폴백 체인 두 분기에 `'Noto Serif CJK KR'` 추가.

| 위치 | 기존 | 신규 |
|------|------|------|
| `:558` (한글 세리프 키워드) | `…'Noto Serif KR',serif` | `…'Noto Serif KR','Noto Serif CJK KR',serif` |
| `:565` (영문 세리프 키워드) | `…'Noto Serif KR',serif` | `…'Noto Serif KR','Noto Serif CJK KR',serif` |
| `:935` (테스트) | (위와 동일) | (위와 동일) |

## 테스트 결과

- `cargo test test_generic_fallback` — ok (1 passed)
- `cargo test` 전체 — **1055 passed, 0 failed** (스냅샷 3건 갱신 후)

## 골든 스냅샷 갱신

폰트 체인 문자열 변경에 따른 차이 외에는 변경 없음을 `diff`로 확인 후 `UPDATE_GOLDEN=1`로 갱신:

- `tests/golden_svg/form-002/page-0.svg`
- `tests/golden_svg/issue-157/page-1.svg`
- `tests/golden_svg/issue-267/ktx-toc-page.svg`

## 비고

- `cargo clippy`의 44건 에러는 기존 회귀(`convert_to_editable_native` 등)로 본 타스크와 무관함을 stash 비교로 확인.
