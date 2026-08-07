---
kind: review-implementation
status: completed-local
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-06
---

# PR #4061 메인터너 보정 기록

## 대상과 경계

- 대상 PR: [#4061](https://github.com/edwardkim/rhwp/pull/4061)
- contributor 원 head: `d03425e893bf81caa78e800a4070acea97e2ba9c`
- 가시성 검토 브랜치: `review/johndoekim-4061-20260806`
- 메인터너 보정 commit: `97c8d9732`

`maintainerCanModify=true`를 확인했다. contributor 원 commit은 rewrite·amend·reset하지 않았고,
동일 가시성 브랜치 위에 보정 code/test commit을 별도로 추가했다.

## 완료한 보정

1. ignore 생성기로 HWPX 4개, HWP 4개, 대조군 2개를 재생성했다.
2. 10개 입력을 HWP 2020 MCP로 PDF 변환했다. 전 건의 성공 상태·1페이지 A4·PDF 보존 경로와
   144 DPI 대표 렌더 해시를 기록했다.
3. Stage 2·Stage 4·계획서·최종 보고서를 현행 10개 코퍼스와 동일한 판정으로 정정했다. CFB
   재포장은 스트림 바이트만 보존하며 OLE 루트 CLSID까지는 무손실이 아니라는 범위를 명시했다.
4. 1,229줄 프로브를 828줄 본문과 417줄 공통 helper로 분리했다. 공통 helper는 CFB traversal/rebuild,
   CLSID 보존, 변종 변경, 코퍼스 검증을 담당한다.
5. focused test, ignore 생성기, fmt, clippy, diff 검사를 현재 보정 head에서 다시 실행해 통과했다.

## 검증 결과

| 항목 | 결과 |
| --- | --- |
| HWP 2020 MCP | 10 / 10 성공, `status=success`, `run_status=0`, `validation=ok`, PDF 1쪽 A4 |
| PDF 화면 판정 | OOXML 변경 6개와 대조군·레거시 단독 변경 4개가 서로 다른 두 렌더 해시 그룹으로 분리 |
| focused 프로브 | 9 passed, 1 ignored |
| ignore 생성기 | 1 passed |
| `cargo fmt --all --check` | 통과 |
| focused clippy `-D warnings` | 통과 |
| `git diff --check` | 통과 |
| 최신 `devel` 병합 simulation | 충돌·공백 오류 없이 통과, 검토 branch 정리 완료 |

## 원격 반영 전 조건

보정 code/test commit이 포함되므로 review-only fast-pass 대상이 아니다. 작업지시자의 push 승인 후에는
원격 contributor head, `git ls-remote`, local branch 시작 source SHA의 일치 및 LFS 대상 여부를 다시
판독한다. 그 뒤 code/test commit과 review·오늘 기록 commit을 순서대로 push하고, 최신 head 전체 CI와
mergeable 상태를 확인한다.
