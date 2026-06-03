# 구현계획서 — Task M100 #1218

**이슈**: [#1218](https://github.com/edwardkim/rhwp/issues/1218) HWP5 wrap=Square 표 단락 세로 측정 부족
**브랜치**: `local/task1218`
**작성일**: 2026-06-01

---

## 단계 분할 (5단계 — 고위험 layout 수정, 계측 우선)

### Stage 1 — 정확한 누락 지점 확정 (계측, 무수정)
- `typeset.rs` wrap-around 흡수 루프(~1214~1391) + 표 배치(~1526~1590) + `current_height` 누적 경로 정독.
- 임시 진단 로그(env `RHWP_TASK1218`)로 문26(pi=258) 처리 시 흡수 분기·누적 높이·표 높이 추적 → ①(pi=259) y 오프셋 ~22px 누락의 **정확한 코드 위치** 확정.
- 산출물: 누락 지점 1곳 특정 (Stage 2 대상 확정).

### Stage 2 — 단락 겹침 수정 (wrap=Square 표 옆/아래 텍스트 높이)
- 흡수된 wrap-around 단락의 "표 아래로 흐른 줄" 높이를 `current_height` 에 정합 누적 (표 높이 vs 텍스트 높이 max).
- 산출물: pi=259 y 정상(≈918px), 단0 diff≈0.

### Stage 3 — z-표 행 압축 수정 (셀 줄높이)
- 셀[2] `lh=825<font` 로 행 겹침 → 셀 내부 줄높이/valign 정합 점검·보정. (Stage 2 와 독립 가능; 별도 커밋.)
- 산출물: z=1.0/1.1 분리 렌더.

### Stage 4 — 회귀 검증 (광범위)
- `cargo test --release` 전체 통과.
- wrap=Square/인라인 표 포함 샘플 다수(09월/11월/2023 등) `export-svg` 시각 회귀 점검 — 레이아웃 유지 확인.
- 산출물: 회귀 0.

### Stage 5 — 시각 검증 + 보고
- 4쪽 문26 ①~⑤ 분리·z-표 정합, 한글 2022 PDF 4쪽 대조.
- `rustfmt`(변경 파일) → 최종 보고서.

---

## 원칙

- **계측 우선**: Stage 1 에서 정확한 위치 확정 전 수정 금지 (가설 기반 수정 회피).
- 최소 수정. 기존 wrap-around 정합(Task #855)·다른 wrap 모드 회귀 차단.
- 진단 로그는 Stage 5 전 제거.
