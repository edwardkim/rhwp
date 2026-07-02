# Task #1658 v3 3단계 완료 보고 — 통제 검증 (채택 게이트)

## 결과 — 전 게이트 통과, 악화 0

| 게이트 | 기준 | 결과 |
|--------|------|------|
| render_page_gate 소형(92) | 일치·under 무회귀 + over 해소 | **일치 73→75(+2), over 5→3, under 14 불변** — 파일별 대조 악화 0 ✅ |
| render_page_gate 대형(452) | 443+ 무회귀 | **443 (98.0%) 불변** ✅ |
| clipping_gate (92) | 회귀 0 | 검사 92 / 회귀 0 / baseline 이탈 0 ✅ |
| valign_offset_gate | fixture 4종 유지 | over 3종 FIX + 가드 OK (BUG 0/회귀 0) ✅ |
| 페이지네이션 샘플 | byeolpyo 4/26, giant 42, scattered 53 | 전부 무회귀 ✅ |
| `issue_1611_footer_page_bottom_pagination` | 2쪽 유지 | 통과 ✅ |
| 신규 가드 3건 + opengov 22건 | GREEN | 통과 ✅ |
| `cargo test --release` 전체 | 실질 실패 0 | **2754 passed / 실질 실패 0** (7건 = svg_snapshot CRLF 노이즈 #1786, 내용 diff 0 — 렌더 변경의 골든 영향 없음 확인) ✅ |
| `rustfmt --check` (변경 파일) | 통과 | 통과 ✅ |

## 개선 상세 (파일별 대조, `output/poc/task1658_v3/controlset_baseline.tsv` 기준)

| 파일 | 전 | 후 | 한글 |
|------|---:|---:|---:|
| 관악소방서 현장대응단 36389312 | 2 | **1** | 1 |
| 디지털도시국 데이터전략과 36398366 (PC셧다운, RCA 원본) | 2 | **1** | 1 |

- 그 외 90건 전부 페이지 수 불변 (Stage 2b 의 75→55 대규모 under 회귀 재발 없음).
- 잔여 over 3건(+1 1건, +2 2건)은 하단 고정 틀과 무관한 별개 원인 — 후속 분류 대상.

## Stage 2b(#1653) 실패와의 대조

| | Stage 2b (일괄 out-of-flow) | 본 라운드 (하단 배타 예약) |
|--|--|--|
| 소비 모델 | 0 (배타 없음) | 0 + **배타 영역(max-union) 차감** |
| vpos 동기화(#1611) | 우회 | **계승 + 선행 틀 소비 차감 보정** |
| 결과 | 일치 75→55, under 12→34 | 일치 +2, under 불변, 악화 0 |
