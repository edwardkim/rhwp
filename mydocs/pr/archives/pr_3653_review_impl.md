---
kind: pr-review
status: active
---

# lpaiu-cs PR #3653·#3655·#3656·#3660 통합 검토·구현 기록

## 접수와 누적 범위

`upstream/devel` 위의 `review/lpaiu-cs-20260731`에 `@lpaiu-cs` open PR 4건의 contributor 기능
commit만 번호순으로 누적하고 [#3661](https://github.com/edwardkim/rhwp/pull/3661)을 만들었다. reviewer
`@jangster77`는 원 PR 네 건에 먼저 지정했다. 원 source branch의 devel merge commit은 통합하지 않았다.

| 원 PR | source head | 통합 commit | 판정 |
| --- | --- | --- | --- |
| #3653 | `c8f0e99bceb66790f4f7b6c83c328cb91983b96a` | `18631bce7` | 그림 byte-by-key API와 omission opt-in |
| #3655 | `de0bf178b746bb54f02843ab2109c085a7638db2` | `e2a3cd1ff` | cell/container undo meta 회귀 |
| #3656 | `9daeb0bb81ef9a7c2e3174a6ee8ab916c759a940` | `f0edd0889` | snapshot execute 실패 rollback |
| #3660 | `5991109d6efd50e46343dd1788e46e91f5ab572d` | `52903c91b` | narrow flow-image query 소비자 |

네 mapping은 `git patch-id --stable`에서 각각 동등하다. #3660 source는 #3653을 이미 포함한 stacked
branch이므로 #3653 기능을 중복 적용하지 않았고, 통합 branch에서 `18631bce7` 뒤 `52903c91b`을 한 번만
적용했다. base 병합이나 contributor source를 rewrite·force-push하지 않았다.

## 수용 계약

1. #3653의 기본 PageLayerTree JSON은 호환성을 위해 image bytes를 계속 포함한다. omission은 opt-in이고
   key lookup은 최종 방출 mime/bytes와 동등해야 한다.
2. #3660은 모든 flow image metadata를 정상 해석할 때만 DOM object-URL 경로를 쓴다. 불완전 응답은
   partial render가 아니라 기존 full-tree data-URL fallback이어야 한다.
3. #3655는 production change 없이 table·textbox·caption·nested by-path의 병합/undo 메타 경계를
   독립 회귀로 보장한다.
4. #3656은 history 등록 전 operation 또는 after snapshot 저장 실패 시 문서와 snapshot id를 before
   상태로 돌리고, rollback 실패도 원 오류와 함께 드러낸다.

## 검증 상태와 후속 순서

code head는 `52903c91bf132f7f3a977afc9cc265859b024c85`다. source #3653·#3655·#3656·#3660의 full CI와
#3661 exact code head의 lint/WASM, frontend gates, Native Skia, archive, default-feature 8 shards, CodeQL,
Canvas visual diff, `Build & Test` aggregate가 모두 성공했다. 현재 상태는 `CLEAN`·`MERGEABLE`이다.

통합 head의 headless Chrome Canvas2D에서는 `3-10월_교육_통합_2022.hwp` p0의 cacheable flow-image key를
bytes로 해석해 blob URL 그림 3장이 모두 decode됨을 확인했다. `field-01.hwp`로 문서를 교체하면 기존
DOM URL이 모두 revoke되고 cache가 0이 됐다. raw SVG가 있는 `143E433F503322BD33.hwp`는 기존 static
경로로 fallback했고, 추가 flow-image 표본 세 건은 blob DOM layer를 만들었다.
로컬은 전용 target에서 fresh `wasm-pack build`만 exit 0을 확인했고, 전체 Cargo는 exact CI와 중복하지
않도록 실행하지 않았다.

1. review 문서와 오늘할일만 commit·push한다. LFS 대상 여부를 staged file별로 판독하고 non-LFS면
   `GIT_LFS_SKIP_PUSH=1` dry-run 뒤 실제 push한다.
2. review-only head가 fast-pass인 것을 확인한다. 코드 검증을 fast-pass 결과로 바꾸지 않는다.
3. #3661을 squash merge하고 merge SHA·`upstream/devel` sync를 확인한다.
4. #3350·#3439의 close 상태를 실제 merge 뒤 확인한다. #3315는 umbrella로 open 유지한다.
5. 원 PR 4건에는 통합 PR·검증 범위·감사의 실제 줄바꿈 comment를 남기고 supersede close한다. source
   fork branch는 보존한다.
6. 실행 중 Cargo/Rust 작업이 없음을 확인한 뒤 `target/review-lpaiu-cs-20260731`만 recoverable하게
   정리하고 review branch/ref를 정리한다.
