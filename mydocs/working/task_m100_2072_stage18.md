# Task M100 #2072 Stage 18 - 일회성 이전 경로 검사 종료

## 목표

#2072 문서 이동의 완료를 확인하기 위한 이전 경로 64개 검사를 영구 CI 계약에서 제거한다. 자동 CI에는
현재 문서 구조의 링크와 메타데이터 무결성 검사만 남긴다.

## 판단

- `retired_markdown_paths.txt`는 문서가 아니라 #2072 migration 검증 입력이다.
- redirect 28개와 삭제 경로 36개의 재참조가 없다는 검증은 Stage 17까지 완료됐다.
- 같은 고정 목록으로 이후 모든 문서 변경에서 `mydocs` 전체를 반복 순회할 필요는 없다.
- 새 문서나 일반 문서 수정은 workflow의 `mydocs/**` trigger가 자동 감지한다.
- 문서 이동 stage에서는 기존 `--forbid-path`와 `--forbid-scan-path`를 일회성으로 사용할 수 있다.

## 변경 범위

- `mydocs/retired_markdown_paths.txt` 삭제
- workflow의 전체 이전 경로 검사 제거
- `--forbid-path-file` 옵션과 관련 문서 제거
- 문서 이동 시 일회성 `--forbid-path` 검사를 사용하는 규칙으로 복원

## 검증 계획

- workflow 실제 `run` 블록 실행
- Python 구문 검사와 `--forbid-path` 음성 검사
- 기본 링크 308개와 메타데이터 200개 검사
- canonical 문서와 workflow에 영구 manifest 참조가 없는지 확인
- `actionlint`와 `git diff --check`

## 결과

- workflow의 실제 `run` 블록은 기본 상대 링크 검사와 메타데이터 검사 두 명령만 포함한다.
- 상대 링크 308개와 메타데이터 200개가 통과했다.
- `--forbid-path` 음성 검사는 금지 링크 1건을 검출하고 종료 코드 1로 실패했다.
- 현재 권위 문서, 검사기, workflow에서 영구 manifest와 `--forbid-path-file` 참조가 제거됐다.
- Python 구문 검사, `actionlint`, `git diff --check`가 통과했다.

문서 관련 경로가 변경되지 않는 일반 코드 PR에서는 이 workflow가 실행되지 않는다. 문서 관련 변경에서도
#2072의 과거 migration 목록은 재검사하지 않는다.
