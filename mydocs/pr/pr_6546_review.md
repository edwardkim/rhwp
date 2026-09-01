# PR #6546 검토 - 문단 내부 저장 vpos 되감김

- 검토일: 2026-09-01
- 작성자: `planet6897`
- 원 PR/source head: `581740ccb1581f6cb9b17bf73ed00d49fd5e6647`
- 적용 commit: `a1648ea87`
- 통합 원장 보정: `cae16410d`
- 상태: 통합 candidate 수용 가능

## 판정

문단 내부의 쪽 규모 저장 `vpos` 되감김을 빌드 번호가 아니라 현재 line segment 상태로 탐지해 쪽
경계로 사용한다. #6542 focused fixture는 3줄이 footer/page number 영역으로 내려가던 회귀를 제거했고,
현재 p6 하단에는 겹침이 없으며 p7에서 본문이 이어진다.

Hancom 2022와의 직접 비교에서는 안전 결함은 해소됐지만 완전한 시각 일치는 아니다. rhwp p6에 한 줄이
더 남아 p7 시작점이 약 한 줄 빠르다. 이 잔여를 숨기지 않으며 이번 수용 범위는 footer 겹침과 잘못된
쪽 경계 제거다.

## 누적 보호 게이트

- #6542 focused test 통과
- #2070 315-page pin 통과
- oracle page-count, off-canvas, text-overlap, overflow-cell 각 16 partition 통과
- #6548과 함께 IR field sweep 1,061건 재계측, 3 skipped, 564 divergence paths
- 실제 dump와 baseline SHA-256 동일:
  `82ad76bdd63829c65836ba3b9466f497032e12cbca30d76a48339b8874c9c795`

`cae16410d`는 두 source PR의 baseline 행을 기계적으로 합친 것이 아니라 누적 head 전수 dump를
재산출해 사전순 원장으로 고정한 integration-only commit이다.
