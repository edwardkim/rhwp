# Task M100 #2193 Stage 1 작업보고서 — 기존 하네스 재실증과 계측 gap 감사

## 0. 판정 요약

- **Stage 판정**: 완료
- **기준**: `upstream/devel@53763944`
- **production 변경**: 없음
- **정확성 회귀**: native 3/3, browser HWP/HWPX와 raw 8/8 GREEN
- **기존 native probe**: 30 case 통과, 각 case 단일 관측이라 반복 기준선으로는 부족
- **기존 browser E2E**: stable/boundary operation과 boundary flush를 분리하지만 mutation,
  page-tree, render/2 rAF 구간별 통계는 부족
- **다음 단계**: 반복 native 기준선 하네스를 먼저 추가하고 p50/p95를 확정

## 1. 실행 환경

| 항목 | 값 |
|------|----|
| 기준 commit | `537639445332e85b76eb29c76e1dae4d8930369f` |
| 작업 브랜치 | `issue-2193-input-pagination-profile` |
| OS / architecture | macOS Darwin 25.5.0 / arm64 |
| Rust / Cargo | 1.93.1 / 1.93.1 |
| Node / npm | v24.15.0 / 11.12.1 |
| Chrome | 150.0.7871.128 |
| wasm-pack | 0.15.0 |
| Rust profile | `release-test` |
| Browser | headless, 1280×900, DPR 1, zoom 1 |
| Browser 반복 | 형식별 1회 smoke |

| fixture | 크기 | SHA-256 |
|---------|-----:|---------|
| `samples/issue1949_giant_cell_nested_tables_perf.hwp` | 303,616 bytes | `ef10261cd29325116028e4f4f3e6be1a72c675eb771bddfd8484e7fe5aa94b4e` |
| `samples/issue1949_giant_cell_nested_tables_perf.hwpx` | 266,523 bytes | `fc6e5f156de470dfbb14aab392389491720ee7fb1bf6f03fe9a018e93b420c65` |

WASM은 해당 commit에서 새로 빌드했다. `pkg/`와 `output/`은 재생성 가능한 ignored local
evidence이며 커밋 대상이 아니다.

## 2. 검증 결과

### 2.1 Native 정확성 핀

```text
cargo test --profile release-test --test issue_2214_page_local_repaint
```

3/3이 4.88초에 통과했다.

- cold representative tree/cursor exact
- warm deferred tree/cursor exact
- cell-flow transition baseline

전체 page count 115, `LINE_SEG.text_start = [0, 44, 84, 122]`, 다음 문단
`vpos = 17160` 계약이 유지됐다.

### 2.2 기존 native matrix probe

```text
cargo test --profile release-test --test issue_2214_cache_matrix_probe -- --ignored --nocapture
```

ignored 1/1이 80.13초에 통과했고 HWP/HWPX 합계 30 case를 생성했다. 출력은
`output/poc/task2214/stage2/native-matrix.json`이다.

| 형식 | case 수 | full flush 최소 | 중앙 관측값 | 최대 | page count |
|------|--------:|---------------:|------------:|-----:|-----------:|
| HWP | 15 | 1,124.317ms | 1,173.222ms | 1,186.453ms | 115 |
| HWPX | 15 | 1,177.405ms | 1,192.955ms | 1,206.691ms | 115 |

page-tree 생성은 대체로 약 20ms, 첫 cursor query는 약 32~34ms였다. 다만 위 최소·중앙·최대는
서로 다른 case의 단일 관측을 요약한 값이다. 같은 case 반복의 p50/p95가 아니므로 #2193의
성능 기준선으로 사용하지 않는다.

### 2.3 Studio 실제 입력 smoke

```text
npm run e2e:issue-2214 -- --runs=1
```

새 WASM과 로컬 Vite/Chrome에서 HWP/HWPX focused run 2/2, IME/iOS raw smoke 8/8이
GREEN이었다. 출력은 `output/poc/task2214/stage4/focused-summary.json`이다.

| 형식 | stable operation p95 | boundary operation | boundary flush | boundary flush 수 |
|------|---------------------:|-------------------:|---------------:|-----------------:|
| HWP | 38.7ms | 964.1ms | 909.2ms | 1 |
| HWPX | 38.1ms | 944.1ms | 892.1ms | 1 |

- stable 49회는 동기 flush 0회, 첫 flow boundary는 1회였다.
- boundary ordering은 mutation → flush → exact cursor 순서를 유지했다.
- page count는 두 형식 모두 115였다.
- IME/iOS raw stable은 flush 0, boundary는 flush 1이었다.
- 43→44 경계 화면은 10,074 pixel이 바뀌었고 이후 네 checkpoint는 0 pixel 변화로
  안정적이었다.

이 수치는 최신 환경의 smoke 관찰값이다. 형식별 1회이므로 절대 성능 판정이나 회귀 hard gate로
사용하지 않는다.

## 3. #2193 완료 조건 대비 gap 감사

| 완료 조건 | 현재 자산 | 판정 | 다음 조치 |
|-----------|-----------|------|-----------|
| HWP/HWPX 대표 문서 | native probe와 E2E가 두 형식 사용 | 충족 | 동일 fixture 유지 |
| cold/warm 통제 | native 30-case가 cold/pre-warm/every-edit 분리 | 부분 충족 | 반복 하네스에서 case를 축소·명시 |
| input/reflow와 pagination 분리 | native는 explicit flush, E2E는 operation/flush 기록 | 부분 충족 | mutation/reflow 자체 시간을 별도 기록 |
| 반복 횟수와 p50/p95 | E2E stable 49 operation만 통계 제공 | 미충족 | fresh document case별 10회 raw sample 추가 |
| page-tree/cursor 비용 | native 단일 elapsed와 exact 결과 존재 | 부분 충족 | pre/post flush 반복 통계 추가 |
| 실제 Studio input-to-display | operation, flush, cursor, Canvas 안정성 존재 | 부분 충족 | invalidation, render, 2 rAF phase timing 추가 |
| 정확성 고정 | native 3 tests와 browser structural/pixel gate | 충족 | 기존 non-ignored gate 계속 실행 |
| 전후 동일 프로토콜 | 아직 구현 전 기준선 없음 | 미충족 | Stage 2/3 JSON schema를 이후에도 고정 |
| 실행 환경·fixture 식별 | browser summary는 일부, native JSON은 없음 | 부분 충족 | Stage 2 산출물에 commit/tool/fixture metadata 추가 |

## 4. 원인 가설의 현재 강도

이번 재실증에서도 flow boundary operation의 약 94~95%가 full pagination flush였다. 따라서
115쪽 전역 pagination이 실제 boundary 지연의 지배 항이라는 가설은 강하다. 반면 stable
operation 약 38ms에는 cursor/page-tree와 browser handler 비용이 섞여 있어 mutation/reflow
자체 비용으로 해석할 수 없다.

즉 production 최적화를 시작할 근거는 아직 부족하다. 우선 native에서 같은 fresh-document
case를 반복해 mutation, pre-flush query, flush, post-flush query의 분포를 고정하고, 그 다음
Studio에서 end-to-display 누락 구간만 계측하는 순서가 적절하다.

## 5. Stage 2 진입 결정

다음 단계는 `tests/issue_2193_input_pagination_perf.rs` ignored diagnostic을 추가하는 것이다.

- HWP/HWPX와 stable/boundary를 분리한다.
- cold/warm case를 명시하고 기본 10회 측정한다.
- raw samples와 p50/p95/max를 `output/poc/task2193/stage2/`에 기록한다.
- timing assertion은 두지 않고 구조 정확성 assertion만 둔다.
- 기존 `issue_2214_page_local_repaint`를 함께 실행한다.
- Stage 2 결과를 확인하기 전 production paginator나 공개 WASM API는 변경하지 않는다.
