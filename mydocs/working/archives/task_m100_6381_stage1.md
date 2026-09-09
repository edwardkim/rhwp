# Task M100 #6381 Stage 1 완료보고 — 계약과 재현 고정

- **이슈**: [#6381](https://github.com/edwardkim/rhwp/issues/6381)
- **기준**: `upstream/devel@2bcf9b261c3b761d114bc2b3a35ed85ccd1e461e`
- **계획 commit**: `c0bc5c486`
- **상태**: 완료

## 1. 고정한 계약

`test-caption`의 고정 좌표 네 곳에 대해 다음 세 실행을 실제 CLI subprocess 계약으로 분리했다.

| 시나리오 | fixture | 기대 |
| --- | --- | --- |
| all-fail | 고정 좌표에 그림이 없는 기존 실문서 | panic 없음, exit 1, stderr 진단, SVG·`완료` 없음 |
| partial-fail | para 0 대상 둘과 para 1 대상 하나만 그림인 합성 HWP | exit 1, stderr 진단, SVG·`완료` 없음 |
| all-pass | 네 대상이 모두 그림인 합성 HWP | exit 0, `완료`, SVG 1개 이상 |

합성 HWP는 `HwpDocument::create_empty()`, `Paragraph::new_empty()`, `insert_picture_native()`,
`export_hwp()`의 공개 경로와 `assets/logo/logo-16.png`만 사용한다. repository에 binary fixture를 추가하지
않으며 테스트 종료 시 임시 파일을 제거한다.

## 2. 구현 전 red 재현

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --test regression_suite_004 \
  -E 'test(/issue_cli_test_caption_no_panic/)' --no-fail-fast
```

- nextest run: `1013663e-7963-4d0c-9280-0333ab007a61`
- 결과: 3건 중 1 pass, 2 fail
- all-pass는 기존에도 정상 통과했다.
- partial-fail은 마지막 setter가 범위 초과였지만 exit 0, SVG 1개, `완료`를 남겼다.
- all-fail은 네 setter가 모두 범위 초과였지만 exit 0, SVG 35개, `완료`를 남겼다.
- 두 실패 모두 panic은 아니었으며, #6381이 지목한 false-pass를 정확히 재현했다.

## 3. Stage 판정

세 fixture가 전부 실패하는 단순 부정 테스트가 아니라 all-pass 보호축을 함께 고정했으므로, 제품 보정이
성공 경로를 깨뜨리는 회귀도 잡을 수 있다. 작업지시자의 “신규 이슈 작업을 시작” 지시와 #6381 승인 기준에
따라 이 계약을 기준으로 Stage 2 구현을 진행했다.
