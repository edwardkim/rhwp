---
kind: pr-review
status: local-validation-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4572 리뷰 - ladder drift 위양성 가드

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#4572](https://github.com/edwardkim/rhwp/pull/4572) |
| 작성자 | `planet6897` |
| base / 원 head | `devel` / `8d1717742dcb3448de59555b1818d79050341e6d` |
| 원 변경 규모 | 1 file, +46/-3 |
| 통합 적용 | `ff292d3c3`, `001206cae` |
| 관련 이슈 | [#4533](https://github.com/edwardkim/rhwp/issues/4533) |

`기타` 컨테이너의 조상 인식, pi 순서 보존, 분절 첫 줄 우선 판정을 도입해 개체 내부 지역 좌표가
본문 사다리로 새어 드리프트가 되는 위양성을 줄인다. 실제 `sample1-repro.hwp` 실행은
`DRIFT pages=23, worst=62.7px, flagged=257`을 보고했으며, 이는 검출 도구의 결과이지 변환 실패가 아니다.

`python3 -m py_compile tools/verify_ladder_drift.py`, 도움말, `기타` nested-node synthetic guard가 통과했고,
통합 HEAD release-test 전체와 Clippy도 통과했다. #4575의 더 엄격한 노드 행 조건과 함께 적용했으며,
릴리스 hold 동안 원 PR close는 보류한다.

## 2026-08-12 후속 head 검토

| 항목 | 검토 기록 |
| --- | --- |
| 최신 원 head | `27b83dec002435d8445e010e123f386482ed6c89` |
| 기준 devel | `c6b43fbc69e2ec84bfc165f5a0eb2d192186b65d` |
| 새 통합 commit | `b7c2c3aaf` |
| 관련 이슈 | [#4533](https://github.com/edwardkim/rhwp/issues/4533) |

원 PR의 앞선 두 commit은 최신 `devel`에 patch-equivalent로 존재했다. 이번에는 `27b83dec`만 적용해
무명 `기타` 컨테이너를 조상 인식에서 투명하게 두지 않고, pi 순서를 유지하며, 다수 줄 분절은 앞부분
기준으로 귀속하게 했다. 엔진 좌표를 바꾸지 않는 검출 도구 변경이다.

`python3 -m py_compile tools/verify_ladder_drift.py`, 전체 release-test nextest
`5,782 passed / 36 skipped`, Clippy를 현재 통합 head에서 통과했다. 문서와 도구 출력의 판정은
드리프트 검출 결과이며 문서 fidelity 보증 자체가 아니다.

**판정: 최신 통합 PR CI 성공과 작업지시자 승인을 조건으로 통합 PR에서 수용한다. #4533은 캠페인 이슈이므로
이번 검출기 보정만으로 닫지 않는다.**
