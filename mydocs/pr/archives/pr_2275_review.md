# PR #2275 검토 - PROTECTED_SKIP 분리

- 검토일: 2026-07-15
- 작성자: planet6897
- 대상: [PR #2275](https://github.com/edwardkim/rhwp/pull/2275), [Issue #2261](https://github.com/edwardkim/rhwp/issues/2261)
- 메타: `devel` 대상, 1파일 `+20/-3`, contributor head `621b2e12cb5770f012b22c7d7f5b721df84018dd`
- 검토 범위: PR 본문·대화, 최신 head 소스 diff, 도구 구문 검사와 원격 CI를 함께 검토했다.
- reviewer: `jangster77` 지정 완료. 최신 head의 원격 CI는 통과 상태다.

## 본문·대화 검토

본문은 캐럿 진입이 차단된 HWP5 문서는 per-PI 비교 자체가 불가능하므로 일반적인
`PARA_COUNT`와 분리해야 한다는 병인을 명확히 한다. `PROTECTED_SKIP`에서 페이지 수만
대조하고 `page_match`와 `page_delta`를 남기며, [Issue #2152](https://github.com/edwardkim/rhwp/issues/2152)의 각주·미주 계열과 별개라는 경계도 적절히 구분한다.

본문의 재현 예시는 rhwp 19쪽, Hancom 20쪽으로 실제 페이지 차이를 보인다. 그럼에도 이
분류를 실패 집계에서 제외하는 것이므로, 이를 일반 `MATCH`나 clean gate로 오해하지 않게
별도 카운터와 각 문서의 `page_delta`가 최종 요약·CI artifact에서 계속 보이는 것이 중요하다.
PR 대화에는 추가 반론이나 범위 변경이 없다.

## 판단

**merge 수용 가능.** 구현은 보호 문서를 `PROTECTED_SKIP`으로 TSV에 개별 기록하고
`page_match`·`page_delta`를 남긴 뒤 일반 mismatch 실패 집계에서는 제외한다. 즉 page delta를
해결하거나 `MATCH`로 분류하는 변경이 아니며, `python3 -m py_compile`과 최신 원격 CI도
통과했다. 다만 최종 요약에는 보호 문서 중 page delta가 있는 문서 수를 별도로 집계하는
후속 보완이 필요하며, 이 분류를 근거로 [Issue #2261](https://github.com/edwardkim/rhwp/issues/2261)을 close하지 않는다.

## 체리픽 누적 검토 기록

- 순서: 2/3
- 적용 커밋: `621b2e12cb5770f012b22c7d7f5b721df84018dd`
- 누적 브랜치 커밋: `8edde1bdf`
- 충돌: 없음
- 선행 의존: 없음
