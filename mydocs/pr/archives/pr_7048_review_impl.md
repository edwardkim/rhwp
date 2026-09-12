---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-12
---

# PR #7048 — 누적 체리픽 적용·후속 계획

## 렌더링 보류 사유 해결 완료 — 2026-09-12

- 최종 code candidate `c15686421dc6914c5bdf8cc0f80d8930bee46dc1`, branch `review/nondraft-7040-7053-20260912`.
- 원 source는 보존하고 그림 흐름·CellBreak 경계·바탕체 serif 대체를 메인터너 commit으로 보정했다.
- 본문/꼬리말 겹침 2→0, 하단 기준선 한컴과 0.04px 차이. 7쪽을 본문 대응 PDF와 직접 비교했다.
- 최종 판정 **메인터너 보정 후 수용 가능**. [검토·전체 검증·잔여 차이](pr_7048_review.md)를 따른다.
- 검증 입력 19개 모두 Git blob과 동일하다. 기존 원본·PDF 17개와 새 합성 HWPX 2개를 보존했다.
- `Summary [   0.705s] 42 tests run: 42 passed (1 leaky), 9537 skipped`, `Summary [ 329.734s] 9533 tests run: 9533 passed (3 slow, 1 leaky), 46 skipped`. Rust lint·Native Skia·WASM 및 8종 42쪽 parity 완료.
- GitHub 통합 PR 생성·최신 head CI·merge·원 PR/이슈 후속 처리는 아직 실행하지 않았다.
  원격 merge 전 조건과 comment/close 계획은 검토 문서에 기록했다.

<details>
<summary>이전 적용 단계와 검증 후보 이력</summary>


- [검토 결과](pr_7048_review.md), 원 source `429993db938324677f9d94fa8001600b6c51746b`.
- 작업 branch `review/nondraft-7040-7053-20260912`, base `ea5d1ff70b1d50301d1e6fdd26248e9d9c10c1fa`.
- code candidate `522a2e80db04cbd84264406ccb8bdd33a21dcc55`. 주 작업공간 `/Users/tsjang/rhwp`에서 순차 적용했다.

## 실제 적용

| 순서 | 원 PR·기능 SHA | 통합 SHA |
| --- | --- | --- |
| 1 | #7040 `6b675ac94c8bff70912dae3a3e61ffc62c49b0c6` | `9eccacadec143993d0b111b572f571daa67aa4d3` |
| 2 | #7048 `22bd8c76064f5e35e53aefb69b67e49058e2fc31` | `f615b85ca80da65ca14e30a83623d7a466799414` |
| 3 | #7048 `b0d657d750a8c05ad9e1a825207a983d79000f1b` | `b818519f62fd74cb6e477f34060af1e9f4b4d832` |
| 4 | #7050 `e28ba0dc7003bf298012f6620c08feba4250f0ed` | `3aa622052cd9078801d3f71340eab15bd045b5d0` |
| 5 | #7053 `5e83a52d55ac64e72f7e754371a3cefcea91836d` | `522a2e80db04cbd84264406ccb8bdd33a21dcc55` |

모든 기능 commit은 `git cherry-pick -x`로 저자·원 SHA를 보존했다.
기여 branch의 devel merge commit은 제외했다. 중복으로 생략한 기능 commit은 없고 충돌도 없었다.
#7040과 #7050은 중첩 표 영역의 상호작용을 함께 검토했다. 원 contributor source history는 변경하지 않았다.

## #7050 source 갱신 반영

- 종전 #7050 변경은 `d03f88cd3`에서 회수했다.
- 새 source `de3301a03891ff2a9287f907f6944a38cf59720b`를 `-x` 체리픽한 commit은
  `78f2a85b103a287c4def67221ff94b3b4e7ed298`다. 충돌은 없었다.
- 이전 5개 적용표는 최초 검증 후보의 이력이다. 현재 #7050은 새 source로 대체됐다.
- 최신 후보의 로컬 테스트/시각 실행은 미수행이며 source CI 34677890610의 최종 SUCCESS를 확인했다.

## 보정 전 단계 기록

1. 완료: non-draft 4건 분류, 원 PR reviewer 지정, head fetch, 최신 devel 위 기능 5개 적용.
2. 완료: 실제 diff·이슈/기존 리뷰·공통 조판 원칙 검토, 집중/전체 검증과 직접 증적 확인.
3. 완료: 원 PR별 review·이 계획·필요한 오늘할일·최종 증적을 같은 통합 branch에 기록.
4. 보류: #7040 줄 좌표/공통 메트릭, #7048 진단 위음성/실패 테스트, #7050 저장 줄 위치 축 보완.
   #7053 단독 변경 판정과 이 통합 branch 전체의 머지 판정은 구분한다.
5. 아직 수행하지 않음: blocker 해소 또는 #7053 분리 후보 검증 후 upstream 임시 branch의 devel 대상 통합 PR.
   owner를 reviewer로 자동 지정하지 않는다. code candidate CI 이후 필요한 문서 보완은 같은 PR의 trailing으로 처리한다.
6. 실제 merge 후: post_merge.md에 따라 exact merge/CI·devel 동기화·원 PR/이슈 comment/close·소유 자원 정리.
   원 PR보다 먼저 닫거나 contributor fork branch를 삭제하지 않는다.

기존 실패나 임의 수정으로 원 source를 강제 rewrite하지 않는다. source head가 바뀌면 새 SHA를 고정하고
변경 범위 검증을 다시 한다. 현재 검토 branch와 원격 조회 ref는 후속 보완을 위해 유지한다.
다른 작업의 worktree, 공유 target, 기존 사용자 파일은 보존했다.

## 메인터너 보정 완료와 남은 원격 단계

- `8099aa8b2`에서 원 source와 분리해 raw 컨트롤 축·줄 그룹·진단 replay 폭을 보정했다.
- `6456aff3a`에서 실제 셀 측정 경로를 타도록 회귀 입력을 보완했다. 최종 code candidate는 `6456aff3a0ee3fea70a12a7d167192e094e971fc`다.
- 집중 25 PASS, 전체 9,527 PASS / 46 skipped, Native Skia·WASM·7종 29쪽 parity를 완료했다.
- 직접 visual sweep 5장, 원본 5종 126쪽 before/after 불변, OVR5 142쪽/48개 객체 변화 0건.
  원본의 기존 한컴 차이·합성 입력과 정상 문서의 증거 구분은 [현재 검토](pr_7048_review.md)에 기록했다.
- `6b0c396b5`에서 음성 대조를 설명하는 테스트 주석만 정정하고 Rust lint 묶음을 재확인했다.
- 원격 integration PR 생성·CI·merge 및 원 PR/이슈 후속 처리는 아직 실행하지 않았다.
  review 기록을 별도 docs-only PR로 분리하지 않으며, 원 source history와 다른 worktree를 유지했다.


</details>
