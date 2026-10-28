# Task M100 #2072 Stage 11 - 이전 경로 재유입 검사 범위 분리

## 목표

기본 상대 링크 검사의 clean 기준선은 루트 안내 문서와 `mydocs/manual`·`mydocs/tech`로 유지하면서,
이동 전 경로의 신규 참조는 `mydocs` 전체에서 거부한다.

## 배경

현재 `--forbid-path` 검사는 기본 상대 링크 검사와 같은 276개 문서만 순회한다. 따라서
`mydocs/plans`, `mydocs/report`, `mydocs/working`, `mydocs/pr` 같은 이력 문서에 이전 경로가 다시
추가되어도 Documentation Link Check가 탐지하지 못한다.

`mydocs` 전체에는 이 작업 이전부터 존재한 깨진 상대 링크가 있어 기본 링크 검사 범위를 단순히 전체로
확대할 수 없다. 링크 존재 검사와 금지 경로 참조 검사의 문서 집합을 분리한다.

## 변경 계획

- `scripts/check_markdown_links.py`에 반복 가능한 `--forbid-scan-path` 옵션을 추가한다.
- 옵션이 없으면 기존과 같이 기본 검사 문서에서 금지 경로를 찾는다.
- 옵션이 있으면 상대 링크 존재 검사는 기존 범위에서 수행하고, 금지 경로만 지정 범위에서 찾는다.
- CI는 `--forbid-scan-path mydocs`를 사용해 이전 경로 40개의 재유입을 모든 저장소 문서에서 거부한다.

## 검증 계획

- `python3 -m py_compile scripts/check_markdown_links.py`
- 기본 링크 검사
- CI workflow에서 추출한 40개 이전 경로와 `--forbid-scan-path mydocs` 검사
- 임시 금지 경로 참조를 입력으로 사용한 실패 동작 단위 확인
- `actionlint .github/workflows/docs-link-check.yml`
- `git diff --check`

## 결과

- 기본 상대 링크 검사는 기존과 동일하게 276개 문서에서 통과했다.
- CI workflow의 이전 경로 40개를 `--forbid-scan-path mydocs`와 함께 실행해 7,068개 문서에서
  금지된 이전 경로 참조가 없음을 확인했다.
- `mydocs/README.md` 한 파일을 대상으로 실제 참조 중인 경로를 금지했을 때 1건을 보고하고 종료 코드 1을
  반환해 실패 동작을 확인했다.
- `python3 -m py_compile`, `actionlint`, `git diff --check`가 통과했다.
