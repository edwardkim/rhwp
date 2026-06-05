# PR #1268 처리 보고서 - legacy HWP OLE 차트 Contents 렌더링

- PR: https://github.com/edwardkim/rhwp/pull/1268
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1251
- 작성일: 2026-06-03
- 작성자: @postmelee
- 처리 브랜치: `local/pr1268-integration`
- 통합 방식: 최신 `devel` 기준 cherry-pick + 문서 archive 정책 반영 + `charming` feature 제외 보정

## 1. 반영 내용

PR #1268의 핵심 구현을 수용했다.

- legacy HWP OLE `/Contents` 스트림 probe/parser 추가
- `VtDataGrid`, `VtChartTitle` 기반 최소 차트 데이터 추출
- `OleChart` IR 및 JSON/base64 helper 추가
- Rust SVG chart renderer 추가
- OLE render priority에 `/Contents` chart parser + `RawSvg` 경로 추가
- `samples/143E433F503322BD33.hwp` fixture 기반 회귀 테스트 추가

PR에 포함된 `mydocs/plans`, `mydocs/report`, `mydocs/working` 루트 문서는 현행 archive 정책에 맞춰 각 `archives/` 아래로 이동했다.

## 2. 통합 중 보정

PR 원본에는 `charming-renderer` feature와 optional native `charming` SSR adapter가 포함되어 있었다.

통합 검증에서 다음 명령이 실패했다.

```text
cargo test --features charming-renderer --test issue_1251_ole_chart_contents -- --nocapture
```

실패 원인:

```text
rust-lld: error: relocation R_X86_64_TPOFF32 against v8::internal::g_current_isolate_
cannot be used with -shared
```

`charming`의 `ssr` feature가 끌어오는 `deno_core/v8` 정적 라이브러리가 현재 `crate-type = ["cdylib", "rlib"]` 링크와 충돌한다. 따라서 통합본에서는 `charming-renderer` feature, `charming` dependency, `src/ole_chart/charming_renderer.rs`, 관련 feature tests를 제외했다.

기본 렌더링 경로는 Rust SVG `RawSvg`로 유지한다. `charming`/ECharts 경로는 별도 adapter 이슈에서 다시 검토하는 것이 안전하다.

## 3. 충돌 해결

GitHub가 표시한 충돌은 `mydocs/orders/20260603.md`에서 발생했다.

해결 방식:

- 현재 `devel`의 2026-06-03 작업 기록 보존
- #1205 완료 행 유지
- #1251 PR 초안 완료 행 추가
- PR 문서 위치는 archive 정책에 맞춰 정리

## 4. 검증 결과

| 항목 | 결과 |
|---|---|
| `cargo fmt --all --check` | 통과 |
| `git diff --check devel..HEAD` | 통과 |
| `cargo test --test issue_1251_ole_chart_contents -- --nocapture` | 통과 |
| `cargo test --lib ole_chart -- --nocapture` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |
| `cargo test --tests --quiet` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |

`charming-renderer` feature 검증은 실패했으며, 위 보정으로 최종 통합본에서는 해당 feature를 제거했다.

## 5. 시각 판정 준비물

메인테이너 시각 판정용 SVG를 생성했다.

| file | 비고 |
|---|---|
| `output/poc/pr1268-ole-chart/143E433F503322BD33.svg` | `samples/143E433F503322BD33.hwp`, debug overlay |

생성 명령:

```text
cargo run --bin rhwp -- export-svg samples/143E433F503322BD33.hwp -o output/poc/pr1268-ole-chart --debug-overlay
```

빠른 확인:

- `hwp-ole-chart hwp-ole-chart-rust-svg` 존재
- `연금 재정 전망`, `적립금`, `수입`, `지출` 존재
- `OLE 개체 (BinData #2)` 없음
- `OLE 차트 미지원` 없음

## 6. 남은 절차

1. 메인테이너 SVG 시각 판정
2. 필요 시 WASM 빌드 후 rhwp-studio 시각 판정
3. 판정 통과 시 `local/pr1268-integration`을 `devel`에 병합
4. `devel`에서 최종 테스트 확인
5. 원격 `devel` push
6. GitHub CI 확인
7. PR #1268 및 issue #1251 종료 처리

## 7. 판정

자동 검증 및 메인테이너 시각 판정 기준으로 통합 가능하다.

`charming` adapter는 현재 crate-type과 충돌하므로 이번 PR 범위에서는 제외했다. 핵심 목표인 legacy OLE `/Contents` 차트의 placeholder 탈출과 Rust SVG 렌더링 경로는 동작한다.

- 2026-06-03 메인테이너 시각 판정 통과: `output/poc/pr1268-ole-chart/143E433F503322BD33.svg`
