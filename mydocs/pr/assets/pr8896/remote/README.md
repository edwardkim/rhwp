# PR #8896 원격 수용·후속 기록

- [수용 PR #8984](https://github.com/edwardkim/rhwp/pull/8984), head `1343907a8340fc032423ce5adb6d248aa80534dd`, merge `f0a2e5379a42d2ff4d18f192634397dbce805833`. 원 [#8896](https://github.com/edwardkim/rhwp/pull/8896)도 간접 MERGED다. 원 fork branch는 보존했다.
- [self-review](https://github.com/edwardkim/rhwp/pull/8984#pullrequestreview-5867893783)와 [전체 CI·병합 영수증](ci-and-merge.json)을 확인했다. 제품 CI와 최종 정책 감사가 모두 성공했으며 재실행·예외·기준 완화는 없었다.
- CI가 checkout한 merge `3383a9f7308ce34acd8ca571f11f5f5f19e783fa`의 tree는 `1a567f6f762ab22d3fad4ca9a15ec0e45bf4cd08`로 후보와 같다. [Git Data API 근거](tested-merge-tree.json).
- [Archive A–E](archive-counts.json)는 예상 실행 수와 실제 13,132 PASS가 일치한다. 로컬 전체 13,326 PASS와 별도 계수다. 새 WMF 검사의 [CI 실행](wmf-test.json)도 확인했다.
- [PR 본문 원격 링크](body-links.json) 12개와 [실제 브라우저 이미지](body-display.json) 4개를 확인했다. 후속 댓글용 [전 6쪽·두 경로의 hash 연결](comment-image-bridge.json)과 [merge SHA raw PNG 24개](comment-assets.json)도 확인했다.
- 이슈 [#8693](https://github.com/edwardkim/rhwp/issues/8693)은 자동 종료(`2026-11-12T02:06:26Z`)를 확인했다. 운영 기록 반영·devel 동기화 뒤 같은 commit의 메인테이너 증적 댓글을 남긴다. [#7620](https://github.com/edwardkim/rhwp/issues/7620)은 OPEN을 유지하며 변경하지 않는다.

## duration 게시 확인 오류의 독립 대조

[메타데이터 workflow](https://github.com/edwardkim/rhwp/actions/runs/43139004399)는 성공한 PR의 B/C/D/E artifact를 검증해 `ready:true`로 수집했다. 게시 후 `duration-publish: published ref identity mismatch`로 **workflow 자체는 failure**다. 이 실패를 success로 바꾸거나 재실행하지 않았다.

실제 metrics ref는 [게시 commit 5ec85702](https://github.com/edwardkim/rhwp/commit/5ec85702fe11ba733da4af4b41fcdcf371a4bdfd)를 가리키며 메시지의 source merge도 위 merge와 같다. 원 artifact 4개를 다시 받아 같은 보정 source의 `refresh-nextest-target-duration-policy.mjs`로 계산했고, [게시 정책](published-duration-policy.json)의 44개 target과 바이트 단위로 같음을 확인했다. [확인 결과·blob SHA·SHA-256](duration-publication.json), [B](target-durations-b.json)·[C](target-durations-c.json)·[D](target-durations-d.json)·[E](target-durations-e.json). 이 재계산은 메타데이터만 처리했으며 제품 빌드·테스트나 원격 쓰기는 실행하지 않았다.

즉시 ref 조회의 지연 가능성은 추정이며, 당시 응답이 보존되지 않아 확정하지 않는다. 게시 뒤 확인 로직의 짧은 재조회·관측 보강은 GitHub 운영 후속 검토 항목이다. 이 PR의 제품 CI와 실제 duration 게시·내용 확인은 완료했다. 병합 뒤 제품 검증 CI는 실행하지 않았다.
