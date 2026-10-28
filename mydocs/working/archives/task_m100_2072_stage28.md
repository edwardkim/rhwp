# Task M100 #2072 Stage 28 - 전체 변경 링크와 redirect 참조 보정

## 배경

최신 `upstream/devel` rebase에서 `mydocs/orders/20260328.md`가 `mydocs/orders/archives/20260328.md`로
이동했다. #2072가 갱신한 investigation 링크는 이동 전 상대 깊이를 유지해 기본 문서 CI 범위 밖에서
3건이 깨졌다.

이슈 본문과 maintainer 코멘트를 다시 대조한 결과, redirect stub 31개의 이전 경로를 소스 주석과 역사
문서가 문자열로 115건 참조하고 있었다. 기존 `--forbid-path`는 지정한 Markdown 링크만 검사하므로
"새 코드와 문서가 이전 경로를 다시 참조하지 못하게 한다"는 완료 조건을 자동 보장하지 못했다.

## 목표

- archive 위치를 기준으로 investigation 링크 3개를 바로잡는다.
- 기본 canonical 문서 검사뿐 아니라 #2072 전체 변경 Markdown을 다시 검사한다.
- redirect stub의 `canonical` 메타데이터에서 이전 경로를 동적으로 수집한다.
- PR base 이후 변경 파일만 추가 검사해 이전 경로 문자열과 상대 링크 재참조를 차단한다.
- 이동된 계획·보고서와 소스 주석의 이전 경로 115건을 현재 canonical 경로로 갱신한다.
- 활성 문서에 남은 개인 checkout 경로를 공개 원본 URL로 바꾼다.
- 최종 감사와 보고서의 링크 검증 근거를 실제 범위에 맞게 보완한다.

## 구현 결과

- `scripts/check_markdown_links.py`
  - `--changed-from <REF>`: merge-base 이후 변경 Markdown을 기본 검사 범위에 합친다.
  - `--forbid-redirect-references`: redirect stub을 동적 allowlist로 사용해 상대 링크와 저장소 경로
    문자열의 재참조를 거부한다.
  - `--changed-from` 사용 시 변경 파일만 검사하고, 수동 전수 진단에서만 전체 추적 파일을 검사한다.
- `.github/workflows/docs-link-check.yml`
  - `fetch-depth: 0`으로 PR base와 merge-base를 사용할 수 있게 했다.
  - PR/push의 base SHA가 있으면 변경분 게이트를 실행한다.
  - 새 redirect나 문서가 추가되어도 YAML 또는 별도 manifest를 수정하지 않는다.
- 경로 현행화
  - redirect 이전 경로 문자열 115건을 canonical 경로로 갱신했다.
  - `plans/archives`, `report/archives`, `working/archives` 재배치에서 깨진 링크 27건을 바로잡았다.
  - `orders/archives/20260328.md`의 investigation 링크 3건을 바로잡았다.
  - OWPML enum 참조의 개인 checkout 경로를 공식 GitHub 파일 URL로 교체했다.

## 검증 계획

- #2072 변경 Markdown 전체 상대 링크 검사
- 기본 링크·메타데이터·Python 구문·workflow·diff 검사
- 최신 `upstream/devel` 기준 제품 소스 변경 여부 확인

## 검증 결과

- `python3 scripts/check_markdown_links.py --changed-from upstream/devel
  --forbid-redirect-references`: 631개 Markdown, 변경 파일 635개, redirect 31개 통과
- `python3 scripts/check_markdown_links.py`: 383개 통과
- `python3 scripts/check_markdown_links.py --forbid-redirect-references`: 전체 추적 텍스트의 이전 경로
  문자열 재참조 0건
- 임시 변경 문서 음성 검사: redirect 상대 링크 1건과 이전 경로 문자열 1건을 각각 검출하고 종료 코드 1
- `python3 scripts/check_document_metadata.py`: 378개 통과
- `python3 -m py_compile scripts/check_markdown_links.py scripts/check_document_metadata.py`: 통과
- `actionlint .github/workflows/docs-link-check.yml`: 통과
