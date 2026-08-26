---
kind: working-note
status: completed
issue: 4939
stage: 2
last_verified: 2026-08-16
---

# Issue #4939 Stage 2 — W0 deterministic baseline

## 결론

Stage 1의 schema·source boundary 위에 deterministic collector와 canonical JSON writer를
구현하고, W0 candidate·baseline snapshot을 생성했다. metric·alias·fallback·font asset과
제품 source는 변경하지 않았다.

## 기준과 commit 경계

- Stage 1 commit: `acb1465a6026747231420945c30407e8f008a898`
- Stage 2 collector commit: `795e7b5fac24cfef79017e9120516570851a03b2`
- `upstream/devel`: `82f28ae8644110d4ccd1528447ab87ddf8ddce6f`로 변동 없음
- machine baseline `sourceCommit`: Stage 2 collector commit

도구를 먼저 commit한 뒤 그 commit을 입력으로 snapshot을 만들었다. 따라서 baseline이 아직 commit되지
않은 generator bytes를 가리키거나, snapshot이 자신을 포함한 미래 commit을 가리키는 자기참조를 피했다.

## RED→GREEN

Stage 2 test를 먼저 추가했을 때 collector export가 없어 다음 오류로 실패했다.

```text
SyntaxError: The requested module '../font_rule_ledger.mjs'
does not provide an export named 'buildBaseline'
```

구현 뒤 결과는 다음과 같다.

```text
tests 10
pass 10
fail 0
```

추가된 계약은 source candidate 30개 폐합, 반복 수집 결정성, 600/401 metric 구조,
lookup 순서와 projection hash, canonical object key·array order·최종 newline이다.

## 생성과 결정성

```bash
node scripts/font_rule_ledger.mjs collect \
  --out mydocs/tech/investigations/issue-4939/font_rule_candidates.json
node scripts/font_rule_ledger.mjs baseline \
  --candidates mydocs/tech/investigations/issue-4939/font_rule_candidates.json \
  --out mydocs/tech/investigations/issue-4939/font_rule_baseline.json
```

같은 출력 경로에 두 번 생성해 각 실행의 SHA-256을 비교했다.

| 산출물 | 첫 실행 | 둘째 실행 | 판정 |
| --- | --- | --- | --- |
| candidates | `7a505a228fc6ded6fcc88679d2f3f3340cb40cef7d7fab4e914cdb07827394a4` | 동일 | match |
| baseline | `a0fac05c3138471eb3e7404fc701f0053caa6c01a923afae60fd4da64064b466` | 동일 | match |

## focused gate

```bash
node --test scripts/tests/font_rule_ledger.test.mjs
cargo test --profile release-test --lib font_metrics
node --test \
  rhwp-studio/tests/font-substitution.test.ts \
  rhwp-studio/tests/local-fonts.test.ts \
  rhwp-studio/tests/canvaskit-font-plan.test.ts \
  rhwp-studio/tests/canvaskit-sfnt-face.test.ts \
  rhwp-studio/tests/renderer-baseline-font-loading.test.ts
node --test scripts/frontend-font-assets.test.mjs
```

| gate | 결과 |
| --- | --- |
| ledger·collector | 10 passed |
| Rust font metrics | 9 passed, 4,038 filtered out |
| Studio font | 33 passed |
| frontend font asset | 6 passed |

## fresh native/WASM parity

```bash
cargo build --release
wasm-pack build --target web --out-dir pkg
node scripts/svg_native_wasm_diff.mjs \
  samples/exam_kor.hwp samples/exam_eng.hwp \
  samples/exam_math.hwp samples/exam_science.hwp \
  samples/synam-001.hwp samples/aift.hwp samples/2010-01-06.hwp
```

- native/WASM version: `0.8.4` 일치
- 7개 공개 fixture, 167페이지
- 7개 문서 모두 `match`, byte 불일치 0

로컬 하네스 보고서에는 절대 경로가 있으므로 commit하지 않았다. 정규화한 path·fixture digest·페이지
결과만 baseline manifest에 기록했다.

## 종료 게이트 판정

- `sourceCommit`: collector를 포함한 실제 입력 commit으로 고정
- input digest: 21개 파일, candidate 수집 뒤 변경 시 fail closed
- metric: 600 entry, 401 unique name, 중복 style key 0
- lookup: 464 input × 4 style = 1,856 projection
- current behavior: Rust legacy 등가, Studio fallback, asset, native/WASM 전부 통과
- private corpus: 접근·재측정·산출물 포함 없음
- 제품 diff: `src/`, `rhwp-studio/src/`, `web/` 변경 없음

Stage 2 종료 게이트를 만족한다.

## 다음 승인 지점

Stage 3에서는 30개 selector boundary 내부의 finite mapping을 실제 행으로 확장하고, ordered chain과
algorithmic predicate를 별도 kind로 수집한다. owner별 candidate 또는 명시적 `not-applicable`,
인식하지 못한 실행 mapping block 0개가 종료 조건이다. 메인테이너 승인 전에는 시작하지 않는다.
