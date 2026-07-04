# PR #1669 처리 계획 기록

## 커밋 구성

원 코드/작업 커밋:

| SHA | 제목 |
|-----|------|
| `45401d8f32485338d8b293a405723ad93a960ae9` | task 1634: 표 셀 전치 복사 붙여넣기 1차 구현 |
| `13bb02a5d8fedeeb0c6744b5b6375fb30905cceb` | task 1634: 표 밖 전치 붙여넣기 보완 |
| `a7049538fb252abe1733342eb2d431db4ad2df5b` | task 1634: 셀 선택 상태 전치 붙여넣기 분기 수정 |
| `b25d571e9795237176e2ef757cb5ab45149cbbb0` | task 1634: 선택 표 자체 전치 지원 |
| `ce3a385681dd14574397921bd94f4a7e0ba997de` | task 1634: 부분 선택 전치 붙여넣기 표 내부 우선 |

문서 후속 커밋:

| 항목 | 내용 |
|-----|------|
| review 문서 | `mydocs/pr/archives/pr_1669_review.md` |
| 처리 계획 | `mydocs/pr/archives/pr_1669_review_impl.md` |
| 오늘할일 | `mydocs/orders/20260629.md` |

## 처리 단계

1. 이슈 #1634 구현 가능성 검토
   - 사용자가 제시한 스프레드시트 `TRANSPOSE` 예시를 기준 동작으로 삼았다.
   - 표 전체 전치와 부분 선택 전치의 기대 동작을 사용자 검증으로 분리했다.
2. 코드 구현
   - 모델, `DocumentCore`, native/WASM API, Studio 명령/메뉴를 단계별로 연결했다.
   - 전체 표 선택은 기존 표 자체 전치, 부분 선택은 기존 표 내부 우선 붙여넣기로 보정했다.
3. 로컬 검증
   - Rust 빌드/테스트/clippy/doctest와 Studio 타입체크/테스트/빌드, WASM 빌드를 수행했다.
   - 렌더 영향 가능성을 고려해 `cargo test --test svg_snapshot`도 별도 확인했다.
4. PR 생성
   - head: `task/m100-1634-transpose`
   - base: `devel`
   - PR: #1669
5. self-merge 후보 문서 보강
   - collaborator self-merge 후보 예외 경로에 따라 review 문서 2건을 처음부터 `mydocs/pr/archives/`에 작성한다.
   - 오늘할일은 PR review 문서 묶음과 같은 문서 후속 커밋에 포함한다.
6. merge 전 최종 확인
   - PR head 최신 커밋 기준 GitHub Actions 통과 여부 확인
   - `mergeable` / `mergeStateStatus` 최신값 확인
   - 작업지시자 최종 승인 후 merge 판단

## 작업지시자 확인 사항

- PR 본문에는 `Closes #1634`를 포함했다.
- 현재 문서는 merge 완료 보고서가 아니라 self-merge 후보 준비 기록이다.
- PR 생성 직후 GitHub 참고 상태는 `MERGEABLE` / `BEHIND`였으며, merge 전 최신 상태 재확인이 필요하다.
