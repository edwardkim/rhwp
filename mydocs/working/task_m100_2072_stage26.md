# Task M100 #2072 Stage 26 - 전체 문서 메타데이터 게이트

## 목표

문서 역할과 생명주기 메타데이터 검사를 파일별 하드코딩 목록에서 디렉터리 자동 수집으로 전환한다.
새 문서가 추가될 때 Python 목록이나 GitHub Actions workflow를 수정하지 않아도 문서 위치 정책이
자동 검증되어야 한다.

## 변경 계획

- `mydocs/manual`, `mydocs/tech`, `mydocs/troubleshootings`의 모든 Markdown을 검사한다.
- redirect stub은 동일 검사 범위 안에서 `status: superseded` 규칙을 적용한다.
- 문서 지도에 실제 채택한 정보구조와 의도적으로 만들지 않은 빈 분류 디렉터리를 설명한다.
- 문서 추가 시 workflow YAML 수정이 필요하지 않음을 명시한다.

## 검증 계획

- 전체 메타데이터와 canonical 경로 검사
- 기본 및 변경 문서 Markdown 상대 링크 검사
- Python 구문, `actionlint`, `git diff --check`
- 제품 소스와 테스트 변경 여부 확인

## 변경 결과

### 동적 검사 범위

`scripts/check_document_metadata.py`의 파일별 `REQUIRED_PATHS`를 다음 디렉터리 단위 범위로 바꿨다.

- `mydocs/README.md`
- `mydocs/manual/`
- `mydocs/tech/`
- `mydocs/troubleshootings/`

따라서 이후 문서를 추가하거나 이동해도 Python 목록이나 `.github/workflows/docs-link-check.yml`을
수정할 필요가 없다. redirect stub도 같은 전체 범위에서 자동 검사한다.

### 검사 확대가 발견한 누락

기존 하드코딩 검사는 242개만 검사해 `manual/memory`의 72개 원문에 공통 분류 필드가 없는 사실을
놓치고 있었다. 기존 `name`, `description`, `type`, `originSessionId`는 보존하고 다음 필드를 추가했다.

- `kind: memory`
- `status: historical`
- `canonical: mydocs/manual/memory/MEMORY.md`
- `last_verified: 2026-07-17`

이는 과거 기록의 사실을 현재 규칙으로 재보증하지 않고 provenance로만 보존한다는 memory 지도와 같다.

### 실제 정보구조 기록

`mydocs/README.md`에 실제 채택한 `manual/verification`, `manual/codex`, `manual/memory`,
`tech/investigations`, `tech/archive`, `tech/webhwp` 경계를 기록했다. 이슈 초안에 있던 빈 분류 디렉터리는
미리 만들지 않았으며, 문서 지도와 메타데이터만으로 권위가 분명한 안정 경로는 유지한다.

## 검증 결과

- 전체 문서 메타데이터: 378개, 이상 없음
- 내부 Markdown 상대 링크: 383개, 이상 없음
- Python 검사 스크립트 구문: 이상 없음
- 문서 workflow `actionlint`: 이상 없음
- `git diff --check`: 이상 없음
- 제품 소스·테스트 변경: 없음

문서와 문서 검사만 변경했으므로 Cargo 빌드·테스트는 수행하지 않았다.
