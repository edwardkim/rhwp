---
kind: pr-review
status: approved-pending-integration-ci
pr: 5608
author: planet6897
integration_pr: 5617
---

# PR #5608 검토: 90도 회전 그림의 프레임 크기를 보존한다

원본: [#5608](https://github.com/edwardkim/rhwp/pull/5608)  
통합: [#5617](https://github.com/edwardkim/rhwp/pull/5617)  
원본 head: `310c37b27aa4e652863d5bb492aa60242e8567ea`  
관련 issue: #5595

## 검토 결과

**승인, 통합 PR CI 대기.** 90도 회전 그림이 긴 변 기준 정사각형 프레임으로 축소되지 않도록 Common frame의 실제 폭·높이를 조판에 사용한다.

## 검증 근거

- 누적 Rust integration: 7,782 passed
- native-Skia lib: 58 passed
- `samples/issue5595_rotated_picture_topbottom.hwpx`를 native-Skia PNG로 내보내 단일 페이지 프레임이 잘리지 않고 출력되는 것을 확인했다.

## 병합 후 처리

통합 PR #5617 병합 뒤 원본 PR #5608과 issue #5595에 통합 및 검증 결과를 댓글로 남기고 각각 close한다.
