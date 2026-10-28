# Task M100 #2072 Stage 8 - historical roadmap archive 분리

## 목표

현재 개발 계획과 대체된 이전 roadmap을 같은 `mydocs/tech/` 루트에 두지 않는다.
이미 historical로 표시된 v1 backup을 `tech/archive/`로 옮기고, 현재 계획의 진입점은
`dev_roadmap.md`로 명확히 한다.

## 이동 대상과 근거

- `dev_roadmap_v1_backup.md`
  - 문서가 2026-02-10 기준이며, 현재 `dev_roadmap.md`는 2026-03-24 기준이다.
  - canonical manifest도 기존 문서를 `snapshot`·`historical`로 명시한다.
  - GitHub issue/PR 파일명 검색에서 외부 이력 참조는 0건이다.

따라서 redirect stub 없이 `mydocs/tech/archive/dev_roadmap_v1_backup.md`로 이동한다. 저장소 내부의
참조는 새 경로로 직접 갱신하고, CI의 이전 경로 금지 검사로 재유입을 막는다.

## 제외 범위

- `dev_roadmap.md`의 기술 사실 현행화
- #1860, #1904, #2125 등의 task 문서 분류
- archive 문서 본문의 과거 계획 수정

## 검증 계획

- 기본 Markdown 링크 검사
- Stage 4~8의 이전 경로를 지정한 `--forbid-path` 검사
- Documentation Link Check YAML 문법과 `git diff --check`

## 결과

- 2026-02-10 기준의 `dev_roadmap_v1_backup.md`를 `tech/archive/`로 이동하고 historical 문서
  진입점을 추가했다.
- canonical manifest와 tech 문서 지도는 현재 `dev_roadmap.md`를 우선하고 archive를 보존용으로
  안내한다. 외부 이력 참조가 없어 이전 경로 redirect stub은 만들지 않았다.
- 기본 링크 검사와 CI workflow에서 추출한 Stage 4~8의 이전 경로 금지 인수 검사가 각각 275개 문서에서
  통과했다. YAML 파싱과 `git diff --check`도 통과했다.

## 다음 단계

#1860, #1904, #2125와 #1472 문서는 이슈별 snapshot·조사인지, 현재 장기 canonical 후보인지 별도
현행성 감사 후 분류한다.
