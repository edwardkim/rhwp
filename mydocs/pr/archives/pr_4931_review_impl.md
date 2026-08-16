# PR #4931 통합·보정 실행 기록

## 목적

kevin9327의 Open non-draft PR 중 아직 `upstream/devel`에 포함되지 않은 #4919~#4924를 하나의 검토
브랜치에 누적하고, 발견한 출력 계약 불일치를 메인터너 보정으로 분리해 최종 PR #4931에 기록한다.

## 기준선과 제외

- 기준선: `upstream/devel@ae5f2a3455636aac8cb1a64fdbcd1b6fb5978076`
- 제외: draft/WIP #4885~#4888
- 이미 기준선 조상: #4925~#4927, #4929~#4930
- Open non-draft 대상 아님: #4928

## 누적 commit

| 순서 | commit | 내용 |
| --- | --- | --- |
| 1 | `462f8daf7` | #4919 service 공통 문서 열기·조회 축 |
| 2 | `1f0b7e879` | service 모듈 선언 형식 정렬 |
| 3 | `302c29861` | #4920 render backend 공통 trait 계층 |
| 4 | `a6cb98e1b` | #4921 문서 의미 diff 라이브러리 |
| 5 | `23f77f51e` | #4922 CAS 판정 exit 3 및 재계획 hint |
| 6 | `3d469ee78` | #4923 agent preflight lint 배선 |
| 7 | `aedb512ed` | #4924 실물 대형 문서 scale ladder 공표 |
| 8 | `210b3ee37` | SVG 출력 계약 메인터너 보정 |
| 9 | `4560ee432` | 검토 local worktree 정리 범위 명확화 |

## 메인터너 보정 근거

`SvgBackend`가 실제로는 clip을 평면화하고 다중 쪽 SVG를 지원하지 않는데 capability에는 이를 포함하고,
`finish`에서 복수 SVG root를 연결해 반환했다. 보정은 capability 선언과 실제 동작을 일치시키고, 두 번째
쪽 시작을 명시 오류로 거부했다. 이 변경은 contributor commit을 재작성하지 않고 후속 독립 commit으로
추가했다.

## 검증과 merge 이후 정리

1. `cargo fmt --check`, Python compile, clippy, 전체 `release-test` integration 회귀를 완료했다.
2. #4931 생성 뒤 이 review와 오늘할일을 trailing commit으로 추가한다.
3. 최신 trailing head의 fast-pass CI와 `MERGEABLE`/`CLEAN`을 확인한 뒤 승인된 self-merge를 수행한다.
4. merge SHA와 devel 반영을 확인하고, 이번 통합용 local worktree와 local branch를 clean 상태에서 제거한다.
   원격 head branch 삭제는 별도 승인과 원본 저장소 소유 확인이 있을 때만 수행한다.
