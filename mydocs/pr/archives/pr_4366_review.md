---
kind: pr_review
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4366 검토 - HWP3 글자 음영 저장 정합

## 라우팅

base route: `maintainer_general.md`

modifiers: `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`, `visual_fixture_evidence.md`, `rework_and_exceptions.md`

누적 체리픽과 메인터너 보정을 포함하므로 실행 순서는
[`pr_4366_4499_review_impl.md`](pr_4366_4499_review_impl.md)에 분리해 기록한다.

## 범위와 기준점

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4366](https://github.com/edwardkim/rhwp/pull/4366) / @johndoekim |
| 원 source head | `311cbc2badc8aa8bf6b5a1c7c8acd35b3fbee6ea` |
| 기준 `upstream/devel` | `8dbe982e89e780fe0612a1bc66aa417bbd6356b2` |
| 누적 검토 브랜치 | `review/johndoekim-20260811` |
| 원 변경 | 21파일, `+1377/-73`, 8 commits |
| 원 PR 상태 확인 | open, mergeable, 원 source head의 required checks 성공 |
| 검토 보정 | `7e37e5b08` - HWPX 실제 검정 음영 보존 |

원 변경은 HWP3 글자 음영의 팔레트 인덱스와 `shade_ratio`를 합성하고, 비율 0을
`0xFFFFFFFF` 음영 없음 sentinel으로 저장한다. 이 변경은 한컴에서 본문을 덮던 검정 막대를
제거하는 저장 정합 수정이다.

## 발견 사항과 메인터너 보정

원 변경은 HWP3의 `100%` 검정 음영을 IR `0x00000000`으로 올바르게 만들지만, HWPX 라이터에
남아 있던 `shade_color == 0 -> shadeColor="none"` 분기가 이 값을 소거했다. `0x00000000`은
실제 불투명 검정이고, 음영 없음은 `0xFFFFFFFF`뿐이므로 HWPX 저장 시 `#000000`을 내보내야 한다.

메인터너 보정 `7e37e5b08`은 `color_hex`의 sentinel 판정을 단일 정본으로 사용하고,
`write_char_pr_preserves_opaque_black_shade` 회귀 테스트를 추가했다. 따라서 다음 두 경우가
분리된다.

| IR 값 | HWPX `shadeColor` | 의미 |
| --- | --- | --- |
| `0xFFFFFFFF` | `none` | 음영 없음 |
| `0x00000000` | `#000000` | HWP3 검정 팔레트 100%의 실제 음영 |

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| 원 source head와 GitHub required checks 대조 | `311cbc2`에서 성공, mergeable 확인 |
| `git diff --check` | 통과 |
| `cargo fmt --check` | 통과 |
| `cargo clippy --profile release-test --all-targets -- -D warnings` | 통과 |
| `write_char_pr_preserves_opaque_black_shade` | 1 passed |
| `issue_4155_hwp3_char_shade_contract` | 7 passed |
| 누적 전체 | `cargo nextest run --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast`: **5,730 passed, 7 slow, 36 skipped**, 437.285s |
| HWP 2020 PDF - `samples/SO-SUEOP.hwp` 원본 | 46쪽, 1,282,291 bytes |
| HWP 2020 PDF - rhwp HWP 변환본 | 47쪽, 1,174,638 bytes, 본문 검증 성공 |

실제 한컴 PDF의 3쪽을 검토했다. 변환본에는 이 이슈의 증상인 줄 크기 검정 막대가 없고 본문이
판독된다. 안정 증적은 [원본 3쪽](../assets/pr_4366_so_sueop_source_p003_hancom2020.png)과
[변환본 3쪽](../assets/pr_4366_so_sueop_rhwp_p003_hancom2020.png)에 보존했다.

원본 46쪽과 변환본 47쪽의 자동 번호, 머리말, 들여쓰기, 페이지 나눔 차이는 남아 있다. 이는
원 PR의 한컴 판정 보고서가 이미 #2151 계열 기존 fidelity 항목으로 분리한 범위이며, 이번 변경은
글자 음영 값만 변경한다. 검정 막대 소멸 판정과 혼동하지 않으며, 레이아웃 fidelity의 종결 판정은
[#3820](https://github.com/edwardkim/rhwp/issues/3820)에서 계속 다룬다.

또한 저장과 렌더의 경계를 분리한다. 저장은 이제 `0x00000000`을 실제 검정 음영으로 보존하지만,
현재 렌더러는 HWP5의 오래된 기본값 `0`과 의도된 검정 음영을 모델에서 구분할 수 없어 이를
보수적으로 음영 없음으로 취급한다. 실제 100% 검정 음영 표본을 얻은 뒤 `ColorRef`의 표현을
확장하는 것은 이번 검정 막대 수정 범위를 넘으므로 #3820 후속 fidelity 항목이다.

## 권고

HWP3 음영 결함과 HWPX의 실제 검정 음영 보존 결함은 모두 로컬에서 닫혔다. 메인터너 보정이
포함된 최신 head를 원 PR에 반영한 뒤 required checks를 다시 확인하는 것을 조건으로 **merge 권고**한다.
현재 단계에서는 GitHub push, comment, merge를 수행하지 않았다.
