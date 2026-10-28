# Task M100 #2072 Stage 14 - 문서 메타데이터와 현행성 감사

## 목표

문서 지도에만 있던 `kind`, `status`, `canonical`, `last_verified`를 실제 감사 대상 문서의 상단
front matter로 적용하고 CI에서 스키마와 canonical 경로를 검증한다.

## 필수 메타 범위

- `mydocs/README.md` canonical manifest와 `manual`·`tech` 문서 지도
- manifest에 등록된 canonical·guide·reference·decision 문서
- CLI canonical과 개별 명령 상세 문서
- `manual/verification/` 전체
- `tech/investigations/`, `tech/archive/`, `troubleshootings/` 전체
- `manual`·`tech`의 redirect stub

이 범위는 역할과 현행성을 이번 작업에서 직접 확인한 문서다. 아직 독립 감사를 하지 않은 legacy 상세
문서에는 날짜를 일괄 주입하지 않는다.

## 검사 규칙

- `kind`: `canonical`, `guide`, `reference`, `investigation`, `decision`, `snapshot`, `memory`
- `status`: `active`, `historical`, `superseded`
- `canonical`: 존재하는 저장소 상대 경로
- `last_verified`: `YYYY-MM-DD`

redirect는 `status: superseded`여야 하고, 조사 leaf는 가까운 이슈 README를 canonical로 둔다. historical
문서의 `last_verified`는 당시 기술 사실의 재검증이 아니라 snapshot 역할·경로·대체 관계를 확인한 날짜다.

## 검증 계획

- `python3 -m py_compile scripts/check_document_metadata.py`
- `python3 scripts/check_document_metadata.py`
- 기본 상대 링크 검사와 전체 이전 경로 금지 검사
- `actionlint .github/workflows/docs-link-check.yml`
- `git diff --check`

## 현행성 감사

- `cargo run --quiet --bin rhwp -- --help`의 일반 사용자 명령을 `cli_commands.md`와 대조했다.
- 누락된 `export-hml` 설명을 추가했고, 나머지 일반 명령이 canonical CLI 문서에 존재함을 확인했다.
- ThorVG 결정 기록에서 현재 존속을 보장할 수 없는 로컬 브랜치 의존 표현을 commit 이력 기준으로 고쳤다.
- SVG 회귀 예시는 특정 로컬 브랜치 이름 대신 `upstream/devel`과 `HEAD`를 사용하도록 현행화했다.
- historical investigation의 날짜는 기술 결론의 현재 유효성을 보증하지 않고 역할·이슈 범위·canonical
  관계를 확인한 날짜라는 경계를 manifest에 명시했다.
- 기존 troubleshooting leaf 61개는 현재 `devel` 재검증 전인 과거 해결 사례이므로 `historical`로
  분류했다. 인덱스와 이번 감사에서 직접 재검증한 PDF 번호 비결정성 문서만 `active`로 유지했다.

## 결과

- canonical·분류 클러스터·redirect 200개 문서에 실제 front matter를 적용했다.
- `check_document_metadata.py`가 필수 필드, 허용 값, canonical 경로 존재, 날짜 형식, redirect의
  `superseded` 상태를 검사하며 전체 대상에서 통과했다.
- 메타데이터가 없는 문서를 직접 검사한 negative check에서 4개 필드 누락을 모두 보고했다.
- 기본 링크 검사 308개와 전체 `mydocs` 7,104개의 이전 경로 금지 검사가 통과했다.
- `python3 -m py_compile`, `actionlint`, `git diff --check`가 통과했다.
