# #5511 Stage 2 기능군 배치 Q2 — render output adapter 경계

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 구현 시작 기준: `afa162890`
- 수행일: 2026-08-19
- 상태: 완료 — Q3 진입 승인 대기

## 1. 결과

메인테이너가 승인한 A안에 따라 CC 25를 넘던 네 handler를 먼저 제자리에서 분해하고,
검증된 일곱 output handler를 세 책임 모듈로 이동했다.

| 모듈 | 소유 명령 | 최종 줄 수 |
|---|---|---:|
| `cli/outputs/vector.rs` | `export-svg`, `export-render-tree`, `export-structure` | 843 |
| `cli/outputs/raster.rs` | `export-png`, `export-png-gpu`, `gpu-info` | 878 |
| `cli/outputs/pdf.rs` | `export-pdf` | 484 |

`src/main.rs`는 Q2 시작의 38,405줄에서 36,453줄로 1,952줄 줄었다. 새 모듈은 모두
1,200줄 상한 이하고, root에는 해당 명령의 최상위 dispatch만 남았다. HML sibling resource를
암묵적으로 열지 않는 공통 판정은 `cli/outputs/mod.rs`에 한 번만 두어 helper 복제를 피했다.

## 2. A안 책임 분해

이동 전에 다음 책임을 `src/main.rs` 안에서 먼저 분리했다.

- SVG: `SvgExportArgs`, option parser, document 설정 helper
- PNG: `PngExportArgs`와 feature-gated option parser
- GPU PNG: `GpuPngExportArgs`, option parser, GPU context 준비, 페이지 선택 helper
- PDF: `PdfExportArgs`와 backend option 검증을 포함한 parser

Clippy `cognitive_complexity`를 default·`native-skia`·`gpu` 세 축에서 다시 실행했다.

| handler | 분해 전 CC | 분해·이동 후 판정 |
|---|---:|---|
| `export_svg` | 38 | CC 경고 없음, 25 이하 |
| `export_pdf` | 32 | CC 경고 없음, 25 이하 |
| `export_png_gpu` | 35 | CC 경고 없음, 25 이하 |
| `export_png` | 26 | CC 경고 없음, 25 이하 |

renderer·parser·serializer 알고리즘은 바꾸지 않았다. option parser와 실행 준비의 제어 흐름만
분리했고, 파일명·출력 바이트·stdout/stderr·종료 코드는 기존 본문과 characterization 계약을
그대로 유지했다.

## 3. 커밋 계보

| 커밋 | 역할 |
|---|---|
| `46579796b` | GPU feature 미활성 stub의 exit 2, 빈 stdout, 정확한 stderr를 characterization |
| `f325418d7` | 네 고복잡도 output handler를 제자리에서 CC 25 이하로 분해 |
| `836d4f233` | 일곱 handler와 전용 helper를 vector·raster·PDF 모듈로 이동 |

분해 전후와 이동 전후에 같은 focused 범위를 실행했다. 첫 분해 HEAD와 최종 이동 HEAD에서
각각 807/807이 통과해 물리 이동 전후 계약을 같은 모집단으로 비교했다.

## 4. 최종 검증

| 검증 | 결과 |
|---|---|
| 이동 전 focused nextest | 807/807 통과, 22.699초 |
| 이동 후 focused nextest | 807/807 통과, 24.335초 |
| release-test 전체 nextest | 7,775/7,775 통과, 3 slow, 38 skipped, 162.834초 |
| Native Skia library `skia` filter | 58/58 통과 |
| missing-picture PNG 대표 회귀 | 2/2 통과 |
| direct PDF 대표 회귀 | 4/4 통과 |
| default·native-skia·gpu compile | 모두 통과 |
| default·native-skia·gpu CC 25 상한 | 대상 함수 경고 없음 |
| `cargo fmt --all -- --check`·`git diff --check` | 통과 |
| `cargo check --all-targets` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --doc` | 8/8 통과, 3 ignored |
| integration manifest 정책 자체 계약 | 16/16 통과 |
| 최신 base manifest check | 754 sources / 3,726 static test attrs / 43 integration targets, 통과 |
| unit-tier 정책 자체 계약과 base check | 12/12, 4,225 tests / 298 modules, 통과 |
| CI impact Node·workflow 계약 | 62/62, 30/30 통과 |
| Markdown link check | 기존 capability 등록부 무결성 오류 16건, Q2 신규 오류 없음 |

검증 준비가 `Cargo.toml`에 만든 두 integration target과 Cargo가 재정렬한 lockfile package 순서는
추적 변경에서 제거했다. 파생 target을 정리한 제출 상태에서 `--prepare` 없는 manifest check가
drift를 보고하는 것은 원본-only 정책에 따른 예상 동작이다.

이번 변경은 CLI adapter의 option parsing과 물리 위치만 바꾸며 renderer/layout/WASM 경계를
수정하지 않는다. SVG·PDF의 기존 바이트·JSON 계약과 native-skia PNG·direct PDF 계약이 직접
통과했으므로 시각 sweep과 WASM 빌드의 발생 조건에는 해당하지 않는다고 판정했다.

## 5. 원격 병합 위험

최종 재조회 시 `origin/devel`과 `upstream/devel`은 `625758ee6`로 같고, 구현 HEAD
`836d4f233`은 5커밋 뒤·64커밋 앞이다. Q2 검증 중 새로 들어온 커밋은 adapter-diff workflow와
문서만 변경했으며 Q2 제품·test 경로와 겹치지 않았다. 최신 base와 구현 HEAD의 merge-tree는
충돌 없이 생성됐다.

열린 PR은 16개이고 `src/main.rs`, `src/cli/outputs/`, `tests/cli_exit_codes.rs`를 포함한 Q2 대상
경로와 교집합은 0개다. 최신 base를 대상으로 manifest와 unit-tier 정책도 다시 통과했다. 이 증거는
시점 판정이므로 push 전에는 exact base SHA, PR head와 merge-tree를 다시 확인한다. remote push는
수행하지 않았다.

## 6. 다음 승인 단위

다음 기능군은 Q3 `text·tables·LLM·CSV·Markdown` data exchange adapter다. stdout·파일·BOM·
JSON 봉투의 동등성 inventory와 열린 PR 경로를 다시 확인하고, 미보호 계약이 있으면 독립
characterization 커밋을 먼저 만든다. Q3는 메인테이너의 배치 종료 승인과 진입 승인 전 시작하지
않는다.
