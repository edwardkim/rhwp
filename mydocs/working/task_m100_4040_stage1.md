# task_m100_4040 Stage 1 — 파일 게이트 native-skia test 3건 CI 회복

- **이슈**: [#4040](https://github.com/edwardkim/rhwp/issues/4040)
- **브랜치**: `issue-4040-native-targets`
- **분기 기준**: `upstream/devel` `d634e608be45d2fd072364a21952a8409d01d9ea`
- **계획서**: [수행](../plans/task_m100_4040.md) / [구현](../plans/task_m100_4040_impl.md)
- **기록일**: 2026-08-07 KST

## 1. 조사에서 계획서를 두 번 고쳤다

착수 전 조사와 RED 재현 과정에서 계획서의 전제가 두 번 틀렸다. 사후에 다듬지 않고 경위를 남긴다.

### 1.1 판별식이 좁았다 — 중첩 게이트를 놓쳤다

처음 쓴 판별식은 정확 일치였다.

```python
r'#!\[cfg\(feature\s*=\s*"native-skia"\)\]'
```

이걸로는 `render_p37_direct_pdf_export.rs` 를 놓친다. 실제 게이트는 중첩이다.

```rust
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
```

RED 재현 때 "알려진 5건이 발견되는지" 단언이 3건만 찾아 실패하면서 드러났다. **발견 패턴 자체를
단언하는 테스트가 없었다면 조용히 넘어갔을 오류**다. inner attribute 안의 feature 언급을 보도록 고쳤다.

### 1.2 `issue_2225`·`render_p37` 의 성격을 잘못 봤다

계획서는 두 파일을 "파일 게이트 정상 사례"로 적었다. 실제로는 다르다.

| 파일 | 게이트 형태 | job | classifier |
| --- | --- | --- | --- |
| `issue_2083_hide_fill_page_background.rs` | 파일 (단순) | ✗ | ✗ |
| `issue_2292_chart_png_clip.rs` | 파일 (단순) | ✗ | ✗ |
| `issue_2293_chart_png_text.rs` | 파일 (단순) | ✗ | ✗ |
| `render_p37_direct_pdf_export.rs` | 파일 (중첩 `all`) | ✓ | ✓ |
| `issue_2225_missing_picture_placeholder.rs` | **함수** | ✓ | ✓ |

즉 파일 게이트는 4건이고 그중 3건이 누락, `render_p37` 이 정상 사례다. `issue_2225` 는 함수 게이트인데도
job 에 등재된 별개 사례다.

이 정정은 [#4132](https://github.com/edwardkim/rhwp/issues/4132) 의 전제에도 영향을 준다 — 3절 참조.

## 2. 구현

### 2.1 `ci.yml` — Native Skia job

`--test` 3개를 release-test·release 두 경로 모두에 추가했다. 기존 두 target 과 같은 형식이다.

### 2.2 `ci-impact-classifier.cjs`

`NATIVE_SKIA_RUST_FILES` 에 세 경로를 추가하고 정렬했다. 이 목록이 workflow 의 `--test` 목록과 짝이며
계약 테스트가 양방향으로 강제한다는 주석을 남겼다.

이 변경이 없으면 **세 파일을 고치는 PR 에서 `native_skia_required=false` 로 판정되어 Native Skia job 이
skip 되고, 추가한 target 이 돌지 않는다.** 판정 변화는 이렇다.

```
before: rust=true  native=false  reason=classified:rust
after : rust=true  native=true   reason=classified:native-skia-rust
```

`render_required` 는 `false` 로, 기존 `issue_2225`·`render_p37` 과 동일하다. fixture 작성 시 `true` 로
잘못 짚었다가 실행 결과로 정정했다.

### 2.3 계약 테스트 — 부류를 강제한다

`test_ci_impact_workflow.py` 에 셋을 더했다.

| 테스트 | 역할 |
| --- | --- |
| `test_every_file_gated_native_skia_test_is_wired` | 저장소를 훑어 파일 게이트 test 가 job·classifier 양쪽에 있는지 |
| `test_discovery_finds_the_known_file_gated_native_skia_tests` | 발견 패턴이 망가지면 위 테스트가 무의미해지므로 패턴 자체를 단언 |
| `test_native_skia_targets_run_in_both_profiles` | release-test 와 release 경로가 같은 target 집합인지 |

기존 `test_native_skia_integration_targets_are_classifier_inputs` 는 **남겼다.** job 에는 있는데
classifier 에 없는 역방향을 계속 감시한다. 두 테스트가 양방향을 이룬다.

## 3. #4132 전제 정정 필요

[#4132](https://github.com/edwardkim/rhwp/issues/4132)(함수 단위 게이트)를 등록할 때 "중복 실행은 설계
판단이 필요한 미지수"로 적었다. 그런데 `issue_2225_missing_picture_placeholder.rs` 는 `#[test]` 2건 중
1건만 게이트되어 있고 이미 Native job 에 등재돼 있다. 즉 **비게이트 1건이 default worker 와 Native job
에서 이미 중복 실행 중이며, 저장소가 그 비용을 이미 받아들이고 있다.**

#4132 의 선택지 A(중복 수용)는 미지수가 아니라 선례가 있는 선택지다. 코멘트로 정정한다.

## 4. 검증

### 4.1 RED 재현

수정 전 상태에서 신규 계약 테스트가 정확히 세 파일을 지목하며 실패했다.

```
AssertionError: Lists differ: ['issue_2083_hide_fill_page_background',
 'issue_2292_chart_png_clip', 'issue_2293_chart_png_text'] != []
```

### 4.2 뮤테이션

| 뮤테이션 | 결과 |
| --- | --- |
| job 에서 `issue_2293` 두 경로 제거 | 1건 실패 |
| classifier 에서 `issue_2292` 제거 | 2건 실패 |
| release 경로에서만 `issue_2083` 제거 (프로파일 비대칭) | 1건 실패 |
| 발견 정규식을 중첩 미지원으로 축소 | 1건 실패 (`render_p37` 놓침) |

### 4.3 로컬 실행 — 세 테스트 실제 통과

`native-skia` 로컬 빌드가 가능해 원격 CI 를 기다리지 않고 직접 실행했다.

```
issue_2293_chart_png_text        chart_png_renders_text_labels ... ok
issue_2292_chart_png_clip        chart_png_renders_full_bbox_not_top_left_fragment ... ok
issue_2083_hide_fill_page_background  hide_fill_page_renders_opaque_white_not_transparent_black ... ok
```

셋 다 `1 passed; 0 failed`. **회복시킨 가드가 실제로 동작하는 테스트임을 확인했다** — 죽은 테스트를
되살린 것이 아니라 유효한 검증이 CI 에서 빠져 있던 것이다.

### 4.4 회귀

| 검증 | 결과 |
| --- | --- |
| workflow 계약 테스트 5개 파일 | 57 passed / 0 failed |
| `node --test ci-impact-classifier.test.cjs` | 28 passed (fixture 1건 추가) |
| `actionlint .github/workflows/ci.yml` | 통과 |
| `node --check scripts/ci-impact-classifier.cjs` | 통과 |
| `git diff --check` | 통과 |

### 4.5 PR #4170 변경 요청 보정 — 2026-08-08

`edwardkim`의 `CHANGES_REQUESTED` 리뷰가 넓은 쪽 오탐 두 부류와 classifier 독립성 공백을 지적했다.

- Rust 문자열·줄/중첩 블록 주석을 마스킹하고 brace depth 0의 crate inner attribute만 찾는다.
- cfg `all`·`any`·`not`을 재귀 하강 parser와 3값 의미 평가로 처리한다. native-skia 비활성 상태에서
  반드시 거짓이면서 활성 상태에서 가능성이 생길 때만 파일 게이트로 분류한다.
- 리뷰의 `any(...)`, raw string, 블록 주석 입력과 중첩 부정·블록 내부 attribute를 회귀로 고정했다.
- classifier 단위 테스트에 세 경로를 각각 단독 입력으로 추가하고 묶음 fixture는 통합 사례로 유지했다.
- 최신 `upstream/devel` `1ede9c7ac`을 merge commit `cd427c37e`로 반영했다. 오늘할일 충돌은 PR의
  `#4080`·`#4040`·`#4132`와 devel의 `PR #4174` 기록을 모두 보존했다.

보정 뒤 workflow 계약 5개 파일 63건, classifier 28건, `actionlint`, `node --check`, `git diff --check`가
통과했다. 전체 `scripts/tests` discovery는 102건 통과 뒤 로컬 Python의 Pillow 미설치 때문에
`test_visual_sweep.py` import 1건만 실패했다. 시각 sweep은 이번 CI 계약 보정 범위가 아니다.

## 5. 다음 단계

1. 작업지시자가 승인한 보정 push와 완료 코멘트 게시 → 새 head 전체 CI와 재검토 확인.
2. Native Skia job 소요시간 증가폭 실측. 현재 368~382초 기준으로 과다하면 계획서 3절 재검토.
3. #4132 전제 정정 코멘트.
