---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-22
---

# PR #7329 검토 기록 — HY헤드라인M·HY울릉도M 가운뎃점 폭

- 원 PR: [#7329](https://github.com/edwardkim/rhwp/pull/7329)
- 관련 이슈: [#7092](https://github.com/edwardkim/rhwp/issues/7092)
- 원 code head: `7fa2944432383ef30d70e8f43bb7ec1546bd2ce5`
- 통합 검토 branch: `review/planet-open-20260922`
- 적용 commit: `162cc53f3` (`cherry-pick -x`, 충돌 없음)

## 변경과 검증

실측 글리프가 있는 HY헤드라인M·HY울릉도M의 U+00B7을 결측 글리프로 오인해 0.3em으로 좁히지 않고,
글꼴 메트릭의 전각 전진폭을 사용한다. `휴먼명조`의 기존 좁은 반례도 움직이지 않도록 신규 두 검사가 잠근다.

- 입력·기준: 기존 추적 자료 `samples/mel-001.hwp` / `pdf/mel-001-hwp-2020.pdf`.
- Native·fresh WASM Visual Sweep 8쪽: 구조 flag 0건. Native pixel/ink 87.837%/24.548%,
  WASM 87.826%/24.580%; 글꼴 raster 차이는 overlay에서 확인 가능한 제한으로 남긴다.
- overlay: `mydocs/pr/assets/pr7329_review/native_overlay_008.png`,
  `mydocs/pr/assets/pr7329_review/wasm_overlay_008.png`.
- focused nextest의 두 신규 검사와 통합 후보 검사 8건은 통과했다.

## 최종 판정

**승인.** 새 face의 전각 계약과 기존 반례 모두를 검증했으며, PDF와 입력은 이미 저장소에 추적되어 있다.

## Merge 후 contributor PR comment 계획

통합 PR의 **실제 merge SHA와 최종 head CI URL**이 확정된 뒤 원 PR에 한국어 감사 코멘트를 남긴다.
HY헤드라인M·HY울릉도M의 U+00B7 전각 계약, `휴먼명조` 반례를 그대로 둔 범위, `mel-001` 8쪽의
Native·fresh WASM 검증을 설명하고 아래 overlay를 본문 이미지로 넣는다.

```text
![Native overlay](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr7329_review/native_overlay_008.png)
![WASM overlay](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr7329_review/wasm_overlay_008.png)
```

Native/WASM ink match가 각각 24.548%/24.580%인 글꼴 raster 한계와 구조 flag 0건을 같이 기록한다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.


본문은 UTF-8 파일로 만든 뒤 `gh pr comment --body-file`로 게시하고, 게시 후 한국어 본문·실제 merge SHA·CI URL·두 이미지 URL이 최종 head를 가리키는지 다시 확인한다.
