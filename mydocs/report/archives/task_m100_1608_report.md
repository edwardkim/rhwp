# 최종 결과보고서 — Task #1608

**제목**: `is_hwp3_origin` 오탐지 제거 — 네이티브 HWPX 부당 HWP3 tolerance 차단
**마일스톤**: M100 · **이슈**: edwardkim/rhwp#1608 · **브랜치**: `local/task1608`

## 1. 문제

`src/parser/hwpx/mod.rs` 의 `is_hwp3_origin = (head version == "1.4")` 판정이 HWPML **스키마
버전**을 HWP3→HWPX 변환 지표로 오인. 네이티브 한글2022 HWPX(version.xml: major=5 minor=1
"Hancom Office Hangul")도 head version 1.4 라, 거의 모든 모던 HWPX 가 HWP3-origin 으로
오탐지되어 부당한 "마지막 줄" tolerance(1600 HU ≈ 21px)를 받았다. 이 tolerance 는
`available_body_height()` 에 +21px 를 더해 경계 문서를 한글보다 1쪽 적게 렌더했다
(Task #1600 −1쪽 갭의 요인 A).

## 2. 수정 (방향 3 — tolerance 제거)

메타데이터로 진짜 변환본과 네이티브를 구별할 판별자가 없음을 조사·재현으로 확정,
파싱 시점의 HWP3 tolerance 부여를 제거.

- `src/parser/hwpx/mod.rs`: `is_hwp3_origin` 판정 + tolerance 부여 블록 제거
  (`hwpml_version` 무손실 보존은 유지).
- `src/parser/hwpx/header.rs`: `parse_hwpx_hwpml_version` 의 오버핏 docstring 정정.
- `tests/issue_1608_hwpx_native_no_hwp3_tolerance.rs`: 네이티브 HWPX tolerance==0 가드.

## 3. 결과

| 지표 | before | after |
|------|--------|-------|
| 통제셋 일치 (92건) | 60 (65.2%) | **66 (71.7%)** |
| −1쪽 | 29 | 21 |
| +초과 | 3 | 5 |
| **net (개선−회귀)** | — | **+6** |

전 회귀 게이트(hwpx baseline, visual_roundtrip, lib 1975 tests, clippy) 통과.
HWP3 변환본(hwp3-sample-hwpx) 16→16 무변동.

## 4. 한계 및 후속

- 회귀 2건(36395325·36382819): 부당 tolerance 가 우연히 정답을 맞추던 네이티브 문서.
  근저는 요인 B(footer 콘텐츠 누적 부족)로, net +6 우세에 따라 수용.
- −1쪽 갭의 요인 B(21건 미해소)는 본 이슈 범위 밖 — 별도 layout-fidelity 조사 필요.
  통제셋·게이트는 자산으로 보존(`tests/fixtures/render_page_controlset.tsv`,
  `tools/render_page_gate.py`).

## 5. 산출물

- 소스: `src/parser/hwpx/mod.rs`, `src/parser/hwpx/header.rs`
- 테스트: `tests/issue_1608_hwpx_native_no_hwp3_tolerance.rs`
- 측정: `output/poc/task1608_baseline.tsv`, `output/poc/task1608_after.tsv`
- 문서: `_impl`, `_stage1~3`, 본 보고서, `mydocs/tech/investigations/issue-1600/render_minus1_page_gap.md` 갱신
