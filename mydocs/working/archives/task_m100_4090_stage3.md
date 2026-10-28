# #4090 Stage 3 - 저장 HWPX tail 가드 적용 범위 검증

## 목적

Stage 1의 `hwpx_explicit_page_break_tail_line`이 실제 fixture에는 적용되면서, 일반 문단의
`vpos=0` 또는 재flow된 줄에 과도하게 적용되지 않음을 단위 계약으로 고정한다.

## 분석

호출부는 `LayoutCompatibilityProfile::hwpx_stored_layout()`일 때만 이 함수를 연결한다.
함수 자체는 아래 조건을 모두 요구한다.

1. 컨트롤이 없는 보이는 텍스트 문단이다.
2. 형식화 줄 수와 저장 `line_segs` 수가 정확히 일치하고 두 줄 이상이다.
3. 다음 문단이 `Page` 또는 `Section` 명시 나눔이다.
4. 마지막 줄만 `vpos=0`이고, 이전 줄은 합성 줄이 아니다.
5. 이전 줄은 본문 하단 30% 안에 있으며 줄 하단이 본문을 넘지 않는다.

따라서 HWP3/HWP5 계보는 호출되지 않고, HWPX라도 단순 `vpos=0`이나 재flow line-count
불일치, 중간 본문 `vpos`에는 적용되지 않는다.

## 구현 계획

1. `src/renderer/typeset.rs`의 기존 단위 테스트 모듈에 최소 문단 생성기를 둔다.
2. 양성 경로에서 마지막 한 줄 분할 위치 `Some(1)`을 검증한다.
3. 명시 나눔 부재, tail `vpos` 불일치, 저장/형식화 줄 수 불일치, 본문 하단 조건 불충족을
   각각 `None`으로 검증한다.
4. 단위 테스트와 Stage 2 fixture 회귀 테스트를 실행해 결과를 이 문서에 기록하고, 코드와
   문서를 함께 커밋한다.

## 결과

### 구현

`src/renderer/typeset.rs`의 기존 단위 테스트 모듈에 다음 계약을 추가했다.

- `Page`와 `Section` 명시 나눔 앞에서만 마지막 한 줄의 분할 위치 `Some(1)`을 반환한다.
- 다음 나눔 없음, `Column` 나눔, tail `vpos != 0`, 저장 줄 수 불일치, 이전 줄이
  본문 하단 30%보다 위인 경우, 줄 하단이 본문을 넘는 경우에는 모두 `None`을 반환한다.

호출부의 `hwpx_stored_layout()` 조건은 기존 동작으로 유지했으므로, HWP3와 일반 HWP5는
이번 가드를 호출하지 않는다.

### 실행 결과

```text
CARGO_TARGET_DIR=target/issue4090-pdf-17pages \
CARGO_INCREMENTAL=0 \
cargo test --profile release-test --lib hwpx_explicit_page_break_tail -- --nocapture

running 2 tests
test renderer::typeset::tests::hwpx_explicit_page_break_tail_requires_all_stored_layout_evidence ... ok
test renderer::typeset::tests::hwpx_explicit_page_break_tail_splits_only_last_stored_line ... ok

test result: ok. 2 passed; 0 failed
```

```text
CARGO_TARGET_DIR=target/issue4090-pdf-17pages \
CARGO_INCREMENTAL=0 \
cargo test --profile release-test --test issue_4090_hwpx_tail_page_break -- --nocapture

running 1 test
test issue_4090_hwpx_tail_lines_follow_the_explicit_page_break ... ok

test result: ok. 1 passed; 0 failed
```

`cargo fmt --check`와 `git diff --check`도 통과했다. rhwp PDF는 생성하지 않았고, 표준
근거는 `pdf/issue4090/`의 HWP 2020 MCP PDF만 유지한다.

## 다음 단계

구현과 fixture·단위 회귀 계약이 모두 고정됐다. 추가 코드 변경이 필요하다는 새 증거가
나오기 전까지는 다음 단계에서 PDF를 새로 생성하지 않고, 현재 브랜치의 커밋·작업 트리와
기준 PDF의 보존 상태를 점검해 완료 상태를 기록한다.
