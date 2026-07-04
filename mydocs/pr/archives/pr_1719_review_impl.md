# PR #1719 처리 계획 — 통합 cherry-pick 반영

## 대상

- 원 PR: #1719
- 관련 이슈: #1718
- 작성자: @planet6897
- 통합 브랜치: `codex/pr1719-1721-bundle`
- 통합 대상 커밋:
  - `ec62608ca50cd180eeb1bff4cf705736d0835095`
  - `6adce3e79aee7941e94e41665a87e8c493017de1`
- 제외 대상:
  - `1b2e46cab73a01575f26e362abe0b9786932139d` (`devel` merge commit)
  - `06a898913828f55a5d84f57a37ad03cf735cfb5f` (`devel` merge commit)

## Stage 1 — Cherry-pick

`upstream/devel` 최신 `add03dc5f64ce59c01122b513ade8a4e97c4977c` 기준 새 브랜치를 만들고
실제 변경 커밋만 순서대로 cherry-pick 한다.

결과:

- 충돌 없음
- 적용 후 커밋:
  - `b40d535dd` — #1718 1차 수정
  - `93904af24` — #1718 page13 회귀 보정

## Stage 2 — 로컬 검증

검증 전 `/Users/tsjang/rhwp/target` 하위 항목을 삭제한다.

완료한 검증:

- release-test `rhwp` 빌드
- #1718 샘플 `dump-pages` 42쪽 확인
- `row_cut_tests` 단위 테스트
- `issue_rowbreak_chart_overlap` 통합 테스트
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`

## Stage 3 — 통합 PR

#1721 커밋과 함께 하나의 통합 PR 로 제출한다.

PR 본문에는 다음을 명시한다.

- #1719/#1721 원 PR을 cherry-pick 형태로 묶었다는 점
- 원 PR의 merge commit 은 제외했다는 점
- 로컬 검증 결과
- 통합 PR merge 후 #1718 close 확인 및 원 PR #1719 supersede close 예정

## Stage 4 — merge 후 후속 처리

통합 PR이 merge 되면 다음을 수행한다.

- #1718 close 여부 확인, 필요 시 수동 close
- 원 PR #1719에 통합 반영 코멘트 작성 후 close
- 오늘할일 문서의 상태를 merge 완료로 갱신
- 통합 PR 브랜치와 `local/pr1719` 정리

결과:

- #1727 merge 완료: `150ca316ee557d6bf95928302166e037d7467b03`
- #1718 수동 close 완료
- #1719 supersede close 완료
- 5~6쪽 PDF 기준 잔여 시각 차이는 #1728로 후속 이슈 등록
- 후속 문서/샘플 보강 PR에는 현재 비교 샘플 4개를 함께 포함
