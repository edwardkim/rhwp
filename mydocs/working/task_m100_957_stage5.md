# Task #957 Stage 5 — 시각 검증 + 최종 작업 정리

## 1. 시각 검증 결과 (작업지시자 확인)

- sample16 page 18 SVG → PNG render
- 다이어그램 + "나. 주요 과업내용" + ○ 통합모델... + 본문 paragraphs 모두 같은 페이지에 표시
- 한컴 viewer page 16 정합 확인

## 2. 최종 변경 요약

### 2.1 코드 변경
`src/renderer/layout.rs` 라인 3465-3486:
- **Fix A**: 빈 caption 시 result_y advance skip
- `caption_is_empty` 가드 추가
- `RHWP_DEBUG_TAC_CURSOR` 진단 영구화

### 2.2 문서 추가
- `mydocs/plans/task_m100_957.md`
- `mydocs/plans/task_m100_957_impl.md`
- `mydocs/plans/task_m100_957_impl_v2.md`
- `mydocs/working/task_m100_957_stage1.md`
- `mydocs/working/task_m100_957_stage4.md`
- `mydocs/working/task_m100_957_stage5.md`
- `mydocs/report/task_m100_957_report.md`
- `mydocs/orders/20260517.md` 갱신

## 3. PR 준비

base = upstream/devel
head = jangster77:local/task957
title: fix: 빈 caption phantom advance 정정 — sample16 page 18 본문 다음 페이지 밀림 해소 (closes #957)
