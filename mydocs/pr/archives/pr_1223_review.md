# PR #1223 검토 — 수식 포함 줄 본문 한글 압축·겹침 해소 (거짓 오버플로우)

- **작성일**: 2026-06-01
- **PR**: #1223 (OPEN)
- **컨트리뷰터**: @planet6897 (핵심 컨트리뷰터 — #1202/#1203/#1208/#1220 머지)
- **연결 이슈**: #1219 (OPEN)
- **base/head**: `devel` ← `feature/issue-1219-equation-line-hangul` (cross-repo)
- **규모**: 7 파일, +576/−15 (소스 `paragraph_layout.rs` + 회귀 테스트 + 작업문서 5)
- **mergeable**: MERGEABLE / BEHIND
- **CI**: **전부 PASS** (Build&Test/Canvas visual diff/Analyze×3/CodeQL)
- **라벨**: enhancement / 마일스톤 v1.0.0

## 1. 문제 (측정/렌더 불일치 — 코드 확인)

인라인 수식(TAC) 포함 줄의 본문 한글이 압축·겹침(`3-09월_교육_통합_2023.hwp` 6쪽 문26,
advance 8.96px vs PDF ≈12px). align=Left·자간 0%·장평 100% 로 압축 의도 없음. `project_equation_always_tac`
(수식=항상 TAC, paragraph_layout 인라인 배치 핵심 경로) 직결.

근본 원인 둘:
1. **줄 경계 오포함**: `est_x` 측정 루프가 전역 `tac_offsets_px` 를 run 경계로 재필터, `total_tac_width_in_line`
   은 `pos <= line_end`(포함) 필터 → **줄 끝 위치(=다음 줄 선두) 수식을 현재 줄 폭에 오포함**(문26
   라인0 에 다음 줄 `a₁=b₁=1` 55px). 렌더 경로(`tac_offsets_for_line` = `char_pos_in_line` 엄격 미만)와 불일치.
2. **선두 미주 이중계상**: `endnote_marker_x_advance`(선두 마커 풀사이즈, available_width 차감)와
   `footnote_positions` 위첨자 측정 양쪽 계상. 렌더는 인라인 위첨자 안 그림.

→ 거짓 `total_text_width > available_width` → 비정렬(Left) 줄에 음수 자간 압축.

## 2. 수정 내용 검토 (paragraph_layout.rs — 측정을 렌더 규칙에 통일)

- TAC 소스를 `line_tac_offsets`(= `tac_offsets_for_line`, 렌더와 동일한 `pos < 다음 줄 시작` 엄격 미만)로 통일.
  `est_x` 루프와 `total_tac_width_in_line` 둘 다 적용(`pos <= line_end` 제거).
- `footnote_positions` 측정에서 `start_line==0` 의 선두 미주(Endnote) 제외(이중계상 차단).
- **측정 코드만 변경** — 렌더 경로/모델 무변경. 측정을 이미 정확한 렌더 규칙에 맞추는 방향이라 일관성↑.

## 3. 위험 평가

- **중간(측정 경로)이나 가드 견고.** 인라인 TAC 가 정상이던 다른 줄/문서에서 측정이 바뀌면 회귀
  가능 → golden SVG 스냅샷 8건이 byte 단위 가드. 측정을 렌더 규칙에 통일하므로 정상 줄은 불변.

## 4. 검증 결과 (로컬 머지 시뮬 `pr1223-verify`)

| 단계 | 결과 |
|------|------|
| merge | ✅ CLEAN (paragraph_layout.rs, 충돌 0) |
| fmt / build | ✅ |
| 전체 테스트 `cargo test --tests` | ✅ **1925 passed, 0 failed** |
| **golden SVG 스냅샷** | ✅ **8 passed** (인라인 TAC/각주/목차/우측탭 회귀 0 — 핵심 가드) |
| issue_1219 회귀 테스트 | ✅ 1 passed (문26 한글 advance ≥ 11.0px) |
| **한글 advance 정량 (정답지 대조)** | ✅ 압축 줄 **보정 전 8.96px → 후 11.93px**(≈PDF 12px) 12글자. 정상 줄 12.0px 보정 전후 동일(40→41, 무관 줄 불변) |

산출물: `output/poc/pr1223/{before,after}/3-09월_교육_통합_2023_006.svg`.

## 5. 판단 — 머지 권고 (시각 판정 게이트)

- 진단 정확(측정/렌더 불일치 둘 다 코드 확인), 측정을 렌더 규칙에 통일(일관성), golden 8건 무회귀,
  한글 advance 정량 정합, CI green.
- 측정 경로 + 시각 영향 → **작업지시자 시각 판정** 게이트(6쪽 문26 본문 한글 겹침 해소 ↔ PDF).
  rhwp-studio 도 가능 시 함께.
- 승인 + 시각 판정 통과 시 메인테이너 로컬 `--no-ff` 머지 + push + WASM 빌드(렌더 측정 경로 변경).

## 6. 비고

- 글리프가 PDF보다 크게 보이는 현상(Noto Sans CJK ↔ 한컴 돋움 폰트 대체, ~30% 큼)은 본 건과 별개 —
  별도 이슈(PR 본문 명시). 본 PR 은 advance(자간) 압축 해소이지 글리프 크기 문제 아님.
- #1220(같은 샘플 4쪽 wrap=Square)과 다른 줄/원인 — 독립.
