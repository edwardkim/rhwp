# PR #2331 메인테이너 검토 — 문서 정보구조와 로컬 검증 체계 정리 (#2072)

- PR: https://github.com/edwardkim/rhwp/pull/2331 (jangster77, collaborator)
- 이슈: #2072 (Closes) / 컨트리뷰터 self-review 동봉 (pr/archives/pr_2331_review.md)
- 규모: 638 파일 (+11,638/−7,096) — 신설 111, 수정 460, 이동 67(redirect stub 31)
- 현 devel(b4c436df, README 현행화 포함) 기준 **충돌 0** — base 가 최신

## 변경 본질

1. **CLAUDE.md 523→38줄 부트로더화** + `AGENTS.md` 신설 — 권위 문서 로딩
   순서 명시, 절차 중복 기록 금지. canonical 우선 원칙.
2. 문서 지도: `mydocs/README.md`(manifest) + `manual/README.md`/`tech/README.md`
   + `manual/verification/`·`tech/investigations/` 재배치, redirect stub 31개.
3. 로컬 검사 도구: `scripts/check_markdown_links.py` + 메타데이터 검사 —
   일반 문서 변경 시 CI 미실행 원칙(가이드 문서화).
4. 코드 접점 3파일은 문서 경로 주석 갱신뿐 (동작 무변경).

## 재실증 (merged tree = 최신 devel + head)

- 상대 링크 검사: **384개 통과** (금일 devel 문서 대량 변경 병합 후에도 green)
- CLAUDE.md 핵심 규칙 이관 추적: 하이퍼-워터폴/승인 게이트/브랜치 정책/
  release-test/pdf 권위/이슈 close → canonical 문서들에서 전부 확인
- `pr/` 처리 규칙 → `pr_review_workflow.md` canonical 로 커버 확인

## 발견 — 규칙 소실 3건 (Folder Roles, docs_and_git_workflow.md)

base 가 최신임에도 CLAUDE.md 전면 재작성 과정에서 폴더 역할 표의 일부가
새 canonical 에 반영되지 않음:

1. **`orders/archives/`** — 전월 이전 오늘할일 보관(매월 초 이동). 2026-07-17
   승인·시행된 규칙(133건 이동 완료 상태)인데 Folder Roles 에 부재
2. **`plans/archives/`** — 완료 계획서 보관 (시행 중 관례)
3. **`feedback/`** — 작업지시자 피드백 폴더 (hyper_waterfall.md 본문에는
   잔존하나 Folder Roles 표에 부재)

## 판단

**merge 권고 + 보완 커밋 선행.** 구조 개편 자체는 정합적이고 검사 도구까지
자기완결적. 소실 3건은 maintainer edit 로 Folder Roles 에 3행 보완 push 후
merge 하는 것이 효율적 (컨트리뷰터 왕복 불필요한 경미 결손).
