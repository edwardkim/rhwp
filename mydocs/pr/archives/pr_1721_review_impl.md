# PR #1721 처리 계획 — 통합 cherry-pick 반영

## 대상

- 원 PR: #1721
- 관련 이슈: #1720
- 작성자: @planet6897
- 통합 브랜치: `codex/pr1719-1721-bundle`
- 통합 대상 커밋:
  - `ae76ea6f8b476609626009b75e36fe2dce6c5050`

## Stage 1 — Cherry-pick

`upstream/devel` 최신 `add03dc5f64ce59c01122b513ade8a4e97c4977c` 기준 통합 브랜치에
#1719 실제 변경 커밋 2건을 먼저 적용한 뒤 #1721 커밋을 cherry-pick 한다.

결과:

- 충돌 없음
- 적용 후 커밋:
  - `ec7dd96c9` — #1720 object_visual_regression.py 추가

## Stage 2 — 로컬 검증

완료한 검증:

- `python3 -m py_compile tools/object_visual_regression.py`
- `python3 tools/object_visual_regression.py --help`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`

제약:

- macOS 로컬 환경에서는 Windows 한글 COM/pyhwpx 대조 파이프라인을 실행하지 않는다.
- 통합 PR CI에서 Python CodeQL과 Build & Test 결과를 추가 확인한다.

## Stage 3 — 통합 PR

#1719와 함께 하나의 통합 PR 로 제출한다.

PR 본문에는 다음을 명시한다.

- #1719는 렌더 보정, #1721은 #1718 잔여 개체 배치 조사 인프라라는 관계
- #1721 단독 변경은 Python 도구/문서 중심이라는 점
- 로컬 Python 검증과 cargo 검증 결과
- 통합 PR merge 후 #1720 close 확인 및 원 PR #1721 supersede close 예정

## Stage 4 — merge 후 후속 처리

통합 PR이 merge 되면 다음을 수행한다.

- #1720 close 여부 확인, 필요 시 수동 close
- 원 PR #1721에 통합 반영 코멘트 작성 후 close
- 오늘할일 문서의 상태를 merge 완료로 갱신
- 통합 PR 브랜치와 `local/pr1721` 정리

결과:

- #1727 merge 완료: `150ca316ee557d6bf95928302166e037d7467b03`
- #1720 수동 close 완료
- #1721 supersede close 완료
- #1720 인프라로 추적할 잔여 RowBreak continuation 시각 차이는 #1728로 후속 이슈 등록
- 후속 문서/샘플 보강 PR에는 현재 비교 샘플 4개를 함께 포함
