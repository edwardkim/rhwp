# PR #374 검토 — Task #362: kps-ai p56 외부 표 안 콘텐츠 클립 회귀 (TAC 셀 vpos 클램프)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#374](https://github.com/edwardkim/rhwp/pull/374) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| base / head | `devel` ← `local/task362` |
| state | OPEN |
| mergeable | **CONFLICTING** (DIRTY) |
| 이슈 | [#362](https://github.com/edwardkim/rhwp/issues/362) (이미 CLOSED) |
| 처리 결정 | **close + 흡수 가치 없음** (옵션 B) |
| 변경 통계 | +6 / -2, 1 file (`src/renderer/layout/table_layout.rs`) |

## 결함 요약

`samples/kps-ai.hwp` 56쪽 외부 표 (pi=535, 1×1 TAC) 안 콘텐츠가 외부 셀 경계 초과해 클립 발생. Task #347 의 `cell_y + pad_top + LineSeg.vertical_pos[0]` 적용이 콘텐츠 꽉 찬 케이스에서 부작용.

## 변경 내용

`src/renderer/layout/table_layout.rs::layout_table_cells` 의 `text_y_start` 계산:

```diff
let text_y_start = if let Some(vpos) = first_line_vpos.filter(|&v| v > 0.0) {
-    // vpos는 셀 컨텐츠 상단(=cell_y+pad_top)으로부터의 첫 줄 top y 오프셋
-    cell_y + pad_top + vpos
+    let remaining_room = (inner_height - total_content_height).max(0.0);
+    cell_y + pad_top + vpos.min(remaining_room)
} else { ... }
```

vpos 를 `inner_height - total_content_height` 로 clamp.

## **메인테이너 작업과 중복 — Task #362 (이미 v0.7.7 에 적용)**

이슈 #362 의 결함은 메인테이너 Task #362 로 v0.7.7 (2026-04-27 배포) 에 정정 완료. 이슈 #362 도 이미 CLOSED.

| 항목 | PR #374 | 메인테이너 Task #362 |
|------|---------|--------------------|
| 결함 | kps-ai p56 외부 표 클립 | 동일 |
| 정정 위치 | `text_y_start` (`table_layout.rs:1284-`) | 동일 |
| 접근 | `vpos.min(remaining_room)` clamp (일반화) | `if !has_nested_table` 가드 (구조 명시) |
| 정정 범위 | p56 단일 case | p56 + p67 + p68-70 + p72-73 (8 항목 누적) |
| 추가 정정 | 없음 | PartialTable nested 분할, wrap-around (Square wrap), hide_empty_line, vpos-reset 가드, 빈 paragraph skip 가드 강화 등 7 항목 |

## 두 접근의 시멘틱 비교

### PR #374: `vpos.min(remaining_room)` (일반화 clamp)

`remaining_room = (inner_height - total_content_height).max(0.0)`

- **측정 의존**: `total_content_height` 의 정확도에 따라 clamp 결과 변동
- **암묵적 시멘틱**: 셀이 꽉 찼는지 여부를 측정값 차이로 추론

### 메인테이너 Task #362: `if !has_nested_table` (구조 명시)

```rust
let has_nested_table = cell.paragraphs.iter()
    .any(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))));
let text_y_start = if !has_nested_table && first_line_vpos.filter(|&v| v > 0.0).is_some() {
    cell_y + pad_top + first_line_vpos.unwrap()
} else { ... }
```

- **구조 의존**: nested table 존재 여부만 판단 (측정값 의존 없음)
- **명시 시멘틱**: nested table 셀 = vpos 미적용 (의도 명확)

## 회귀 위험 분석 — 일반화 접근 vs 케이스별 가드

작업지시자 관측 (메모리 등록): **한컴 호환은 일반화보다 케이스별 명시 가드가 안전**. 한컴 자체의 비일관성으로 일반화 알고리즘이 다른 케이스에서 회귀 발생.

PR #374 의 잠재 회귀:

| 측정 결함 케이스 | PR #374 영향 |
|---|---|
| `total_content_height` 가 실제보다 작게 계산 (nested table 측정 결함) | `remaining_room` 이 실제보다 큼 → vpos clamp 약함 → **여전히 클립** |
| `total_content_height` 가 실제보다 크게 계산 | `remaining_room=0` → vpos 무시 → Task #347 의 효과 손실 (exam_eng p4 등) |
| PartialTable nested 분할 (kps-ai p67) | clamp 시점에 분할된 `total_content_height` 부정확 |
| Square wrap 흡수 (kps-ai p68-70) | wrap-around paragraph height=0 흡수와 `total_content_height` 가 어긋남 |
| hide_empty_line 적용 (kps-ai 후속) | height=0 처리와 `total_content_height` 어긋남 |

→ PR #374 단독 머지 시 본 case (p56) 만 정정되고 다른 7 항목 (p67, p68-70, p72-73 등) 은 여전히 결함.

## 흡수 가치 평가

PR #366 (page_number) 처럼 흡수할 가치:
- **모듈 추출 가치**: 없음 (단일 6 줄 변경)
- **회귀 테스트 신설**: 없음
- **다른 경로 정정**: 없음
- **시멘틱 가치**: 메인테이너 명시 가드보다 약함 (측정 의존)

→ **흡수 가치 없음**.

## 처리 결정 — 옵션 B (close)

### 이유
1. **이슈 #362 이미 CLOSED** — 메인테이너 Task #362 (8항목 누적) 로 v0.7.7 에 정정 완료
2. **시각 판정 통과** — 작업지시자가 v0.7.7 에서 p56, p67, p68-70, p72-73 모두 정상 확인
3. **흡수 가치 없음** — 일반화 clamp 시멘틱이 메인테이너의 명시 가드보다 약함, 흡수 시 회귀 위험 증가
4. **메모리 원칙 부합** — 한컴 호환은 일반화보다 케이스별 명시 가드가 안전

## PR 댓글 (간결)

```
kps-ai p56 외부 표 클립 회귀 정정 PR 검토했습니다.

본 결함은 메인테이너 Task #362 (8 항목 누적: vpos 가드, PartialTable nested 분할,
Square wrap 어울림, hide_empty_line 등) 로 v0.7.7 (2026-04-27 배포) 에 정정 완료
되었습니다. 이슈 #362 도 close 상태입니다.

본 PR 의 vpos clamp 접근 (`vpos.min(remaining_room)`) 과 메인테이너의 nested table
가드 (`has_nested_table` 분기) 는 다른 시멘틱입니다. 메인테이너 작업이 p56 외에
p67 (PartialTable nested 분할), p68-70 (Square wrap), p72-73 (표 누락 차단) 등
8 항목 광범위 정정을 포함하므로 본 PR 은 close 합니다.

상세: mydocs/report/task_m100_362_report.md
감사합니다.
```

## 처리 단계

1. 본 검토 문서 commit
2. PR 댓글 (위) + close
3. `pr_374_report.md` 작성
4. local/devel commit + push

## 참고

- 이슈: [#362](https://github.com/edwardkim/rhwp/issues/362) (CLOSED)
- PR: [#374](https://github.com/edwardkim/rhwp/pull/374) (OPEN, DIRTY)
- 메인테이너 Task #362: `mydocs/report/task_m100_362_report.md`
- 메모리: `feedback_hancom_compat_specific_over_general.md`
