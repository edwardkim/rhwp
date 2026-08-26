# #4467 Stage 4 완료 — 업스트림과 다운스트림 경계 명문화

- **Issue**: [#4467](https://github.com/edwardkim/rhwp/issues/4467)
- **브랜치**: `task/4467-project-roadmap`
- **피드백**: [업스트림과 다운스트림의 경계](../feedback/task_m100_4467_upstream_downstream_boundary.md)
- **완료일**: 2026-08-10 KST

## 1. 작업지시자 판정

rhwp는 브라우저 확장, VS Code 확장과 npm을 공식 업스트림 배포 대상으로 유지하면서, 공통 엔진을
사용하는 데스크톱·사내 뷰어·Google Docs·macOS·iOS·Android 앱과 백엔드 서비스가 다운스트림에서
활발히 파생되는 생태계가 되었다. 이제 로드맵이 구현 위치와 유지 책임의 경계를 안내해야 한다.

## 2. 업스트림 범위

- HWP 계열 공통 엔진과 조판·편집·저장·출력
- 공용 Web/WASM 기반
- Chrome·Edge·Firefox 확장 프로그램, VS Code 확장 프로그램, npm 패키지
- 여러 자동화와 백엔드가 재사용하는 CLI·MCP·공개 API 계약
- 공통 회귀·시각 검증, 보안, 호환성, 문서와 공식 릴리스

CLI와 MCP는 모든 서비스별 구현을 업스트림에 넣는 범위가 아니라, 다운스트림이 안정적으로 연결할
공통 규약으로 정의했다.

## 3. 다운스트림 범위

- 데스크톱과 macOS·iOS·Android 완제품
- 사내 HWP·HWPX 뷰어와 조직별 업무 시스템
- Google Docs 등 특정 외부 서비스 연계
- 조직별 인증, 권한, 저장소, 결재, 과금, 배포와 전용 화면

다운스트림은 제품 설계와 사용자 지원, 데이터·보안, 플랫폼별 패키징을 독립적으로 책임한다.

## 4. 기여 판단 기준

ROADMAP에 변경 유형별 권장 위치 표를 추가했다. 두 범위가 섞이면 제품 코드는 다운스트림에 두고,
범용 결함 수정과 재사용할 확장점만 작은 이슈·PR로 분리한다. 새 공식 배포 대상은 유지 담당자,
배포 권한, 보안 대응, CI와 장기 지원 책임을 먼저 합의한다.

CONTRIBUTING에도 PR 전 판단 절을 추가해 처음 온 기여자가 이 경계를 찾을 수 있게 했다.

## 5. 기존 판정

PR #4322의 Windows Tauri 셸과 사내 NSIS 정책을 다운스트림으로 안내한 판정과 일치한다. 이번 변경은
그 판정을 특정 PR의 예외가 아니라 전체 생태계에 적용할 일반 원칙으로 정리했다.

## 6. 검증

현재 로컬 기준선에서 다음 검사를 통과했다.

```text
python3 scripts/check_markdown_links.py --changed-from upstream/devel --forbid-redirect-references
검사 문서: 536개 / 변경 파일: 17개 / redirect stub: 30개
내부 Markdown 상대 링크: 이상 없음

python3 scripts/check_document_metadata.py
메타데이터 검사 문서: 521개
문서 메타데이터: 이상 없음

공식 배포 대상, CLI·MCP, 다운스트림 제품군과 새 플랫폼 판단 기준 표현 검사
필수 항목 누락 없음

git diff --check
통과
```

최신 원격 `devel`에 작업 브랜치를 다시 정렬한 뒤 같은 검사를 최종 반복한다.
