# Task M100 #2428 최종 보고서 — 각주 hit-test fast-reject 종료 검증

## 결론

[#2428](https://github.com/edwardkim/rhwp/issues/2428)은 현재 유효한 미구현 작업이 아니다.
요구한 최적화는 [#2521](https://github.com/edwardkim/rhwp/pull/2521)을 통해 `devel`에 통합됐고,
2026-07-23 최신 `upstream/devel@29b5547e`의 HWP/HWPX production WASM 실제 클릭 경로에서
종료 조건을 모두 만족했다.

문서 PR이 merge된 뒤 검증 결과를 이슈에 남기고 수동 close한다. 추가 구현 이슈로 존치할 근거는
없다.

## 반영된 구현

- 구현: `0564f976c4c5d513aa52270d0408267e14bba682`
  - page-local pagination metadata의 `footnotes` 유무를 O(1)로 확인한다.
  - 각주가 없는 page에서는 Studio가 native `hitTestFootnote`를 호출하지 않는다.
- 회귀: `2c785d9bb3f65116e6fda81b4b12f89ede3e8e01`
  - 각주가 있는 첫 page, 없는 마지막 page, 범위 밖 page의 native/WASM 계약을 고정한다.
- 통합: PR #2521, merge commit `625e23a3d59ebe1002ef96a6d52a99c54e4b0f73`

## 종료 검증 요약

| 완료 조건 | 결과 | 판정 |
| --- | --- | --- |
| HWP/HWPX UI 114쪽 일반 본문 fast-reject | 포맷별 12회, native `hitTestFootnote` 0회 | PASS |
| 전체 클릭 지연 감소 | HWP p50 258.1→2.8ms, p95 268.6→9.225ms | PASS |
| 캐럿 정확성 | 24/24회 cell paragraph 2499, offset 77/78, page 113 | PASS |
| 표/각주 오진입 방지 | 24/24회 표 선택 false, 각주 모드 false | PASS |
| 실제 각주 마커와 영역 | HWP/HWPX 모두 진입·유지·본문 복귀 | PASS |
| native 회귀 | focused Rust test 1 passed | PASS |
| production 산출물 | WASM build와 Studio build 통과 | PASS |

상세 환경, 픽스처 해시, 좌표와 하위 호출 계측은
`mydocs/working/task_m100_2428_stage1.md`에 보존한다.

최초 검증 뒤 `devel`이 46커밋 전진해 PR 직전에 다시 rebase했다. #2428 코드·회귀·픽스처에는
직접 변경이 없었지만 render/layout과 각주 조사 변경의 간접 영향을 고려해 focused test,
production build, HWP/HWPX pointer matrix를 전부 재실행했다.

## 이슈가 닫히지 않은 이유

이슈가 열린 상태인 것은 구현 미완료가 아니라 통합 후 상태 정리 누락이다.

1. 원 기여 PR #2471은 `Issue: #2428`로 참조만 했고 closing keyword를 사용하지 않았다.
2. 원 PR은 직접 merge되지 않고 2026-07-20에 closed/unmerged 처리됐다.
3. 실제 코드는 누적 통합 PR #2521로 cherry-pick되어 `devel`에 merge됐다.
4. 저장소 기본 브랜치는 `main`이다. GitHub의 closing keyword 자동 종료는 기본 브랜치를 대상으로
   하므로 `devel` 통합에서는 자동 close에 의존할 수 없다.
5. #2521 merge 뒤 원 PR 정리는 수행됐지만 #2428의 구현 포함 여부 재검증과 수동 close가 후속으로
   실행되지 않았다.

따라서 #2428의 OPEN 상태는 기술적 잔여 작업을 의미하지 않는다. 원 PR의 비직접 통합과
`devel` 중심 운영에서 필요한 수동 종료 단계가 빠진 운영상 공백이다.

참고: [GitHub의 이슈-PR 연결 및 자동 종료 규칙](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/linking-a-pull-request-to-an-issue)

## 종료 순서

1. 이 보고서와 working 기록을 담은 문서 PR을 `devel`에 merge한다.
2. #2428에 구현 merge와 종료 검증 결과, 문서 PR 링크를 코멘트한다.
3. 코멘트 게시 후 이슈를 `completed`로 수동 close한다.

문서 PR 본문에는 `Related: #2428`을 사용한다. `Closes #2428`로 검증 문서 merge와 동시에
닫힌 것처럼 표현하지 않으며, merge된 증적 링크를 확보한 뒤 별도 상태 전환한다.

## 이슈 코멘트 초안

```markdown
종료 검증을 완료했습니다.

- 구현은 누적 통합 PR #2521을 통해 `devel`에 반영됐습니다.
  - 구현: `0564f976c4c5d513aa52270d0408267e14bba682`
  - 회귀: `2c785d9bb3f65116e6fda81b4b12f89ede3e8e01`
- production WASM에서 거대 표 HWP/HWPX를 각각 12회 실제 클릭했습니다.
  - native `hitTestFootnote`: 두 포맷 모두 0/12회
  - HWP 전체 handler: p50 258.1→2.8ms, p95 268.6→9.225ms
  - 24/24회 셀 문단 2499의 offset 77/78로 정확히 진입, 표 선택·각주 오진입 없음
- 실제 각주 HWP/HWPX에서 본문 marker 진입, 각주 영역 유지, 본문 복귀를 확인했습니다.
- focused Rust test, production WASM build, Studio production build가 통과했습니다.
- 최초 검증 뒤 46커밋 전진한 최신 `devel@29b5547e`로 rebase하고 같은 matrix를 재실행했습니다.
- 종료 검증 문서: PR #<검증-문서-PR>

이슈가 열린 채 남은 이유는 구현 미완료가 아닙니다. 원 PR #2471이 직접 merge되지 않고
#2521에 흡수됐고, 저장소 기본 브랜치가 `main`인 반면 통합은 `devel`로 이뤄져 자동 종료 대신
수동 상태 정리가 필요했으나 그 후속이 누락됐습니다.

완료 조건을 모두 만족하므로 이 코멘트와 함께 이슈를 닫습니다.
```

## 잔여 위험

- 이 측정은 한 macOS/arm64 headless Chrome 환경의 12회 표본이며 일반 성능 SLA를 정의하지 않는다.
- 이슈 본문에는 HWPX 변경 전 독립 baseline이 없어 HWPX 감소율은 산출하지 않았다. 다만 현재
  HWPX의 native 호출 생략, 절대 지연, 캐럿 정확성과 실제 각주 회귀를 직접 검증했으므로 종료를
  막지 않는다.
- 거대 표 마지막 상태 갱신에서 기존 `getCursorRectInCell` 오류가 `hitTest` cursor rect로
  fallback되는 경고가 포맷별 1회 있었지만 최종 캐럿 rect와 offset은 모두 정확했다. 이는
  `hitTestFootnote` 경로의 회귀가 아니며, 정리 시 #2400 계열의 별도 진단 대상으로 본다.
- 이후 `pageHasFootnoteFootholds` metadata와 render tree가 불일치하는 새 픽스처가 발견되면 새
  재현 이슈로 추적한다. 현재 #2428 범위에서 알려진 잔여 결함은 없다.
