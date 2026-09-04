# Task M100 #2072 문서 정보구조 리팩토링 최종 보고서

## 결과

이슈 #2072 본문과 2026-07-11 maintainer 코멘트의 완료 조건을 최신 `upstream/devel` 기준으로 모두
충족했다. 문서 동작·권위·생명주기를 내용 기준으로 분류했으며 제품 동작은 변경하지 않았다.

## 최종 정보구조

- `mydocs/manual/README.md`: 반복 작업, 명령, 운영·검증 절차의 진입점
- `mydocs/tech/README.md`: 기술 사실, 설계 결정, canonical과 조사 근거의 진입점
- `mydocs/manual/verification/`: 시각 검증 정책과 실행법
- `mydocs/manual/codex/`: 저장소 부트스트랩과 현행 문서·Git 절차
- `mydocs/manual/memory/`: 과거 피드백·프로젝트 memory의 historical provenance
- `mydocs/tech/investigations/issue-####/`: 특정 이슈의 가설·실험·관찰
- `mydocs/tech/archive/`: 대체된 계획·설계와 역사 자료
- `mydocs/tech/webhwp/`: 2026-02 webhwp bundle 역분석 기록
- `mydocs/troubleshootings/`: 재현 가능한 증상, 확정 원인과 재발 방지 절차

이슈 초안에 예시로 제시된 빈 분류 디렉터리는 일괄 생성하지 않았다. 문서 지도와 front matter만으로
권위가 분명한 문서는 경로 안정성을 유지하고, 이동 이득이 확인된 클러스터만 독립 커밋으로 분리했다.

## Canonical 정리

- CLI 옵션·동작: `mydocs/manual/cli_commands.md`
- PR 리뷰·통합: `mydocs/manual/pr_review_workflow.md`
- 시각 검증 정책: `mydocs/manual/verification/visual_verification_governance.md`
- 시각 검증 실행: `mydocs/manual/verification/visual_sweep_guide.md`
- HWP 5.0 정오표: `mydocs/tech/hwp_spec_errata.md`
- Document IR LineSeg: `mydocs/tech/document_ir_lineseg_standard.md`
- 파서와 공통 IR 책임: `mydocs/tech/parser_architecture.md`

개별 CLI·검증 문서는 상세 설명으로 남기고 canonical의 옵션·정책을 중복 정의하지 않도록 정리했다.
저장소 루트 `AGENTS.md`와 `CLAUDE.md`는 개인 환경이나 긴 절차를 포함하지 않는 부트로더로 축소했다.

## Maintainer 보완사항 반영

1. 링크 검사 도구를 이동 전에 추가하고, 자동 CI 대신 필요 시 실행하는 로컬 가이드로 정리했다.
2. `kind`와 `status`를 분리하고 `canonical`, `last_verified`를 독립 검사한다.
3. 경로 이동과 함께 활성 문서의 개인 경로, 종료 브랜치, 오래된 명령과 고정 수치를 감사했다.
4. 저장소 루트 `AGENTS.md`를 추가하고 전역 사용자 설정을 저장소 계약에서 제외했다.
5. 내부 링크와 소스·문서 경로 문자열을 새 경로로 갱신하고 외부 이력용 redirect만 31개 유지했다.
6. investigation, troubleshooting, reference 자산의 역할과 승격 경계를 문서 지도에 명시했다.

문서별 하드코딩 메타데이터 목록과 retired-path manifest 의존도를 제거했다. 로컬 검사 도구는 지정한
디렉터리의 새 Markdown을 자동으로 찾으므로 문서가 늘어날 때 Python 목록을 수정하지 않는다. 대규모 이동
작업에서는 base 이후 변경 Markdown과 변경 코드·문서의 redirect 이전 경로를 선택적으로 검사한다.
일반 Markdown 추가·수정에는 GitHub Actions를 실행하지 않는다.

## 최종 검증

- canonical Markdown 상대 링크: 384개 문서 통과
- 최신 `upstream/devel` 이후 변경분: 636개 Markdown, 변경 파일 639개, redirect 31개 통과
- front matter와 canonical 경로: 379개 문서 통과
- `manual` 144개, `tech` 170개, `troubleshootings` 64개 Markdown 모두 분류
- investigation: 32개 이슈 디렉터리, 91개 Markdown
- redirect: 31개, 변경 코드·문서의 이전 경로 재참조 0건
- 활성 장기 문서의 실제 개인 checkout 경로 0건. historical provenance와 일반 `/Users/me` 예시는 보존
- Python 구문과 `git diff --check`: 통과
- 문서 링크 GitHub Actions workflow 제거, 로컬 사용 가이드 추가
- 최신 `upstream/devel` 기준 behind 0
- 제품 동작·테스트 변경: 0건. 제품 소스 주석 2건과 검증 도구 설명 1건의 문서 경로만 현행화

문서와 문서 검증 인프라만 변경했으므로 Cargo 빌드·테스트는 수행하지 않았다.

## 결론

#2072에서 요구한 문서 지도, canonical 분리, 조사·보관 경계, 제한된 redirect, 저장소 부트스트랩,
필요 시 실행하는 링크·메타데이터 로컬 검사를 갖췄다. 이후 문서 이동은 이 정보구조를 다시 설계하는 작업이 아니라,
새로 역할 충돌이 확인된 클러스터를 내용 기준으로 감사하는 일반 유지보수로 처리할 수 있다.
