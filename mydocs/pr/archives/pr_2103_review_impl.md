# PR #2103 처리 계획

## 1. 적용 경로

이 PR은 collaborator self-merge 후보 예외 경로의 옵션 1로 처리한다.

- 코드 PR head에 review 문서를 포함한다.
- E2E 대표 asset을 `mydocs/pr/assets/` 아래에 안정 파일명으로 보존한다.
- 오늘할일 문서를 같은 PR head에서 갱신한다.
- merge 전 최종 조건은 PR head 최신 커밋 기준 GitHub Actions 통과와 작업지시자 승인이다.

## 2. 커밋 구성

작성 시점 코드 커밋은 다음 6개다.

| SHA | 제목 |
|---|---|
| `671c4b4af` | task 2069: OLE 미리보기 선택 메타 보존 |
| `70f9fbea3` | task 2069: OLE 편집 표식과 속성 UI 보정 |
| `1dc2304b6` | task 2069: OLE 문단부호 흐름 보정 |
| `ea1524d89` | task 2069: OLE 내부 클릭 개체 선택 보정 |
| `1bfeff256` | task 2069: OLE 캡션 설정과 재진입 Enter 보정 |
| `200f64d85` | task 2069: OLE/그림 캡션 및 붙여넣기 정합화 |

옵션 1 문서/asset/오늘할일 커밋을 추가하면 PR head SHA는 갱신된다. 최종 merge 전에는 최신 head 기준 CI를
다시 확인한다.

## 3. 검증 체크리스트

- [x] focused Rust tests
- [x] `cargo test --profile release-test --tests`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `wasm-pack build --target web --out-dir pkg`
- [x] `rhwp-studio` production build
- [x] `rhwp-studio` issue 2069 E2E
- [x] `git diff --check`
- [ ] GitHub Actions latest head 확인
- [ ] 작업지시자 merge 승인

## 4. 후속 처리 계획

1. 옵션 1 문서/asset/오늘할일 커밋을 PR #2103 head에 push한다.
2. GitHub Actions 실행 상태를 확인한다.
3. CI 통과와 작업지시자 승인을 받은 뒤 merge한다.
4. merge 후 `devel`을 `upstream/devel`로 동기화한다.
5. #2069 auto-close 여부를 확인하고 필요하면 후속 코멘트를 남긴다.
6. PR head 브랜치와 로컬 작업 브랜치를 정리한다.

## 5. 코멘트 초안

PR merge 전 별도 코멘트는 남기지 않는다. merge 후 코멘트가 필요하면 다음 내용을 사용한다.

```markdown
검토 및 머지 완료했습니다.

확인한 내용은 다음과 같습니다.

- OLE RawSvg/control metadata 보존으로 한셀 OLE 개체 선택과 개체 속성 진입이 가능함을 확인했습니다.
- 빈 문단 + non-TAC OLE의 우측 캐럿 위치와 Enter/Backspace/Enter 재진입 흐름을 회귀 테스트와 E2E로 확인했습니다.
- OLE/그림 캡션 설정·제거·자동번호 재계산과 그림 선택 후 캐럿 이동 뒤 붙여넣기를 확인했습니다.
- 로컬 검증: `cargo test --profile release-test --tests`, `cargo clippy --all-targets -- -D warnings`, WASM build, studio build, issue 2069 E2E 통과.

대표 E2E 증적은 PR 기록 asset으로 보존했습니다.
```
