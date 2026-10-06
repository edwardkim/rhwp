# 구현계획서 — task_m100_4040

- **이슈**: [#4040](https://github.com/edwardkim/rhwp/issues/4040)
- **수행계획서**: [`task_m100_4040.md`](task_m100_4040.md)
- **기록 시각**: 2026-08-07 KST

## 1. 파일별 변경

| 파일 | 변경 |
| --- | --- |
| `.github/workflows/ci.yml` | `Native Skia tests` job 에 `--test` 3개 추가 (release-test·release 두 경로 모두) |
| `scripts/ci-impact-classifier.cjs` | `NATIVE_SKIA_RUST_FILES` 에 3개 경로 추가 |
| `scripts/tests/test_ci_impact_workflow.py` | 부류 강제 계약 테스트 추가 |
| `scripts/tests/fixtures/ci-impact-classifier-prs.json` | 분류 fixture 추가 |
| `scripts/tests/ci-impact-classifier.test.cjs` | fixture 기대값 추가 |
| `mydocs/plans/task_m100_4040{,_impl}.md` | 계획서 |
| `mydocs/working/task_m100_4040_stage1.md` | 단계 기록 |
| `mydocs/pr/archives/pr_<N>_review.md` | PR 번호 발급 후 |
| `mydocs/orders/20260807.md` | 오늘할일 |

## 2. `ci.yml` — Native Skia job

현재 두 경로가 각각 3줄이다(release-test / release). 각 경로에 3줄을 더한다.

```bash
cargo test --profile release-test --features native-skia --test issue_2083_hide_fill_page_background --verbose
cargo test --profile release-test --features native-skia --test issue_2292_chart_png_clip --verbose
cargo test --profile release-test --features native-skia --test issue_2293_chart_png_text --verbose
```

기존 두 target 과 같은 형식을 유지한다. job 의 조건·의존은 건드리지 않는다.

**비용**: target 3개 추가로 Native Skia job 이 길어진다. 현재 이 job 은 실측 368~382초다. 세 테스트는
각각 `#[test]` 1건이고 이미 빌드된 산출물을 재사용하므로 증가폭은 링크·실행 시간 정도로 예상하나,
**실측은 원격 CI 로 확인한다.** 예상보다 크면 수행계획서 3절의 범위를 재검토한다.

## 3. `ci-impact-classifier.cjs`

```js
const NATIVE_SKIA_RUST_FILES = new Set([
  'tests/issue_2083_hide_fill_page_background.rs',
  'tests/issue_2225_missing_picture_placeholder.rs',
  'tests/issue_2292_chart_png_clip.rs',
  'tests/issue_2293_chart_png_text.rs',
  'tests/render_p37_direct_pdf_export.rs',
]);
```

정렬 순서로 넣어 다음 추가 때 diff 가 깨끗하게 남게 한다. 판정 결과는
`rust=true, native=true, reason=classified:native-skia-rust` 로, 기존 두 파일과 같아진다.

## 4. 계약 테스트 — 부류를 강제한다

기존 `test_native_skia_integration_targets_are_classifier_inputs` 는 한 방향만 본다. **저장소를 훑어
파일 게이트된 test 를 발견하는** 테스트를 추가한다.

```python
def test_every_file_gated_native_skia_test_is_wired(self):
    """`#![cfg(feature = "native-skia")]` 파일 게이트 test 는
    Native Skia job 과 classifier 양쪽에 모두 있어야 한다.

    양쪽 어디에도 없으면 기존 단방향 테스트의 순회 대상이 아니라
    조용히 빠진다 — #2083·#2292·#2293 이 그 경로로 새어 나갔다.
    """
```

판정 규칙: `tests/*.rs` 중 선두에 `#![cfg(feature = "native-skia")]` 가 있는 파일마다
① Native Skia step 에 `--test <stem>` 이 있고 ② classifier 에 `'tests/<stem>.rs'` 가 있어야 한다.

발견 패턴 자체가 망가지면 조용히 무의미해지므로, 알려진 5개 파일이 발견되는지도 함께 단언한다
(#4080 의 `test_workflow_contract_wiring.py` 와 같은 방어).

기존 단방향 테스트는 **남긴다.** 역방향(job 에는 있는데 classifier 에 없음)을 여전히 잡는다.

## 5. classifier fixture

`ci-impact-classifier-prs.json` 에 세 파일 단독 변경 표본을 추가하고, `ci-impact-classifier.test.cjs`
기대값을 `rust=true, native=true, codeql`·`reason=classified:native-skia-rust` 로 고정한다.

기존 fixture 27건의 기대값은 건드리지 않는다.

## 6. 검증 계획

| 단계 | 검증 | 기대 |
| --- | --- | --- |
| RED | 계약 테스트만 먼저 추가 | 3건 실패 (job·classifier 양쪽 누락) |
| GREEN | workflow·classifier 수정 | 전건 통과 |
| 회귀 | `test_ci_impact_workflow.py` 기존 18건 | 무회귀 |
| 회귀 | `node --test ci-impact-classifier.test.cjs` | 기존 27건 + 신규 무회귀 |
| 회귀 | #4080 계약 테스트 25+3건 | 무회귀 |
| lint | `actionlint .github/workflows/ci.yml` | 진단 없음 |
| 뮤테이션 | job 에서 `--test issue_2293…` 제거 | 신규 테스트 실패 |
| 뮤테이션 | classifier 에서 한 경로 제거 | 신규 테스트 실패 |
| 원격 | Native Skia job 로그 | 세 target 실행·통과, 소요시간 기록 |

로컬 `native-skia` 빌드 가능 여부는 착수 시 확인한다. 가능하면 세 테스트를 직접 실행하고, 불가하면
원격 CI 로그를 1차 증적으로 삼는다는 사실을 Stage 기록에 명시한다.

## 7. 롤백

workflow 3줄 × 2경로와 classifier 3줄이 전부라 revert 로 즉시 되돌아간다. 되돌리면 검증 공백이
원상 복귀할 뿐 다른 판정에 영향이 없다.

## 8. 승인 요청 사항

착수 전 다음을 확인받는다.

1. **범위 확대** — 이슈는 `issue_2293` 하나만 지목했으나 같은 상태의 파일 3건 전부를 대상으로 한다.
2. **함수 단위 cfg 제외** — `tests/cli_exit_codes.rs` 의 native-skia 함수는 별도 이슈로 분리한다.
3. **Native Skia job 시간 증가** 를 받아들인다. 실측 후 과다하면 재검토한다.
