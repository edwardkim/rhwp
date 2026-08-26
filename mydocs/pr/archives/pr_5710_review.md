---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-20
---

# PR #5710 검토 - 자리차지 표 밴드를 누락한 저장 사다리 보정

## 접수 메타데이터

| 항목 | 검토 기록 |
| --- | --- |
| PR / 작성자 | [#5710](https://github.com/edwardkim/rhwp/pull/5710) / `planet6897` |
| base / 원 PR head | `devel` / `6ea35cd9b27bdf1f47dc80a4af0fef0bf5a79eac` |
| 변경 규모 | 13 files, +278 / -5 |
| 통합 검토 branch | `review/planet6897-20260820` |
| local cherry-pick | `20f3a42af` |
| 통합 기준 | `upstream/devel@cfe2c351e` 위에 #5709 후 #5710 적용 |
| 관련 issue | #5699 H1 |

원 PR은 비 draft이며 작성 시점 확인에서 Full CI·CodeQL·Render Diff·Native Skia와 관련
필수 검사가 통과했다. mergeability와 check 상태는 외부에서 변할 수 있으므로 merge 직전
최신 head로 재확인해야 한다.

## 변경 범위와 검토 결과

`typeset_tac_table`에서 저장된 표 줄 높이의 흐름 계상이 선언 높이와 실측 높이 모두의 1/4
미만이고 두 높이가 정합할 때만 저장 사다리 누락으로 판정한다. 이 경우 실측 표 높이를
흐름에 반영하고, `PageContent::ladder_band_tables`와 `HeightCursor::min_flow_floor`를 통해
후속 문단의 저장 vpos 후방 스냅이 표 밴드 안으로 되돌아가지 않게 한다.

게이트도 함께 확인했다. HWPX는 제외하고, 직파싱 HWP3는 `treat_as_char` 조합만 대상으로 하며,
선언·실측 높이가 크게 발산하는 기존 문서 계열은 자동 제외한다. typeset에서 판정한 표 목록을
렌더러에 전달하므로 렌더 단계가 별도의 근사식으로 재판정하지 않는다.

검토 결과 판정 범위와 기존 문서 보호 게이트가 명확하고, 후속 흐름 바닥을 페이지·단 경계에서
리셋하는 경로도 포함되어 있다. 추가 메인터너 보정은 필요하지 않았다.

## 체리픽 및 충돌

- 최신 `upstream/devel@cfe2c351e` 기반 가시성 branch에 #5709를 먼저 적용했다.
- #5710 source head `6ea35cd9`를 `20f3a42af`로 누적 적용했다.
- 충돌은 없었으며 #5718은 이 변경 뒤에 같은 branch에 적용했다.

## 검증

- `node scripts/rust-test-suite-manifest.mjs --prepare` 및 `--check`: 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check upstream/devel...HEAD`: 통과
- 집중 테스트 `issue_5699_ladder_band_tripwire`: 2/2 통과
- 통합 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`: **8,001 통과, 38 skip**
- 전체 실행에서 H1의 영월군 페이지 수·표 아래 본문 위치 계약도 다시 통과했다.
- 원 PR의 샘플 기반 Canvas Render Diff와 Native Skia 결과는 통과했다. 통합 branch에서는
  동일 sample 회귀와 전체 Rust suite를 추가 확인했다.

## 판정

차단 결함과 추가 메인터너 보정 필요 사항은 발견하지 못했다. #5699 H1 범위는 통합 branch에서
수용 권고다. #5699의 H2/H3 후속 범위는 이 PR의 판정을 변경하지 않으며, 원격 작업은 수행하지
않았다. merge 전 최신 head·required check·관련 후속 issue 상태를 다시 확인해야 한다.
