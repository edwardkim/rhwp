---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3308.md
last_verified: 2026-08-01
---

# Task #3308 Stage 2 보고 — 비율 하한 + 셀 내 가운데 구현

## 구현 (승인된 설계)

1. `render_normalization.rs` — 스트레치 조건에 **하한
   `NESTED_STRETCH_MIN_RATIO = 0.9`** 추가. 근소 미달(#2195 보호 대상 0.956~0.995)만
   셀 폭으로 확장.
2. `layout/table_layout.rs` `compute_table_x_position` 중첩 분기 — 하한 미만
   비-TAC 중첩 표는 **선언 폭 유지 + 셀 내 가운데** (`area_x + (area_w−w)/2`).
   발동을 비-TAC·셀 내부·비율<0.9 로 좁게 가드(케이스별 원칙). 근거 주석에
   3중 권위(편집기 판독·재저장·정답지 0.6px)와 "저장 h_offset 은 한컴도 미사용"
   판정 명기.

## fixture 효과

| 측정 | 수정 전 | 수정 후 | 한컴 |
|---|---|---|---|
| 중첩 표 폭 | 641.3 (스트레치) | **435.1** | 435.1 (115.13mm) |
| 중첩 표 x | 102.0 | **200.4** | ≈198.9 |
| **직인 텍스트 x** | 696.2 | **600.2** | **598.7 (차 1.5px ≤ 계약 5px)** |
| 용지규격 x | 590.9 | 590.9 불변 | 590.7 |

## red-check — 두 축 독립 증명

`tests/issue_3308_nested_table_width.rs` (render tree JSON 좌표 계약, rhwp_bin 규약):

| 제거 축 | 결과 |
|---|---|
| 하한 가드 제거 | 직인 x=696.2 — **원결함 정확 재현** FAILED |
| 가운데 배치 제거 | 직인 x=501.8 — **좌편향** FAILED (가운데가 load-bearing 증명) |
| 복원 | ok |

(오늘 교훈 반영: 수정 커밋 후 red-check — `git checkout` 원복 안전.)

## 회귀 확인 (focused)

- **#2195 쪽수 게이트**(issue_1891 규제영향분석서 82·157쪽 + issue_1842): 6 passed
- 표/중첩 계열 테스트 전체: **91 passed / 0 failed**
- 정규화 단위 테스트: fixture 를 스트레치 창(0.95)으로 정렬 + **하한 경계 계약
  테스트 신설**(0.679 제외 / 0.956 유지) — 5 passed
- fmt 통과

## Stage 3 계획 (승인 필요 게이트 포함)

정답지 이미지 스왑(p7 직인 위치) · samples 쪽수 A/B ·
`cargo test --tests` 전체(PR CI급 — **별도 승인**) · clippy · Skia 3종 · wasm 재빌드 ·
이중 baseline 확인(fixture 신규 없음).
