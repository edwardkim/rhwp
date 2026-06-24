# 최종 결과보고서 — Task #1499: HWPX 라운드트립 시각 정합성 게이트 (render-diff)

- **이슈**: edwardkim/rhwp#1499 (M100) · **브랜치**: `local/task1499`
- **작성일**: 2026-06-24

## 1. 목표 달성 요약

라운드트립(parse→serialize→reparse)이 유발하는 **렌더 기하 변화**(페이지 수, 렌더 노드
삽입/삭제, bbox 변위)를 정량화하는 시각 회귀 게이트를 신설했다. 기존 `hwpx-roundtrip`
baseline 은 IR 뼈대(구조)만 검증하여 IrDiff 0 인데도 렌더가 달라지는 회귀를 놓쳤다 —
본 게이트가 그 공백을 메운다. 폰트 래스터화에 의존하지 않는 **결정론적 기하 비교**다.

## 2. 산출물

| 항목 | 파일 |
|------|------|
| 비교 코어 + CLI | `src/diagnostics/render_geom_diff.rs` (신설) |
| 모듈 등록 | `src/diagnostics/mod.rs` |
| CLI dispatch/help | `src/main.rs` |
| 회귀 게이트 | `tests/visual_roundtrip_baseline.rs` (신설) |
| 매뉴얼 | `mydocs/manual/render_diff_command.md` (신설), `CLAUDE.md` |
| 측정 데이터 | `output/poc/task1499/geom_inventory.tsv` |

## 3. 비교 알고리즘

페이지별 RenderNode 트리를 전위순회 평탄화 → `(타입 태그, bbox)` 시퀀스 → **타입 LCS
매칭**(DP). 매칭쌍 변위 = `max(|Δx|,|Δy|,|Δw|,|Δh|)`. 삽입/삭제는 구조 불일치로 분류.
텍스트 내용·스타일은 비교 대상이 아니다(그것은 `ir-diff` 담당).

판정: `PASS`(페이지 일치 ∧ 삽입삭제 0 ∧ 변위 ≤ 임계) / `STRUCT_MISMATCH` / `DISP_OVER`.

## 4. CLI 3모드

- `render-diff <a.hwpx>` — 자기 라운드트립.
- `render-diff <a.hwpx> <b.hwpx>` — 두 파일 비교.
- `render-diff --batch <폴더> [-o]` — 전수 → `geom_inventory.tsv`. 하드 실패 시 종료코드 1.
- `--threshold <px>` (기본 0.5).

## 5. 측정 결과 (`samples/hwpx` 57건)

| 등급 | 건수 |
|------|------|
| PASS (baseline) | 36 |
| XFAIL (라운드트립 렌더 회귀) | 20 |
| EXCLUDED (HWPX 아님) | 1 |

- **임계 확정**: PASS 36건의 매칭 노드 변위는 **전부 0.0px**. 부동소수 여유를 두어
  `DEFAULT_THRESHOLD_PX = 0.5` 채택 (이슈 제시값과 일치).
- **게이트 유효성 입증**: `el-school-001.hwpx` 는 `hwpx-roundtrip` IrDiff=0(구조 보존)
  인데도 본 게이트가 **임베디드 Image 노드 1→0 누락**을 검출. 기존 게이트의 사각을 정확히 포착.

### XFAIL 분류 (모두 IrDiff 0, 후속 수정 대상)

- **단순 문서 렌더 회귀 (우선 수정 후보)**: `el-school-001`(Image 누락), `143E433F503322BD33`,
  `expense_report`, `hy-002`, `2026_oss_rst`, `exam_social-p1`, `footnote-01`, `shape-001`(변위 6.8px),
  `hwpx-h-01`(변위 514px).
- **복합 실문서(ORACLE_UNFIT) known 변화**: `exam_social`, `exam-kor-2p/3p/4p`, `exam_kor`,
  `aift`, `k-water-rfp`, `[2027] 온새미로 1 본교재`, 보도자료 ff×3.

## 6. 검증

- 코어 단위테스트 5건 (`cargo test --lib render_geom_diff`).
- 게이트 3건 (`cargo test --test visual_roundtrip_baseline`).
- 전체 `cargo test` 회귀 통과 (메모리 룰 `feedback_full_cargo_test_before_pr` 준수).
- 두 파일 동일 입력 비교 → PASS·변위 0 (비교기 정합성 확인).

## 7. 범위 밖 (후속)

- XFAIL 20건의 개별 렌더 회귀 수정 (특히 임베디드 이미지 누락) — 별도 이슈.
- 픽셀/SSIM 보조 비교 (P1) — 폰트·플랫폼 잡음으로 본 게이트와 분리. 작업트리
  `compare_svg_pdf.sh` 자산은 이 P1 라인으로 보존.
- 한컴 정답지 시각 충실도 — 본 게이트(내부 회귀 방지)와 별개 축.

## 8. 결론

이슈 범위(비교 코어 / CLI 3모드 / baseline 테스트)를 모두 충족했다. 36 PASS 를 회귀
방지선으로 고정하고, 20 XFAIL 로 기존 게이트가 못 잡던 시각 회귀를 가시화했다.
