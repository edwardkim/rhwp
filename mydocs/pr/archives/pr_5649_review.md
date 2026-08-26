---
kind: pr-review
status: approved-pending-ci
pr: 5649
author: planet6897
base: devel
---

# PR #5649 검토: #5586 선언 높이를 넘는 중첩 내용 off-canvas 진단

PR: [#5649](https://github.com/edwardkim/rhwp/pull/5649)  
원본 head: `672ffa18ea545d5c63070c7fcedcd1687fcb79fc`  
통합 검토 브랜치: `integration/planet6897-20260819`  
통합 PR: [#5669](https://github.com/edwardkim/rhwp/pull/5669)

## 검토 결론

**승인, trailing 문서 head CI 대기.** 최신 `upstream/devel` 위에 번호순 체리픽했으며 충돌은 없었다. 구현 결함은 발견하지 못했다.

## 검토 근거

- 원 PR의 변경과 통합 후보 전체 diff를 검토했다.
- `nested_content_past_page_box_is_off_canvas`를 포함한 통합 후보의 전체 `release-test` 회귀는 7,869건 통과했다.
- lint, Native Skia, Canvas visual diff, CodeQL, archive와 regular·slow shard를 포함한 code candidate CI가 통과했다.
- source-side 테스트 증가 정책 위반 4건은 메인터너 보정으로 기존 외부 통합 회귀 target에 이관했다.

## 병합 조건과 후속 처리

이 trailing 문서 commit의 fast-pass CI와 mergeability를 확인한 뒤 #5669를 병합한다. 병합 후 원 PR #5649에 통합 사실과 검증 결과를 댓글로 남기고 close하며, 원격 head 브랜치를 정리한다.
