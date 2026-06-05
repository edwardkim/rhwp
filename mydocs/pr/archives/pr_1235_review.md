# PR #1235 검토 — 수식 큰 연산자(Σ/∏/∫) 피연산자 간격 추가

- **작성일**: 2026-06-02
- **PR**: #1235 (OPEN)
- **제목**: `수식 큰 연산자(Σ/∏/∫) 피연산자 간격 추가 (closes #1233)`
- **컨트리뷰터**: @planet6897
- **연결 이슈**: #1233
- **base/head**: `devel` ← `feature/issue-1233-eq-bigop-spacing`
- **Head SHA**: `cec32c77d45a7fcad5c568d43cba82fa1f501f1d`
- **PR 기준 base SHA**: `f83c43b57ee4e9bf3e5ecb8be73f53a81f290430`
- **현재 local/devel**: `5752a1f7` (#1234 반영 후)
- **규모**: 9 files, +372 / -5, 1 commit
- **mergeable**: true
- **PR 댓글**: 없음

## 1. PR 요약

PR #1235는 #1233에서 보고된 수식 큰 연산자 뒤 피연산자 간격 문제를 수정한다.

증상은 다음과 같다.

```text
∑bₙ
∫f(x)
```

처럼 큰 연산자와 피연산자가 붙어 보이는 문제이며, 한컴 PDF 정답지는 큰 연산자 뒤에 적정 간격을 둔다.

핵심 구현은 `equation/layout.rs`에서 큰 연산자 box width에 trailing pad를 더하고,
SVG/canvas 렌더러는 연산자 자체를 pad 제외 폭에 중앙정렬하도록 맞추는 것이다.

## 2. 주요 변경 범위

| 영역 | 변경 |
|---|---|
| `src/renderer/equation/layout.rs` | `BIG_OP_TRAIL_PAD = 0.45` 추가, ∑/∏/∫ width에 trailing pad 반영 |
| `src/renderer/equation/svg_render.rs` | limits 연산자 중앙정렬 기준을 `lb.width`에서 `lb.width - pad`로 보정 |
| `src/renderer/equation/canvas_render.rs` | 웹 canvas 렌더러도 SVG와 동일하게 중앙정렬 보정 |
| `mydocs/plans/task_m100_1233*.md` | 수행/구현 계획 문서 추가 |
| `mydocs/working/task_m100_1233_stage*.md` | 단계별 작업 기록 추가 |
| `mydocs/report/task_m100_1233_report.md` | 최종 보고서 추가 |

## 3. 타당한 부분

### 3.1 수정 위치

원인은 `layout_row`가 형제 노드를 `x += b.width` 방식으로 붙여 배치하는 데 있고,
일반 연산자는 `layout_symbol`에서 자체 pad를 갖지만 큰 연산자 계열은 trailing pad가 없다는 점이다.

PR은 `layout_row`에 일반 커닝을 도입하지 않고, 문제가 되는 BigOp width만 보정한다.
영향 범위를 수식 큰 연산자에 한정한 점은 적절하다.

### 3.2 SVG/canvas 동시 보정

BigOp box에 pad를 더하면 limits 연산자를 그대로 `lb.width` 중앙에 놓을 경우 연산자 자체가 우측으로 밀릴 수 있다.
PR은 SVG와 canvas 모두 `center_w = lb.width - fs * BIG_OP_TRAIL_PAD`를 기준으로 중앙정렬해,
pad 전체를 trailing 공간으로 남기도록 맞춘다.

웹 canvas와 SVG가 같은 시각 모델을 공유해야 하므로 이 보정은 필요하다.

### 3.3 레이아웃 안전성 주장

PR 설명대로 inline 수식은 control advance에 맞춰 가로 스케일되므로, 자연폭 증가가 곧바로 본문 advance 증가로 이어지지는 않는다.
따라서 본문 페이지네이션 영향은 제한적일 가능성이 높다.

다만 실제 `dump-pages` 불변과 SVG/WASM 시각 판정은 현재 `local/devel` 기준으로 다시 확인해야 한다.

## 4. 확인 필요 사항

### 4.1 PR base가 현재 devel보다 뒤처짐

PR 기준 base는 `f83c43b5`이고 현재 `local/devel`은 `5752a1f7`이다.

그 사이 #1232, #1234 등이 반영되었으므로 현재 devel 기준 검증 브랜치에서 병합해야 한다.
코드 충돌 가능성은 낮아 보이지만, 수식/렌더링 테스트는 반드시 재실행해야 한다.

### 4.2 pad 값 문서 불일치

최종 보고서와 코드/PR 본문은 `BIG_OP_TRAIL_PAD = 0.45`를 최종값으로 설명한다.
하지만 `task_m100_1233_stage3.md`에는 아직 다음과 같이 남아 있다.

```text
BIG_OP_TRAIL_PAD = 0.1
→ pad 값 0.1 확정
```

이 상태로 수용하면 후속 작업자가 최종값을 오해할 수 있다.
수용 과정에서 Stage 3 문서를 `0.1 → 0.25 → 0.45` 확정 흐름으로 보완하는 것이 좋다.

### 4.3 테스트 범위

PR의 단위 테스트는 BigOp width에 trailing pad가 들어갔는지 확인한다.
다만 실제 문제는 TAC 수식이 본문 control width에 맞춰 압축되는 시각 결과이므로,
다음 시각 판정이 필요하다.

```text
samples/3-09월_교육_통합_2023.hwp 6페이지 문26
∑bₙ, 첫 ∑(...), 문25 적분 ∫ 주변
```

## 5. 권장 검증

현재 devel 기준 검증 브랜치에서 다음을 실행한다.

```text
git diff --check HEAD
cargo fmt --all --check
cargo test --lib equation
cargo test --test issue_1219_equation_line_hangul_advance
cargo test --lib
cargo test --tests
docker compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
```

메인테이너 시각 판정용 SVG:

```text
target/debug/rhwp export-svg samples/3-09월_교육_통합_2023.hwp -p 5 -o output/poc/pr1235-eq-bigop-spacing
```

필요하면 `--debug-overlay`, `--show-grid=3mm`도 별도 산출한다.

## 6. 권장 처리

권장안: **수용 후보로 진행한다. 단, 현재 `local/devel` 기준 검증 브랜치에서 병합하고, Stage 3 문서의 pad 값 불일치를 보완한 뒤 자동 테스트/WASM/Studio 빌드와 메인테이너 시각 판정을 게이트로 둔다.**

이 PR은 수식 큰 연산자 내부 간격만 겨냥하고, SVG/canvas 렌더러 양쪽 보정도 포함하고 있어 방향은 좋다.
다만 수식은 실제 사용자가 보는 시각 차이가 핵심이므로 자동 테스트 통과만으로 종료하지 않고 maintainer 시각 판정을 거쳐야 한다.

## 7. 다음 승인 요청

다음 단계로 진행하려면 작업지시자 승인이 필요하다.

권장 절차:

```text
1. `local/pr1235-verify` 브랜치를 현재 `local/devel`에서 생성
2. PR #1235를 병합 시뮬레이션
3. Stage 3 문서의 `0.1 확정` 표현을 최종 `0.45 확정` 흐름으로 보완
4. 자동 테스트/WASM/Studio 빌드
5. 6페이지 수식 SVG 및 웹 canvas 시각 판정 후 local/devel 반영
```
