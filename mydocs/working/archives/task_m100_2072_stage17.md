# Task M100 #2072 Stage 17 - 이전 문서 경로 manifest 분리

## 목표

문서가 이동될 때마다 `.github/workflows/docs-link-check.yml`의 명령 인자를 수정해야 하는 결합을 제거한다.
문서 CI workflow는 고정하고, 폐기된 Markdown 경로는 별도 manifest에서 관리한다.

## 설계

- `mydocs/retired_markdown_paths.txt`에 이전 경로를 한 줄에 하나씩 기록한다.
- `check_markdown_links.py`에 반복 가능한 `--forbid-path-file` 옵션을 추가한다.
- workflow는 manifest 파일 하나만 전달한다.
- 새 문서 추가는 기존 `mydocs/**` trigger가 자동 감지한다.
- 문서 이동 시에는 YAML이 아니라 manifest에 이전 경로만 추가한다.
- 개별 진단용 `--forbid-path` 옵션은 유지한다.

## 검증 계획

- Python 구문 검사와 manifest 64개 파싱 확인
- redirect stub 28개가 manifest에 모두 포함되는지 집합 대조
- 기본 링크·메타데이터·전체 이전 경로 검사
- workflow의 실제 `run` 블록 실행
- `actionlint`와 `git diff --check`

## 결과

- manifest의 이전 경로 64개가 정렬·중복 없음 조건을 충족했다.
- redirect stub 28개가 manifest에 모두 포함됨을 확인했다.
- 금지된 링크를 포함한 입력은 종료 코드 1로 실패하고, manifest 파일 누락도 즉시 실패했다.
- workflow의 실제 `run` 블록에서 상대 링크 308개, 메타데이터 200개, 전체 `mydocs` 이전 경로
  검사 7,109개가 통과했다.
- Python 구문 검사, `actionlint`, `git diff --check`가 통과했다.

workflow는 더 이상 개별 문서 경로를 포함하지 않는다. 새 문서 추가에는 별도 설정 변경이 없고,
문서 이동·폐기 시 `mydocs/retired_markdown_paths.txt`만 갱신한다.
