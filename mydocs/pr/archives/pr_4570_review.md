---
kind: pr-review
status: local-validation-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4570 리뷰 - 자리차지 표 앵커 줄 재배치

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#4570](https://github.com/edwardkim/rhwp/pull/4570) |
| 작성자 | `planet6897` |
| base / 원 head | `devel` / `0d9d0d5acd6e46be1715d154450ed3142c917dc5` |
| 원 변경 규모 | 8 files, +450/-80 |
| 통합 적용 | `79311639c`부터 `76efeef1e`까지 9개 기능·golden commit |
| 관련 이슈 | [#4533](https://github.com/edwardkim/rhwp/issues/4533) |

비-TAC TopAndBottom 표의 앵커 줄만 저장 사다리 증거가 있을 때 밴드 아래로 옮긴다. 후속 문단과
밴드 자체를 움직이지 않고, HWP5/HWP3/HWPX 계보별 조건을 분리해 광범위한 vpos snap으로 확대되지 않게 했다.
golden `issue-157` 갱신도 같은 앵커 줄 좌표 변화에만 한정됐다.

통합 HEAD의 release-test 전체, Native Skia 3종, WASM build, Clippy와 focused provenance stage-1 6건을
통과했다. 렌더 영향 변경이므로 GitHub 통합 PR의 최신 head Full CI와 Render Diff를 merge 전 다시 확인한다.
릴리스 hold 동안 원 PR을 merge 또는 close하지 않는다.

## 2026-08-12 후속 head 검토

| 항목 | 검토 기록 |
| --- | --- |
| 최신 원 head | `00e685094d78651ce706fc6ce99efba0f9b2eb0a` |
| 기준 devel | `c6b43fbc69e2ec84bfc165f5a0eb2d192186b65d` |
| 새 통합 commit | `47b173cfb`, `796179850`, maintainer 보정 `68ea700c6` |
| 관련 이슈 | [#4533](https://github.com/edwardkim/rhwp/issues/4533), [#4654](https://github.com/edwardkim/rhwp/issues/4654) |

이전 통합 뒤 원 PR에 추가된 `d65e0fc`는 최신 `devel`에 동등한 내용이 있어 체리픽이 빈 변경으로
종료되어 건너뛰었다. `1af4beaa`의 Square 예약 공간 처리와 `00e68509`의 전면 그림 낱장 배치는
새로 적용했다. 후자의 원 조건은 정확히 절반인 그림도 과반으로 취급했으므로, 메인터너가
`68ea700c6`에서 `2 * fullpage_count > noninline_picture_count`의 **엄격한 과반**으로 좁히고
경계 회귀를 추가했다. 이 보정은 원 PR의 “과반” 의도와 일치하며 2/4 그림 문단의 뜻밖 낱장화를 막는다.

`fullpage_image_single_page_policy_requires_strict_majority` focused 검증, 전체 release-test nextest
`5,782 passed / 36 skipped`, Clippy, Native Skia 58+2+4, WASM build를 현재 통합 head에서 통과했다.
원 PR의 비공개 코퍼스 좌표 증적은 stage 기록에만 있어 공개 HWP 2020 PDF 대조를 새로 만들지 않았다.

**판정: 최신 통합 PR의 CI가 같은 head에서 성공하고 작업지시자가 승인하면 통합 PR로 수용한다. #4533과
#4654는 통합 PR merge 뒤 실제 해결 범위를 다시 확인할 때까지 닫지 않는다.**
