---
kind: guide
status: active
canonical: mydocs/manual/markdown_link_check_guide.md
last_verified: 2026-07-17
---

# 문서 링크와 메타데이터 로컬 검사 가이드

이 문서는 저장소 문서의 상대 링크, redirect 이전 경로와 front matter를 로컬에서 검사하는 방법을
설명한다. 이 검사는 GitHub Actions에서 자동 실행하지 않으며, 일반 Markdown 파일을 추가하거나
본문만 수정할 때마다 수행할 필요도 없다.

## 실행 시점

다음 작업에서만 필요에 따라 실행한다.

- 문서 파일이나 디렉터리를 이동할 때
- redirect stub을 추가·제거할 때
- canonical 문서 또는 문서 정보구조를 리팩토링할 때
- PR 준비나 리뷰에서 문서 링크 검증을 명시적으로 요구할 때

일반적인 문구 수정, 작업 기록 추가, 새 Markdown 한 파일 추가는 자동 검사 대상이 아니다. 해당 문서에
저장소 내부 링크가 많아 작성자가 확인할 필요가 있을 때만 파일 단위 검사를 선택한다.

## 기본 링크 검사

루트 안내 문서와 `mydocs/manual`, `mydocs/tech`, `mydocs/troubleshootings`의 장기 문서를 검사한다.

```bash
python3 scripts/check_markdown_links.py
```

특정 파일이나 디렉터리만 검사할 수도 있다.

```bash
python3 scripts/check_markdown_links.py mydocs/manual/example.md
python3 scripts/check_markdown_links.py mydocs/tech/investigations/issue-1234
```

## 변경분과 redirect 검사

대규모 이동이나 정보구조 리팩토링에서는 기준 브랜치 이후 변경 Markdown과 변경 코드·문서의 redirect
이전 경로 재참조를 함께 검사한다.

```bash
python3 scripts/check_markdown_links.py \
  --changed-from upstream/devel \
  --forbid-redirect-references
```

`--forbid-redirect-references`는 `# 이동됨` redirect stub의 `canonical` 메타데이터에서 이전 경로를
동적으로 찾는다. 별도 migration 목록은 만들지 않는다.

단일 이전 경로를 진단할 때는 다음 옵션을 사용한다.

```bash
python3 scripts/check_markdown_links.py \
  --forbid-scan-path mydocs \
  --forbid-path mydocs/manual/<이전-경로>.md
```

## 메타데이터 검사

장기 문서의 `kind`, `status`, `canonical`, `last_verified`를 검사한다.

```bash
python3 scripts/check_document_metadata.py
```

## 검사 범위

- 저장소 내부 상대 Markdown 링크와 이미지 링크를 검사한다.
- 외부 URL의 응답 상태와 같은 문서 안의 anchor는 검사하지 않는다.
- 오류가 있으면 위치와 대상 경로를 출력하고 종료 코드 1을 반환한다.
- 자동 CI gate가 아니므로 실행 여부와 결과는 해당 작업의 stage 또는 PR 검증 기록에 명시한다.
