# PR #2331 리뷰 - #2072 문서 정보구조와 로컬 검증 체계 정리

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | [#2331](https://github.com/edwardkim/rhwp/pull/2331) |
| 제목 | `docs: #2072 문서 정보구조와 로컬 검증 체계 정리` |
| 작성자 | `jangster77` |
| base / head | `devel` / `codex/task2072-doc-ia-final-20260717` |
| 관련 이슈 | [#2072](https://github.com/edwardkim/rhwp/issues/2072) |
| 규모 | 문서 작성 시점 참고값: 639 files, +11,647/-7,096 |
| reviewer | `edwardkim` 지정 |
| merge 상태 | 문서 작성 시점 참고값: `MERGEABLE/BLOCKED` (최신 head CI 진행 중) |

검토 기준 구현 head는 `b98a84cb8680a0869084091ca46855a7fc892b34`다. 최종 merge 조건은 review
문서 추가 뒤 최신 PR head 기준 GitHub Actions 통과와 작업지시자 승인이다.

## 관련 이슈와 완료 조건

[Issue #2072](https://github.com/edwardkim/rhwp/issues/2072)는 `manual`과 `tech` 루트에 섞인 절차,
장기 기술 사실, 특정 이슈 조사와 역사 자료를 역할별로 분리하고 새 세션이 권위 문서를 찾을 수 있게 하는
정보구조 리팩토링이다. 이슈 본문과 후속 코멘트의 완료 조건을 다음과 같이 반영했다.

1. 이동 전에 링크 기준선과 저장소 내부 참조 검사를 추가했다.
2. `kind`와 `status`를 역할과 생명주기로 분리하고 `canonical`, `last_verified`를 독립 필드로 사용한다.
3. canonical 선정 과정에서 종료 상태, 오래된 명령과 중복된 운영 규칙의 현행성을 함께 감사했다.
4. 저장소 루트 `AGENTS.md`를 일반 경로 기반 부트로더로 추가하고 `CLAUDE.md`의 중복 절차를 축소했다.
5. 내부 참조는 새 경로로 갱신하고 외부 이력 호환이 필요한 31개 경로만 redirect stub으로 유지했다.
6. `tech/investigations`, `troubleshootings`, `archive`의 경계를 내용 기준으로 구분하고 비-Markdown reference
   자산의 canonical 위치를 문서 지도에 기록했다.
7. 일반 Markdown 추가마다 CI를 실행하지 않고, 이동·정보구조 변경 시 사용하는 로컬 링크·metadata 검사
   가이드를 남겼다.

## 변경 범위

- `mydocs/README.md`, `mydocs/manual/README.md`, `mydocs/tech/README.md`에 저장소 문서 지도와 canonical
  manifest를 추가했다.
- 시각 검증 상세 문서를 `mydocs/manual/verification/`으로 모으고 기존 외부 참조 경로는 제한 redirect로
  보존했다.
- 특정 이슈 조사 문서를 `mydocs/tech/investigations/issue-*`로, 대체된 문서를 `archive/`로 이동했다.
- 장기 문서에 `kind`, `status`, `canonical`, `last_verified` front matter를 추가했다.
- `scripts/check_markdown_links.py`와 `scripts/check_document_metadata.py`에 동적 문서 발견, 변경분 검사,
  redirect 이전 경로 재참조 검사를 구현했다.
- `.github/workflows/docs-link-check.yml`은 제거하고
  `mydocs/manual/markdown_link_check_guide.md`에 필요한 경우의 로컬 실행법만 남겼다.
- 제품 소스 2곳과 `tools/verify_pi_page_vs_hangul.py`는 이동한 문서 경로를 가리키는 주석·설명만 갱신했다.
  제품 런타임 동작과 테스트 계약은 바꾸지 않는다.

## 렌더 영향과 Visual Sweep

renderer/layout 제품 로직, WASM, Studio 화면 출력, golden과 기준 PDF를 변경하지 않는다. 제품 소스 diff는
문서 경로 주석 2곳뿐이므로 visual sweep 대상이 아니다.

## 검증

- `python3 scripts/check_markdown_links.py`
  - 384개 문서의 내부 Markdown 상대 링크 통과
- `python3 scripts/check_markdown_links.py --changed-from upstream/devel --forbid-redirect-references`
  - 변경 Markdown 636개, 변경 파일 639개, redirect stub 31개 통과
  - broken link 및 redirect 이전 경로 재참조 0건
- `python3 scripts/check_document_metadata.py`
  - 장기 문서 379개의 역할·생명주기·canonical 경로 통과
- `python3 -m py_compile scripts/check_markdown_links.py scripts/check_document_metadata.py`: 통과
- `git diff --check`: 통과
- 최신 `upstream/devel` 51개 커밋 병합: 충돌 없이 완료
- Cargo 빌드·테스트: 제품 동작 변경이 없는 문서 구조와 로컬 검사 도구 변경이므로 수행하지 않음

GitHub Actions는 문서 작성 시점에 CI/CodeQL/Render Diff preflight가 성공했고 전체 job은 진행 중이다.
이 상태는 최종 판정값이 아니며, review 문서가 추가된 최신 head에서 다시 확인한다.

## Findings

blocking finding은 없다.

639개 파일을 다루는 대형 이동 PR이라 GitHub의 파일 단위 diff만으로는 rename과 내용 변경을 함께 읽기
어렵다. 다만 작업을 30개 Stage와 기능별 커밋으로 분리했고, 변경분 전수 링크 검사, metadata 검사,
redirect 재참조 금지로 주요 이동 회귀를 자동 대조했다. 제품 동작 변경은 포함하지 않아 렌더·편집 회귀
위험과 분리된다.

## 운영 기록과 최종 권고

collaborator self PR 옵션 1에 따라 오늘할일은 PR 생성 전에 `mydocs/orders/20260717.md`에 포함했고, 이
review 문서는 PR 번호 발급 후 같은 PR head에 추가한다. 시각 검증 대상이 아니므로 별도 asset은 없다.

**Accept / merge 권고.** [Issue #2072](https://github.com/edwardkim/rhwp/issues/2072)의 문서 지도,
canonical 분리, 조사·보관 경계, 제한 redirect, 저장소 부트스트랩과 필요 시 실행하는 로컬 검사를 모두
충족한다. review 문서 추가 뒤 최신 PR head의 GitHub Actions가 통과하고 작업지시자가 승인하면 merge한다.
merge 후 #2072 자동 close를 확인하고 PR·이슈 후속 기록과 로컬·원격 작업 브랜치 정리를 수행한다.
