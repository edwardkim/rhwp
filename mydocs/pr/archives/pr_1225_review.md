# PR #1225 검토 — HWP5 수식-only 셀 z-표 행 압축 수정 (tac 순서매핑)

- **작성일**: 2026-06-01
- **PR**: #1225 (OPEN)
- **컨트리뷰터**: @planet6897 (핵심 컨트리뷰터 — #1202/#1203/#1208/#1220/#1223 머지)
- **연결 이슈**: #1221 (OPEN) — #1220 의 "범위 외 z-표 행 압축" 후속
- **base/head**: `devel` ← `fix/1221-cell-eq-row-mapping` (cross-repo). head parent=`4f5a8e22`(#1223 이전)
- **규모**: 7 파일, +182/−4 (소스 `paragraph_layout.rs` +23 단일 블록 + 작업문서 6)
- **mergeable**: MERGEABLE / BEHIND
- **CI**: **전부 PASS** (Build&Test/Canvas visual diff/Analyze×3/CodeQL)
- **라벨**: enhancement / 마일스톤 v1.0.0

## 1. 문제 (계측 확정)

`3-09월_교육_통합_2023.hwp` 4쪽 문26 z-표(표준정규분포표): z-열 "1.0"/"1.1" 이 겹쳐 "1.01." 로
표시되고 첫 행 z 가 빔. z값은 인라인 수식(TAC)이고, 수식-only 셀 문단은 텍스트가 없어 모든
LINE_SEG.text_start=0 → **모든 composed 줄 char_start=0(degenerate)**. tac→줄 char-범위 매핑이:
- line0 `[0,0)` 빈 범위 → 수식 0개(행1 빔)
- line1 `[0,MAX)` → 같은 문단 모든 수식 흡수 → 두 수식 같은 y 가로 인접("1.01." 겹침).

## 2. 수정 내용 검토 (paragraph_layout.rs 빈-runs tac 블록)

`index_based_tac` 가드(4조건 모두 충족 시에만 순서 1:1 매핑):
1. `lines.len() > 1` 2. `lines.len() == tac_offsets_px.len()` 3. `all(|l| l.runs.is_empty())`
4. `windows(2).any(|w| w[1].char_start <= w[0].char_start)` (char_start degenerate)

`tac_on_line(k,pos)` = `index_based_tac ? k==line_idx : (pos in [start,end))`. **그 외엔 기존
char-범위 그대로 → 일반 텍스트+수식 문단 불변**. 매우 보수적 가드(`feedback_hancom_compat_specific_over_general`).

## 3. 위험 평가

- **중간(공통 paragraph_layout)이나 가드 견고.** 4조건 동시 충족(수식-only 셀 z-표 같은 degenerate)
  에만 발동, 그 외 char-범위 유지. golden SVG + 전체 페이지 diff 로 검증.

## 4. 검증 결과 (로컬 머지 시뮬 `pr1225-verify`)

| 단계 | 결과 |
|------|------|
| merge | ✅ CLEAN (#1223 과 같은 파일이나 다른 블록 — 충돌 0) |
| fmt / build | ✅ |
| 전체 테스트 `cargo test --tests` | ✅ **1925 passed, 0 failed** |
| golden SVG 스냅샷 | ✅ 8 passed (표/수식 회귀 0) |
| **전체 23쪽 SVG diff (devel vs devel+#1225)** | ✅ **4쪽 1페이지만 변화(4줄), 나머지 22쪽 무영향** |
| **z-표 행 분리 (정량)** | ✅ 셀 cell-clip-178 수식 2개: 보정 전 둘 다 y=925.3(가로 인접 겹침) → 후 y=913.3 / 925.3(세로 분리) + x 305.2 정렬 |

산출물: `/tmp/devel_all` vs `/tmp/merged_all` 4쪽 diff.

> 검증 메모: 초기 before/after 비교에서 stash 꼬임으로 byte 동일 오판 → devel 단독 vs 머지본
> 전체 페이지 diff 로 재측정하여 4쪽 z-표 정확히 1곳 변화·무회귀 확정.

## 5. 판단 — 머지 권고 (시각 판정 게이트)

- 진단 정확(계측+코드), 4조건 보수 가드, 전체 페이지 diff 로 4쪽만 변화·22쪽 무영향, golden 8건·
  1925 passed, z-표 행 분리 정량 확인, CI green.
- 셀 렌더 + 시각 영향 → **작업지시자 시각 판정** 게이트(4쪽 z-표 1.0/1.1/1.2/1.3 각 행 분리 ↔
  한글 2022 PDF). rhwp-studio 가능 시 함께.
- 승인 + 시각 판정 통과 시 메인테이너 로컬 `--no-ff` 머지 + push + WASM 빌드.

## 6. 비고

- #1220(4쪽 wrap=Square 답안 겹침)·#1223(수식 줄 한글 압축)과 같은 샘플의 다른 결함 — 독립.
- 셀 valign centering·line-height 클램프·table_layout/shape 경로는 z값 미통과(PR 진단 배제 확인).
