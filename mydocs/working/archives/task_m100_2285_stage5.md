# 단계별 기록 S5 - 철회된 handle-only 보정 (M100 #2285)

- **이슈**: [edwardkim/rhwp#2285](https://github.com/edwardkim/rhwp/issues/2285)
- **대상 PR**: [edwardkim/rhwp#2286](https://github.com/edwardkim/rhwp/pull/2286)
- **브랜치**: `codex/task2285-handle-only-20260716`
- **단계**: S5 / 철회됨
- **작성일**: 2026-07-16

## 당시 판단

이슈 본문의 handle-only 설명을 제품 요구로 단정해, URL/드롭/input 로드는 최근 문서에 남기지 않도록
보정했다. IAB의 새 격리 프로필에서 URL 로드 뒤 최근 문서가 비어 있는 것도 확인했다.

## 정정

위 판단은 실제 Chrome 프로필 동작을 확인하지 않은 오류였다. 제품 요구는 일반 문서를 열면 최근 문서에
한 건이 표시되는 것이다. URL/드롭/input 로드는 파일명·형식·시각만 가진 meta-only 항목으로 기록하고,
선택하면 파일 선택기로 다시 고르는 것이 맞다. IAB의 빈 목록은 격리된 새 프로필의 빈 IndexedDB 상태일
뿐 실제 Chrome 결과가 아니다.

`2d6884b`의 handle-only 보정은 S7에서 철회됐으며 최종 이력에는 포함하지 않는다. 실제 Chrome과
최신 build 검증은 [S7 기록](task_m100_2285_stage7.md)을 따른다.
