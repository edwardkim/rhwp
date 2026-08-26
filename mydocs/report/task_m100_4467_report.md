---
kind: report
status: active
last_verified: 2026-08-10
---

# Task #4467 최종 보고서 — 프로젝트 로드맵과 업스트림 경계 정립

- **Issue**: [#4467](https://github.com/edwardkim/rhwp/issues/4467)
- **브랜치**: `task/4467-project-roadmap`
- **devel 기준**: `upstream/devel` `c20377b9e`
- **계획서**: [task_m100_4467.md](../plans/task_m100_4467.md)

## 결론

README에 남아 있던 초기 `v0.5 → v1.0 → v2.0 → v3.0` 로드맵을 루트
`ROADMAP.md`로 분리했다. 초기 방향은 보존하되 현재 프로젝트의 규모와 진행 방식을 반영해,
버전 번호를 엄격한 착수 순서가 아니라 목표의 성숙도와 완료 기준으로 설명한다.

현재는 v1.0 조판 완성도와 v2.0 협업 기반이 함께 진행 중이다. 40명이 넘는 외부 기여자와
두 명의 콜레보레이터가 이미 참여하고 있다는 실측을 반영해, v2.0의 남은 목표를 협업의 시작이
아니라 지속 가능한 역할 분담·호환성·실패 복구·유지 책임으로 고쳤다.

## 주요 결정

- `ROADMAP.md`를 프로젝트 전체 제품 방향의 권위 문서로 두고 README는 현재 위치와 진입점만 제공한다.
- 로드맵은 한국어를 모국어로 사용하는 일반 관심자가 읽는 공개 문서로 작성한다.
- 조판과 협업은 순차 단계가 아니라 병행 축이며, 단계 번호는 완료 목표를 나타낸다.
- 공통 문서 엔진, Web/WASM 기반, 공식 브라우저·VS Code 확장, npm 패키지, CLI 릴리스·설치 패키지,
  GHCR 이미지, GitHub Action과 범용 CLI·MCP·API 계약은 업스트림 범위다.
- 데스크톱·macOS·iOS·Android 앱, 사내 뷰어, Google Docs 연계와 조직별 제품 구현은
  다운스트림을 기본으로 한다.
- 혼합 변경은 제품 구현을 다운스트림에 두고, 여러 프로젝트가 재사용할 엔진 결함 수정과 확장점만
  작은 업스트림 이슈·PR로 분리한다.
- 새 공식 배포 대상은 구현보다 먼저 유지 담당자, 배포 권한, 보안 대응, CI와 장기 지원 책임을
  별도 거버넌스 논의로 합의한다.

## 문서 구조

- `ROADMAP.md`: 제품 로드맵의 권위 문서
- `README.md`, `README_EN.md`: 현재 상태 요약과 로드맵 진입점
- `CONTRIBUTING.md`: 기여 전 업스트림·다운스트림 판단 안내
- `mydocs/README.md`, `llms.txt`, `mydocs/tech/README.md`: 문서 지도 연결
- `mydocs/tech/agent_roadmap/README.md`: #3907을 AI 활용과 자동화의 하위 기술 지도로 구분
- `mydocs/feedback/`, `mydocs/working/`: 네 차례의 작업지시자 판정과 단계별 변경 근거

## 검증

최신 `upstream/devel` `c20377b9e` 위로 네 개의 작업 커밋을 재배치한 뒤 다음 검사를 통과했다.

- `python3 scripts/check_markdown_links.py --changed-from upstream/devel --forbid-redirect-references`
  - self-review 보정 후보 기준 557개 문서, 변경 파일 19개, redirect stub 30개, 내부 상대 링크 이상 없음
- `python3 scripts/check_document_metadata.py`
  - 540개 문서, 메타데이터 이상 없음
- 공식 배포 대상과 CLI·MCP, 다운스트림 제품군 표현 검색 통과
- `git diff --check upstream/devel...HEAD` 통과

문서 전용 변경이므로 Rust·rhwp-studio·WASM 테스트와 시각 검증은 적용하지 않았다.

## 남은 절차

[PR #4512](https://github.com/edwardkim/rhwp/pull/4512)를 Open PR로 생성했다. 작업지시자는
maintainer self-review 경로를 선택했다. 보정과 review 기록을 포함한 최신 head의 CI와 `COMMENTED`
self-review를 확인한 뒤 별도 merge 승인으로 최종 판정한다.
