---
타스크: #460 HWP3 파서 렌더러 중립 재구현
문서: 최종 결과 보고서
브랜치: local/task460
작성일: 2026-04-29
상태: 완료
---

# 최종 결과 보고서 — Task #460

## 목적 및 배경

Task #417(HWP3 파서)과 Task #425(혼합 단락 겹침 수정)에서 추가된 렌더러 내 HWP3 전용 분기를
파서 처리로 이동하여, 렌더러가 HWP3를 인식하지 않아도 정상 출력되도록 한다.

## 수행 결과

### Stage 1: AutoNumber U+FFFC → 공백 (파서 처리)

**`src/parser/hwp3/mod.rs`**
- ch=18(AutoNumber) 처리 시 `'\u{FFFC}'` 대신 `' '`(공백) push
- 캡션 텍스트 "그림 " + `' '` = `"그림  "` → HWP5/HWPX `"  "` 패턴과 일치

**`src/renderer/layout/paragraph_layout.rs`**
- `'\u{fffc}'` 전용 탐색 분기(4줄) 제거
- HWP5/HWPX/HWP3 공통 `"  "` 패턴 단일 경로 유지

### Stage 2: 혼합 단락 LINE_SEG 높이 보정

**`src/parser/hwp3/mod.rs`** — `fixup_hwp3_mixed_para_line_segs()` 추가

Para-relative TopAndBottom non-TAC 그림이 있는 단락에서:
1. 그림 구역 `[fig_top, fig_bottom]` 탐색
2. LINE_SEG 누적 위치로 마지막 "그림 위쪽" seg(split_idx) 탐색
3. `seg[split_idx].line_height = fig_bottom - pos` (그림 하단까지 확장)
4. `seg[split_idx].text_height = 0`, `line_spacing = 0` (advance=lh+ls 보장)

**동작 검증 (pi=76, samples/hwp3-sample.hwp)**:
```
ls[5]: lh=36700 HU, th=0, ls=0  ← 보정 적용 ✓
렌더러 y+=489.3px → 줄 6: y_start+596px (그림 하단) ✓
document.rs 총 advance: 8000+36700+11200 = 55900 HU ✓
```

### 렌더러 변경 최소화 분석

| 파일 | 처리 | 근거 |
|------|------|------|
| `paragraph_layout.rs` U+FFFC 분기 | **제거** | 파서에서 처리 |
| `picture_footnote.rs` base_y 변경 | **유지** | HWP3 전용 아님 — vert_offset>0 Para-relative 그림 일반 버그 수정. 원복 시 다음 단락이 그림 구역 내 시작 |
| `pagination/engine.rs` 주석 | **유지** | 코드 변경 없음, 무해 |
| `src/bin/dump_pictures.rs` | **유지** | 렌더러 아님, 진단 도구 |

## 최종 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1068 passed, 0 failed ✓ |
| `cargo clippy -- -D warnings` | 경고 없음 ✓ |
| `export-svg hwp3-sample.hwp` | 22페이지 완료, LAYOUT_OVERFLOW 없음 ✓ |
| HWP5/HWPX 샘플 5종 | 모두 정상 완료, 회귀 없음 ✓ |

## 완료 기준 대조

- [x] 렌더러에 HWP3 전용 코드 전무 (`paragraph_layout.rs` U+FFFC 분기 제거)
- [x] `cargo test --lib`: 기존 통과 수(1068) 유지
- [x] `cargo clippy -- -D warnings`: clean
- [x] `hwp3-sample.hwp` SVG: 그림 겹침 없음, AutoNumber 정상
- [x] HWP5/HWPX 5종 byte 동일 (회귀 0)

## 수정 파일 요약

| 파일 | 변경 내용 |
|------|---------|
| `src/parser/hwp3/mod.rs` | AutoNumber ch=18 → `' '` + `fixup_hwp3_mixed_para_line_segs()` |
| `src/renderer/layout/paragraph_layout.rs` | U+FFFC 분기 제거 (4줄) |
