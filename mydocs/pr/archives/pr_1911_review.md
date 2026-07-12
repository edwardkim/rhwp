# PR #1911 Review — task 1655: HWPX 수식 flowWithText 보존

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1911 |
| 작성자 | `jangster77` |
| base | `devel` |
| head | `task_m100_1655_equation_flowwithtext` |
| 관련 이슈 | https://github.com/edwardkim/rhwp/issues/1655 |
| 문서 작성 시점 코드 변경 규모 | 5 files, +170/-1 |
| 문서 작성 시점 참고 SHA | `10556bb2a03c8bdfd2032a479d5dba5fbb4002a0` |

`draft`, `mergeable`, CI 상태는 merge 전 최신 PR head 기준으로 다시 확인한다.

## 관련 이슈 요약

#1655는 HWPX 수식(`hp:equation`)의 `hp:pos@flowWithText`가 roundtrip에서 보존되지 않는 문제를
다룬다. parser는 이미 `flowWithText`를 `Equation.common.flow_with_text`로 읽을 수 있지만,
serializer는 수식 경로에서 `flowWithText="1"`을 고정 방출하고 있었다.

## 변경 범위

- `src/serializer/hwpx/section.rs`
  - `render_equation`에서 `flowWithText="1"` 하드코딩을 제거했다.
  - 입력 문서에서 읽거나 IR에 설정된 `Equation.common.flow_with_text` 값을 방출한다.
- `src/serializer/hwpx/mod.rs`
  - 수식 `flow_with_text=false`가 XML에서 `flowWithText="0"`으로 방출되고 재파싱 뒤에도 보존되는
    roundtrip 테스트를 추가했다.
- `src/serializer/hwpx/roundtrip.rs`
  - `Control::Equation` 비교에서도 기존 `ObjectFlowWithText` 게이트를 재사용한다.
  - 수식 `flowWithText` 차이가 IR diff에서 검출되는지 테스트를 추가했다.
- 문서
  - `mydocs/plans/task_m100_1655.md`
  - `mydocs/working/task_m100_1655_stage1.md`

## 렌더 영향 및 visual sweep 판정

이 PR은 HWPX serializer/roundtrip 보존 게이트 변경이다. renderer/layout/typeset/paint 경로는 수정하지
않고, 페이지 수 또는 기준 PDF 대비 시각 정합을 주장하지 않는다.

따라서 visual sweep 대상이 아니다. 검증은 XML 방출값, 재파싱 IR, `diff_documents` 게이트를 중심으로
수행했다.

## 로컬 검증

```bash
env CARGO_INCREMENTAL=0 cargo test --lib task1655 -- --nocapture
cargo fmt --check
env CARGO_INCREMENTAL=0 cargo test --lib serializer::hwpx -- --nocapture
git diff --check
env CARGO_INCREMENTAL=0 cargo clippy --lib -- -D warnings
```

결과:

- `task1655` focused test 2개 통과
- `serializer::hwpx` 범위 275개 통과
- fmt check 통과
- diff check 통과
- `clippy --lib -D warnings` 통과

## 리스크와 범위 밖

- 수식 렌더링 품질이나 수식 파서 문법을 바꾸는 PR이 아니다.
- HWP5 binary 수식 저장 경로는 변경하지 않는다.
- `allowOverlap`, `affectLSpacing` 같은 다른 수식 `hp:pos` 속성의 보존은 별도 이슈 범위다.

## 최종 권고

PR head 최신 커밋 기준 GitHub Actions가 통과하면 merge 후보로 판단한다.
옵션 1 self PR 경로이므로 review 문서와 오늘할일을 PR head에 포함한다.

