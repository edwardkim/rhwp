# #3128 Stage 3 — focused 회귀와 시각 검증

- **Issue**: #3128
- **기록일**: 2026-08-18 KST
- **성격**: 소급 완료 기록
- **전체 PR gate**: 미실행, 별도 승인 필요

> 이 Stage는 로컬 focused 검증 결과만 보존한다. 저장소 절차가 별도 승인을 요구하는 전체
> release/WASM/PR CI 성격 검증을 통과했다고 해석하지 않는다.

## 1. 전용 수용 테스트

`tests/cases/issue_3128_terminal_nested_table_geometry.rs`에 다음을 고정했다. 최신 devel의 자동
sharding 정책에 따라 review worktree에서 generated suite로 배정해 실행한다.

- 전체 82쪽
- p34 continuation top·bottom의 PDF 좌표 허용 오차
- 후속 직접편익 표 y 좌표
- 첫 들여쓰기 continuation 줄의 `연동시스템 등` wrap

최종 재실행 결과: **2 passed, 0 failed**.

## 2. focused 회귀 결과

다음 suite를 통과했다.

- `issue_2308_render_normalized_derived_state` 5건
- `issue_1891` 4건
- `issue_2430_cell_rewrap_threshold` 2건
- `issue_3820_rowbreak_rowspan_band` 4건
- `issue_2007_nested_cell_pagination` 15건
- #3637 nested-table start/vpos 회귀 각 1건
- `overflow_cell_baseline`: 683 samples, 3 skipped, 실패 0

특히 초기 broad tracking 시 156쪽으로 바뀌었던 80168 계열은 scope를 좁힌 뒤 다시 157쪽을 유지했다.

## 3. 시각 비교

최종 CLI를 다시 빌드하고 다음과 같은 단일 페이지 sweep을 수행했다.

```bash
scripts/visual_sweep.py \
  --key issue3128-final \
  --hwp samples/76076_regulatory_analysis.hwp \
  --pdf samples/issue1891/76076_regulatory_analysis-2024.pdf \
  --pages 34 --dpi 96 --rhwp-bin target/debug/rhwp
```

| 지표 | 수정 전 | 수정 후 |
| --- | ---: | ---: |
| pixel match | 87.69871% | 90.03412% |
| ink match | 10.22922% | 11.11311% |

전역 font glyph·paint 차이가 남아 ink match 자체는 낮다. #3128 판정은 전체 raster 점수만으로 하지 않고,
PDF border 좌표, render-tree geometry, line wrap과 side-by-side 이미지를 함께 사용했다. 이 issue의 핵심인
continuation 하단과 후속 표 anchor는 기준 오차 안으로 들어왔다.

## 4. 정적 검사

- `cargo fmt --all -- --check`: 통과
- `git diff --check`: 통과
- debug CLI build: 통과

## 5. 남은 gate

- 최신 `upstream/devel` 반영 후 충돌·회귀 재확인
- renderer 변경 범위의 전체 release test
- Clippy `-D warnings`
- WASM build/test
- 최종보고서 확정과 PR 준비

위 검증은 수행·구현 계획 승인 뒤 별도 실행 승인을 받아 진행한다.
