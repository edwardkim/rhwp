# 1차 리팩토링 Baseline Manifest (Phase 0 freeze) — #1904

- 작성일: 2026-07-04 / **기준 commit: `00014ecf` (devel)**
- 목적: 리팩토링 전 행동을 재현 가능하게 고정 — 이후 모든 무변동 게이트의 비교 기준
  (jangster77 보완 2 반영: 샘플·명령·commit·환경·임계값·제외 목록 명기)

## 1. 열린 PR inventory (freeze 진입 조건 — 보완 1)

**2026-07-04 기준 열린 PR: 0건** — #1875/#1894 등 렌더/layout 계열이 모두 처리된 상태에서
freeze 진입. 신규 렌더링 PR 은 1차 진행 중 hold 라벨 운용(계획 §4).

## 2. 측정 환경 (고정)

| 항목 | 값 |
|---|---|
| OS | Linux 6.6.114.1 WSL2 |
| rustc / cargo | 1.93.1 |
| rasterizer | rsvg-convert 2.58.0 / pdftoppm 24.02.0 (poppler) |
| Python | 3.12.3 (OVR/하니스) |
| 폰트 | 저장소 `ttfs/opensource` + `web/fonts` (OFL 계열) — 시스템 한컴 폰트 없음 |
| 빌드 | `cargo build --release` (테스트는 `--profile release-test`) |

## 3. 행동 고정 자산 + 생성/검증 명령

| 자산 | 위치 | 검증 명령 | 허용 임계값 |
|---|---|---|---|
| golden SVG (7건) | `tests/golden_svg/` | `cargo test --profile release-test --test svg_snapshot` | 내용 diff 0 |
| 페이지 오라클 | `tests/fixtures/render_page_controlset.tsv`(93) + `render_page_oracle_1658.tsv`(453) | `verify_pi_page_vs_hangul.py` 계열 | 일치 유지(악화 0) |
| roundtrip baseline | `hwpx_roundtrip_baseline`(4)·`hwp5_roundtrip_baseline`(3)·`visual_roundtrip_baseline`(3) | `cargo test --profile release-test --test <각>` | 하드실패 0 |
| **OVR baseline (신규, 5샘플)** | `mydocs/metrics/2026-07-04/ovr/*.baseline.json` | `python tools/object_visual_regression.py <샘플> -o out --no-hwp --baseline <json>` | 개체 이동/리사이즈 0 (±2px tol) |

> 갱신 이력: `rowbreak-problem-pages`는 2026-07-07 재생성 (기준 `a05e6f1b`) — #1936 머지로
> 발생한 p13/p14 재배분 3건(obj4/5/6)이 **작업지시자 시각 판정 통과**(한컴 정답지 기준)로
> 정합 상태로 확정되어 baseline을 현행화. 판정 전까지는 "추가 변동 0" 기준으로 운용했음.
> 갱신 이력 2: `rowbreak-problem-pages`는 2026-07-12 재생성 (기준 `6b2d23cc`) — #2211
> (중첩/TAC 표 행 성장 pad 제외)로 p7 tac 노트박스 h 162.8→159.0px (한컴 154px 방향의
> 의도된 정합 개선)이 **작업지시자 시각 판정 통과**로 확정되어 현행화.
| 대시보드 영점 | `mydocs/metrics/2026-07-04/` | `./scripts/metrics.sh --snapshot` | (개선 측정 영점) |
| 전체 테스트 | — | `cargo test --profile release-test --tests` | FAILED 0 (영점 2,820) |

### OVR baseline 대표 샘플 (선정 사유)

| 샘플 | 개체 | 사유 |
|---|---|---|
| `rowbreak-problem-pages.hwp` | 표 8 | RowBreak 분할 대표 (18쪽) |
| `pr-1674.hwp` | 표 5 | co-anchored 다중 표 + page-over 이력 (35쪽) |
| `exam_science.hwp` | 표 13 | 시험지 다개체 밀집 |
| `issue1853_caption_precedes_body_split.hwpx` | 분할 표 | #1878 예산 교정 검증 샘플 (52쪽) |
| `issue1835_tac_stale_height.hwp` | TAC 표 | #1881 높이 확장 검증 샘플 |

## 4. 제외 목록 (복잡도 지표 모집단 정의 — 보완 3 잠정, v2 산식에서 확정)

- 자동 생성 데이터: `font_metrics_data.rs`(45,951) / `johab_map.rs` / `pua_oldhangul.rs`
- 인라인/통합 테스트 코드 (`#[cfg(test)]`, `tests/`, `wasm_api/tests.rs`)
- **잠정 모집단**: "1,200줄 초과"는 **.rs runtime 로직 한정 70개**를 1차 추적 지표로,
  대시보드 카드(전체 80개 — studio .ts 포함)는 참고 병기. CC 예외(CLI dispatch 등)는
  v2 산식 확정 시 정의.

## 5. 게이트 실행 요약 (1차 리팩토링 모든 PR 공통)

```bash
cargo fmt --check && cargo clippy --profile release-test --all-targets
cargo test --profile release-test --tests                      # FAILED 0
for b in mydocs/metrics/2026-07-04/ovr/*.baseline.json; do     # OVR 무변동
  s=$(basename "$b" .baseline.json)
  python tools/object_visual_regression.py samples/$s.* -o /tmp/ovr-gate/$s --no-hwp --baseline "$b"
done
```
