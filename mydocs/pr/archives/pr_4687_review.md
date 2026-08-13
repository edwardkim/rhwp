---
kind: review
status: accepted
pr: 4687
issue: 4680
author: planet6897
base: devel
---

# PR #4687 검토 기록

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#4687](https://github.com/edwardkim/rhwp/pull/4687) |
| 작성자 | `planet6897` (Jaeuk Ryu) |
| 기준 브랜치 | `devel` |
| 검토 경로 | collaborator 매개 외부 PR, source head 직접 보정 |
| 관련 이슈 | [#4680](https://github.com/edwardkim/rhwp/issues/4680) |
| 원 기여자 code head | `0eb105d99f6a0488b83b2b6e73a41799b4f275d1` |
| 메인터너 보정 head | `1cd726ba3134a0db3a33c5c10c1ad789f75a9cd2` |
| 문서 작성 시점 mergeable | `CLEAN` 참고값. merge 직전에 다시 확인 필요 |

### 적용 절차

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md, review_only_fast_pass.md, post_merge.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
collaborator_external_pr.md, intake_and_review.md, local_validation.md,
review_only_fast_pass.md, post_merge.md
current code candidate: 1cd726ba3134a0db3a33c5c10c1ad789f75a9cd2
```

`maintainerCanModify=true`을 확인하고, 사용자가 열어 둔 주 작업공간의
`review/planet6897-4687-20260813` 브랜치에서 contributor source history 위에만 보정을
추가했다. contributor commit은 rebase, amend, force-push 하지 않았다.

## 변경 검토

원 기여자 변경은 HWP3에서 첫 문단의 `SectionDef`/`ColumnDef` 제어문자가 본문 뒤로 밀리지 않도록
제어문자 슬롯을 예약한다. 다만 초기 변경은 `char_offsets`만 이동했다. 같은 UTF-16 문단 좌표계인
`char_shapes`, `range_tags`, `line_segs.text_start`를 그대로 두면 제어문자 뒤의 스타일, 태그, 줄 시작
좌표가 본문보다 앞선 위치를 가리킨다. 이는 HWP 직렬화 계약을 깨는 P1 결함이었다.

메인터너 보정 `1cd726ba3`은 `Paragraph`에 공통 선행 확장 제어문자 슬롯 예약 API를 추가하고, 기존
inline 삽입과 동일하게 모든 위치 메타데이터를 이동시켰다. HWPX-to-HWP 변환과 body-text 직렬화
fallback 모두 이 공통 API를 사용하도록 정리했다. 제어문자를 이미 위한 빈 슬롯이 있는 IR은 재이동하지
않는다.

변경 범위는 다음 세 파일이다.

- `src/document_core/converters/hwpx_to_hwp.rs`
- `src/model/paragraph.rs`
- `src/serializer/body_text.rs`

renderer/layout, fixture, baseline, CI workflow 변경은 없다. 따라서 별도 시각 sweep은 판단 근거로
선택하지 않았다. 한글 2022 실제 개방 결과는 contributor가 [#4680 진행 코멘트](https://github.com/edwardkim/rhwp/issues/4680)에
남긴 외부 오라클 증적으로만 참조했고, 이 검토에서 독립적인 Windows COM 재실행 결과라고 주장하지 않는다.

## 검증 결과

다음 검증은 메인터너 보정 후 `target/pr-review`에서 순차로 완료했다.

| 검증 | 결과 |
| --- | --- |
| `cargo test --profile release-test --target-dir target/pr-review --lib issue4680 -- --nocapture` | 3 passed |
| `cargo test --profile release-test --target-dir target/pr-review --lib shift_for_inline_control_insert -- --nocapture` | 2 passed |
| `cargo test --profile release-test --target-dir target/pr-review --lib test_roundtrip_with_section_def_control -- --nocapture` | 1 passed |
| `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 5,785 passed, 36 skipped, 7 slow, exit 0 |
| `cargo fmt --check` | passed |
| `cargo clippy --all-targets -- -D warnings` | passed |
| `git diff --check` | passed |
| 최신 `upstream/devel` merge tree | 충돌 없음 |

동일 code head의 GitHub Actions도 모두 성공했다.

- [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/31622416215): 성공
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31622415987): Rust, Python, JavaScript/TypeScript 성공
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/31622416016): Canvas visual diff 성공

## 남은 위험과 후속 과제

이번 PR은 HWP3→HWP 저장 시 선행 구역 정의 제어문자 좌표 계약만 해결한다. HWP3→HWPX 개방 실패,
`03908.h2h`의 0쪽 결과, 본문 보존 불일치는 [#4680](https://github.com/edwardkim/rhwp/issues/4680)에
계속 남긴다. 이 PR의 merge로 해당 이슈를 닫지 않는다.

## 최종 권고

**수용 및 merge 권고.** 메인터너 보정이 원 변경의 좌표 계약 누락을 해소했고 focused·전체 로컬 회귀와
동일 code head의 Full CI, CodeQL, Render Diff가 성공했다. 이 문서와 오늘할일만 추가한 trailing
docs-only head의 preflight/aggregate, 최신 mergeability를 확인한 뒤 작업지시자 승인에 따라 merge한다.
