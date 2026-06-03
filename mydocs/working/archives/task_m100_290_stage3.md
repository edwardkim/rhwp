---
타스크: #290 cross-run 탭 감지가 inline_tabs 무시
단계: 3 / 4 — 시각 회귀 검증
브랜치: local/task290
작성일: 2026-04-24
---

# Stage 3 완료 보고서

## 1. 목표

구현계획서 Stage 3 의 검증 3 항목 완료:

1. `samples/exam_math.hwp` p.7 item 18 통합 테스트 추가 + pass
2. exam_math.hwp 20 페이지 + 주요 샘플 before/after SVG 비교
3. exam_math p.7 before/after/PDF 3 면 시각 비교 PNG

## 2. 통합 테스트

### 2.1 신규 파일 `tests/tab_cross_run.rs`

```rust
#[test]
fn task290_exam_math_p7_item18_not_right_aligned() {
    // exam_math.hwp 페이지 7 을 렌더 후 item 18 의 "수" 글리프 x 좌표를 검사.
    // 수정 전: x ≈ 290.9 (우측 정렬 버그)
    // 수정 후: x ≈ 109.8 (좌측 정렬 정상)
    // 임계치 < 200 으로 회귀 방지.
}
```

### 2.2 결과

```
$ cargo test --test tab_cross_run
running 1 test
test task290_exam_math_p7_item18_not_right_aligned ... ok
test result: ok. 1 passed; 0 failed
```

## 3. 회귀 검증 (before/after SVG 비교)

`git worktree` 로 Stage 1 커밋 (7d3bbba, 수정 전) baseline 생성 → exam_math + 주요 샘플 전체 페이지 SVG 비교.

### 3.1 결과 — 총 184 페이지 중 **1 페이지만 변경**

| 문서 | 변경 / 전체 | 비고 |
|------|------------|------|
| exam_math.hwp | **1 / 20** | p.7 만 (item 18 의도된 수정) |
| biz_plan.hwp | 0 / 6 | 변화 없음 |
| exam_eng.hwp | 0 / 11 | 변화 없음 |
| exam_kor.hwp | 0 / 25 | 변화 없음 |
| hwp-3.0-HWPML.hwp | 0 / 122 | **RIGHT inline tab (저작권\t1) 회귀 없음** |
| **합계** | **1 / 184** | 모든 의도 외 변화 0 |

### 3.2 exam_math.hwp p.7 변경 내용

`diff before after` 결과 — 14 줄 변경 (7 쌍). 모두 item 18 첫 줄의 좌측 정렬 이동:

| 글리프 | before x | after x | 설명 |
|--------|----------|---------|------|
| 수 | 290.91 | **109.80** | -181 px 이동 (좌측 정렬) |
| 열 | 304.86 | **123.75** | -181 px |
| 이 | 354.73 | **173.63** | -181 px |
| 모 | 375.36 | **194.25** | -181 px |
| 든 | 389.31 | **208.20** | -181 px |
| 자 | 409.93 | **228.83** | -181 px |
| 연 | 423.89 | **242.78** | -181 px |
| 수 | 437.84 | **256.73** | -181 px |
| (수식) | 458.73 | **277.63** | -181 px |
| 에 | 468.56 | **287.45** | -181 px |
| 대 | 489.18 | **308.08** | -181 px |
| 하 | 503.14 | **322.03** | -181 px |
| 여 | 517.09 | **335.98** | -181 px |
| (g tr) | 325.91 | **144.80** | 수식 래퍼 위치 |

일관되게 **-181.11 px** 만큼 좌측 이동 — cross-run RIGHT 오판으로 인한 역산 배치(420.11 기반) 제거 후 정상 좌측 진행 위치로 복원.

### 3.3 hwp-3.0-HWPML.hwp RIGHT inline 탭 회귀 확인

페이지 3 저작권 줄 확인:
- "저" glyph at x=102.67 (유지)
- "1" glyph at x=663.04 (우측 정렬 유지)

122 페이지 전체 byte-identical → RIGHT inline 탭 경로 **회귀 0**.

## 4. 시각 비교 PNG (before/after/PDF)

`mydocs/working/task_m100_290_stage3/` 에 3 면 저장:

- `p7_before.png` — 수정 전 SVG 렌더 (item 18 "수열" 우측 끝 정렬 버그)
- `p7_after.png` — 수정 후 SVG 렌더 (PDF 와 동일한 좌측 정렬)
- `p7_pdf.png` — 한컴 PDF 레퍼런스

시각 확인:
- **BEFORE**: `18.` ······················· `수열 {a_n}이 모든 자연수 n에 대하여`
- **AFTER**: `18. 수열 {a_n}이 모든 자연수 n에 대하여` (PDF 와 일치)

## 5. 전체 검증 요약

| 항목 | 결과 |
|------|------|
| 단위 테스트 `cargo test --lib task290` | 5/5 pass |
| 통합 테스트 `cargo test --test tab_cross_run` | 1/1 pass |
| SVG snapshot `cargo test --test svg_snapshot` | 3/3 pass |
| 전체 `cargo test --lib` | 955 pass (선존재 14 fail 별개) |
| clippy `cargo clippy --lib -- -D warnings` | clean |
| 회귀 (4 문서 184 페이지) | 1/184 변경 (의도된 수정만) |
| 시각 검증 (3 면 PNG) | AFTER = PDF 일치 |

## 6. 다음 단계 예고

Stage 4 에서:
- 최종 결과 보고서 `mydocs/report/task_m100_290_report.md` 작성
- `mydocs/orders/20260424.md` 에 #290 완료 항목 갱신 (신규 등록 → 종료)
- `mydocs/troubleshootings/tab_tac_overlap_142_159.md` 에 #290 섹션 추가
- 이슈 #290 close
- 모든 stage 커밋 확인 + `git status` 클린

## 7. 승인 요청

Stage 3 결론:
- 통합 테스트 추가 + pass ✓
- 184 페이지 회귀 검증 — 변경 1 페이지 (의도된 item 18 수정) ✓
- RIGHT inline 탭 회귀 0 ✓
- 시각 비교 (BEFORE/AFTER/PDF) — AFTER 가 PDF 와 일치 ✓

Stage 4 (최종 보고 + 이슈 close) 진행 승인 요청.
