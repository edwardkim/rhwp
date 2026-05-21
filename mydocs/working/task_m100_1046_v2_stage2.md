# Stage 2 완료보고서 — #1046: 표 첫행 강제배치 → 조건부 이월 (가용공간 overhead 정합)

- 타스크: #1046 (M100), 브랜치 `local/task1046`
- 단계: 측정 통일(B) Stage 2 — 진단 확정 후 수정 (작업지시자 승인: 루프 진입부 가드)
- 작성일: 2026-05-21

## 1. 수정 내용 (1지점, typeset.rs `place_table…` 분할 진입부)

분할 진입부의 기존 pre-loop 가드(첫 행이 잔여공간보다 크면 다음 페이지로 이월)가
`remaining_on_page = table_available - current_height` 만 사용해 **첫(비연속) fragment 의
렌더러 y_start 점프**(host_spacing.before + TopAndBottom·vert=Para·v_off>0 표의
vertical_offset)를 차감하지 않았다. 그 결과 잔여를 과대평가해 첫 행이 실제로 안 들어가는데도
가드를 통과 → 일반 행 강제 분기(`!splittable && r==cursor_row`)가 통째로 밀어넣어 본문 초과.

수정: 가드의 `remaining_on_page` 에서 첫 fragment overhead(host_before + vert_offset)를
차감 — 루프 내 `page_avail`(host_before_overhead/vert_offset_overhead)과 동일 overhead.

```
remaining_on_page = (table_available − current_height − (host_before + vert_offset)).max(0)
```

genuine page-larger(행/블록 > base_available)는 가드 자체가 발동하지 않아 기존 강제 유지 →
회귀 없음. 측정·강제 로직은 손대지 않았다.

## 2. 확정 진단 (pi=242 SIR-002, page 28→29)

- 종전: `remaining_on_page=941.1−875.7=65.4` (overhead 미차감) ≥ split_unit_h 34.9 →
  가드 미발동 → 강제 배치 → 렌더러 82.8px(=vert_off 38.1 + 행0 34.9 + host) → **19.2px 초과**.
- 수정: `65.4 − (host_before 4.0 + vert_off 38.1) = 23.4 < 34.9` → 가드 발동 →
  `advance_column_or_new_page()` → 표 전체 page29 로 이월 → **통째(518.9px) 배치, overflow 0**.

## 3. 결과 (대상 샘플, 비공개 185p)

| 지표 | baseline | 수정 후 |
|------|----------|---------|
| LAYOUT_OVERFLOW 총건 | 18 | **16** |
| in-scope (page-larger 제외) | 16 | **14** |
| 해소 | — | pi=242(19.2), pi=256(6.9) |
| page-larger (pi=323, pi=567) | 2 | 2 (불변) |
| 신규 overflow | — | 0 |
| 총 페이지 수 | 185 | 185 (불변) |

## 4. 회귀·정합 검증

- `cargo test --release`: **1516 passed / 0 failed** — 골든 SVG 회귀 0.
- 한컴 2022 PDF(`pdf/…제안요청서-2022.pdf`) 대조: SIR-002 표가 한 페이지에 **통째** 배치
  (PDF index 28, 요구사항번호~산출정보 전체). 동일 구조 SFR-013~017 표도 모두 통째 배치 +
  안 들어가면 다음 페이지로 통째 이동 → 수정 동작과 정합. **분할 미발생이 정답**.

## 5. 잔여 (다음 단계 후보)

in-scope 14건(218/266/290/308/361/429/600/781 + 섹션1 268/354/357/406) — vert_offset 형이
아닌 다른 패턴 가능. page-larger 2건은 범위 외(별도 이슈). 다음 단계에서 잔여 분류 후
공통 패턴 추가 정합 검토.
