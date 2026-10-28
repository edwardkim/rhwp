---
kind: implementation
status: completed
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-08-13
---

# Task M100 #3820 Stage 170 - saved object flow anchor와 paint inset

## 회귀

이월된 Stage 168의 엄격한 saved-object fit predicate를 완료하면서
`issue_3820_body_top_table_border_clip`에 실제 회귀가 드러났다.

```text
p168 table_y=698.24, expected=86.93, body clip_y=83.16
```

p33의 paint-only top-frame 사례는 계속 통과했다. 영향받은 successor fragment는
`samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`의
`pi=1775`였다.

`RHWP_DIAG_SCAN=1` 기록은 다음과 같다.

```text
pi=1775 cur_h=607.5 declared=314.2 avail=956.2
saved=(611.3, 921.7) bottom_fits=false
```

3.8px 차이는 임의의 drift가 아니다. p168/p214 회귀 계약이 보존하는 stored `283 HU`의
물리적 top paint inset과 정확히 일치한다. Stage 168은 이 물리 object top을 flow cursor와
비교했으므로, source-owned continuation이 현재 페이지 꼬리에 잘못 출력되고 실제
continuation은 다음 페이지로 밀렸다.

## 보정

`saved_span`은 이제 세 값을 기록한다.

1. 이동하지 않은 LineSeg flow anchor
2. 양수 `vertical_offset` 이후의 물리 object top
3. 물리 object bottom

declared-fit 및 split predicate는 current flow cursor를 이동하지 않은 anchor와 비교하되,
물리 object bottom은 여전히 body boundary 안에 있어야 하거나 이를 넘어야 한다. 기존 native
HWP5 near-anchor 및 internal-reset resync 경로는 의도적으로 물리 top을 사용하는 동작을
유지한다.

이는 source에서 유도한 보정이며, 넓은 pixel tolerance를 되돌리지 않는다.

## 검증

- `cargo test --profile release-test --target-dir target/task-3820-stage168 --test issue_3820_body_top_table_border_clip -- --nocapture`
  - 2 passed, 0 failed (`p33`, `p168`, `p214` 계약 포함)
- `cargo test --profile release-test --target-dir target/task-3820-stage168 --test issue_4490_4491_anchor_flow -- --nocapture`
  - 2 passed, 0 failed

## 결과

Stage 168의 saved-object strictness는 이제 flow anchor와 paint-only stored inset을 구분한다.
p168 successor table은 다시 body top에서 시작하며, owner geometry를 이동하거나 outer top
stroke를 clipping하지 않는다.
