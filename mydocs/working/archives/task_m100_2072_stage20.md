# Task M100 #2072 Stage 20 - 문서 링크 CI 범위와 migration 설명 정합화

## 목표

`mydocs/**` 변경에서 실행되는 문서 CI가 새로 분류한 `troubleshootings`의 내부 링크도 실제로 검사하도록
범위를 넓힌다. Stage 18에서 영구 이전 경로 검사를 제거한 정책과 시각 검증 문서의 설명도 일치시킨다.

## 변경 범위

- 기본 링크 검사 대상에 `mydocs/troubleshootings` 추가
- 기존 historical troubleshooting에서 확인된 깨진 내부 링크 27건 정리
- 저장소 내부 자료는 현재 경로로 갱신
- 외부 `hwplib` 소스는 공식 upstream GitHub 파일로 연결
- 저장소에 없는 과거 로컬 분석 스크립트는 링크가 아닌 historical 경로로 표기
- 시각 검증 이전 경로는 CI 영구 금지가 아니라 이동 stage의 일회성 `--forbid-path` 검사임을 명시

## 검증 계획

- 기본 링크 검사에 `troubleshootings`가 포함되는지 확인
- 전체 변경 Markdown 링크 검사
- 메타데이터 검사
- workflow 실제 run 블록, Python 구문, `actionlint`, `git diff --check`

## 결과

- 기본 링크 검사 범위가 308개에서 374개로 늘었고 `mydocs/troubleshootings`가 포함됐다.
- 기존 historical troubleshooting의 깨진 내부 링크 27건을 정리한 뒤 기본 검사 오류가 0건이다.
- #2072 전체 변경 Markdown 319개를 별도로 검사해 내부 상대 링크 오류가 없음을 확인했다.
- `hwplib` 두 외부 소스 URL은 upstream GitHub API로 실제 파일 존재를 확인했다.
- 메타데이터 205개 검사, Python 구문 검사, `actionlint`, `git diff --check`가 통과했다.
- workflow의 run 블록은 기본 링크 검사와 메타데이터 검사만 유지하며 문서 추가마다 YAML을 수정하지 않는다.
