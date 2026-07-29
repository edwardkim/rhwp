# PR #2666 검토 기록

| 항목 | 내용 |
|---|---|
| PR | [#2666](https://github.com/edwardkim/rhwp/pull/2666) |
| 작성자 / base | [@planet6897](https://github.com/planet6897) / `devel` |
| 관련 | [#2430](https://github.com/edwardkim/rhwp/issues/2430), [#2559](https://github.com/edwardkim/rhwp/issues/2559) |
| 원 commit / 누적 적용 | `1aece55f` / `92bd93b29` (충돌 없음, 선행 [PR #2510](https://github.com/edwardkim/rhwp/pull/2510)·[PR #2627](https://github.com/edwardkim/rhwp/pull/2627)) |
| 범위 | 2026-07-20 r18 10k survey의 당시 측정 결과 기록 |
| 처리 경로 | collaborator 누적 통합 검토. merge 전 최신 원 PR 상태와 CI 재확인 필요 |

## 메인터너 보정과 검증

- 원 보고서가 현재 `devel` 결과처럼 읽히지 않도록 `HEAD=abae64173`의 역사적 측정 기록임을 명시했다.
  [PR #2627](https://github.com/edwardkim/rhwp/pull/2627)은 이후 [PR #2702](https://github.com/edwardkim/rhwp/pull/2702)로
  통합된 사실도 기록했다.
- 기여자 후속 코멘트에 맞춰 측정상 이탈 2건을 실제 각주 밴드 knife-edge 1건과 한글 캐럿 측정 아티팩트
  1건으로 구분했다. 원 수치 자체는 새 10k 실행으로 재주장하지 않는다.
- 문서 보정 뒤 `git diff --check`, 전체 release-test integration, clippy가 성공했다.

## 권고

보고서는 과거 검증 증거로는 보존 가치가 있으나 최신 `devel` 정량 지표가 아니다. 현재 수치로 일반화하지
않는 문구가 보정됐으므로 merge 가능하다. [#2430](https://github.com/edwardkim/rhwp/issues/2430)과
[#2559](https://github.com/edwardkim/rhwp/issues/2559)의 잔여 판별자/knife-edge 추적은 open으로 유지한다.
