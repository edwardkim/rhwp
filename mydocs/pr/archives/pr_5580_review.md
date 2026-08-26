---
kind: pr-review
status: approved-pending-integration-ci
pr: 5580
author: planet6897
integration_pr: 5617
---

# PR #5580 검토: 문서 열기 흐름을 빈 문서·빈 쪽 기준으로 정리한다

원본: [#5580](https://github.com/edwardkim/rhwp/pull/5580)  
통합: [#5617](https://github.com/edwardkim/rhwp/pull/5617)  
원본 head: `b95bda5d84dafd1d69009fa29893c8c23ff2ed7c`

## 검토 결과

**승인, 통합 PR CI 대기.** PWA launch queue로 수신한 문서는 현재 문서를 교체할 수 있을 때 열리고, idle 상태의 빈 문서는 그 전에만 생성된다. launch queue 전달이 지연돼 빈 문서가 잠시 보일 수는 있으나, 수신 문서를 막거나 손실하지는 않는다.

## 검증 근거

- `npm --prefix rhwp-studio test`: 991 passed, 1 skipped
- `npm --prefix rhwp-studio run build`: 통과
- 누적 Rust integration: 7,782 passed

## 병합 후 처리

통합 PR #5617이 병합되고 최신 head CI가 성공한 뒤, 원본 PR #5580에 체리픽 통합 사실과 검토 결과를 댓글로 남기고 close한다. 별도 연결 issue는 원본 PR에서 지정되지 않았다.
