# 구현 계획서 v2 — Task #1238: 미주 between-notes margin 누락 (min-gap 모델)

- **이슈**: #1238 (M100 / v1.0.0)
- **브랜치**: `feature/issue-1238-between-notes-margin`
- **선행**: `plans/task_m100_1238_impl.md`(v1, 가산 모델 — **반증·폐기**), `tech/between_notes_multiline_1238.md` §7
- **작성일**: 2026-06-02

## 배경 — v1 폐기 사유

v1 의 "다줄 마지막 줄 trailing 복원"(가산) 접근은 두 변형(blanket / `pagination_gap==0` 게이트)
모두 회귀 가드 4건(issue_1139·1189)을 **동일하게** 깨뜨려 반증됨. 세 문서 모두 경계에서
`pagination_gap==0` → 해당 판별자는 무효. 실측(tech §7.1)으로 between-notes 는 **가산이 아니라
min-gap(클램프)** 임이 확정됨.

## 설계 — min-gap 클램프

미주 사이 간격을 **최소 between_notes(7mm≈26.5px) 로 보장**하되, 자연 간격이 이미 그 이상이면
변경하지 않는다.

```
새 미주의 첫 문단 시작 y := max(incoming_y, prev_endnote_last_content_bottom + between_notes_px)
```

| 문서 | natural gap | max(·, 26.5) | 효과 |
|------|-------------|--------------|------|
| 3-11월 문22 (target) | 0.0 | 26.5 | ✅ 수정 |
| 3-09월 문15 (guard) | 26.5 | 26.5 | 무변경 ✅ |
| 3-10월 문19 (guard) | 52.9 | 52.9 | 무변경 ✅ |

가산이 깨뜨린 지점(이미 ≥7mm 인 경계 과대가산)을 정확히 회피한다.

## 단계 구성 (3단계)

### Stage 1 — 재개 완료 (코드 무변경)
- 산출: `tech/between_notes_multiline_1238.md` §7 (실측 표 + min-gap 모델 + 잔여 리스크).

### Stage 2 — min-gap 클램프 구현
1. **typeset.rs**: between-notes 가 적용되는 **새 미주의 첫 문단**의 렌더 para_index(=`paragraphs.len()
   + local`)와 적용할 `between_notes_px` 를 수집 → `PaginationResult` 신규 필드로 전달
   (v1 (B)의 배선 패턴 재사용. 단 의미는 "첫 문단 + 마진값").
2. **layout.rs**: 세터 + 조회 헬퍼. `endnote_para_base` 기반 절대 인덱스 맵
   `para_index -> between_notes_px`.
3. **paragraph_layout.rs**: 미주 문단 진입부에
   - "직전 미주 마지막 content bottom"을 상태로 추적(미주 흐름 한정),
   - 현재 문단이 "새 미주 첫 문단" 집합에 속하면 시작 y 를
     `max(y, prev_endnote_last_bottom + between_notes_px)` 로 클램프.
   - **가산 아님** — 이미 충분하면 no-op.
- **검증**: 전체 `cargo test`(가드 4건 + 골든 포함) 통과, 문22 gap 0→~26.5px.

### Stage 3 — 시각·회귀·문서화
- **시각**: 3-11월 14쪽 문22(외 다줄로 끝나는 미주) above-gap PDF(한글 2022) 정합 + 3-09·10월 비회귀.
- **회귀**: 골든 + 전체 `cargo test`. 특히 issue_1139(문15 [24,32]), 1189(q18→q19 [205,235],
  q19→q20 [180,210]) 와 페이지 수 불변.
- **산출**: `report/task_m100_1238_report.md`.

## 회귀 가드

| 위험 | 방어 |
|------|------|
| 이미 ≥7mm 경계 과대 (v1 실패 원인) | max() no-op — 가산 아님 |
| 3-10월 문18(gap=0) 클램프 → q간격/overflow (tech §7.3) | 가드 4건 + 골든이 PDF 캘리브레이션값으로 직접 검출. 위반 시 즉시 보고·재설계 |
| 누적 밀림 페이지 수 변동 | issue_1139 page-count + 골든 스냅샷 |
| #1236(PR#1240) 머지 | 클램프는 미주-**사이** 진입부, #1236 은 미주-**내부** trailing → 직교 |

## 단계별 커밋 정책

각 Stage 소스 + `working/task_m100_1238_stage{N}.md` 동반 커밋. 무관 rustfmt diff 금지.
