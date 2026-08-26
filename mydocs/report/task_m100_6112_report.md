# Task M100 #6112 최종 결과 보고서

- **이슈**: [#6112](https://github.com/edwardkim/rhwp/issues/6112)
- **선행 이슈**: [#5738](https://github.com/edwardkim/rhwp/issues/5738)
- **브랜치**: `codex/issue-6112-toolbox-collapse`
- **구현 commit**: `d707d4cf2b4efef757420381b6a460f1b325482e`
- **작성일**: 2026-08-26 KST
- **현재 상태**: 로컬 구현·검증·사후 문서 승인 완료, PR 준비 진행

## 결과 요약

기존 `view:toolbox-basic`을 버튼·보기 메뉴·단축키의 단일 진입점으로 유지하면서 한글 2024식 기본
도구 상자 접기/펴기를 추가했다. 신규·미설정 사용자는 큰 아이콘 도구 상자가 접힌 상태로 시작하고,
기존에 명시적으로 저장된 표시 선택은 보존된다.

## 완료 조건 판정

| 완료 조건 | 결과 | 근거 |
| --- | --- | --- |
| 신규 기본 도구 상자 접힘 | 충족 | user settings·theme-init 기본값과 브라우저 TC1 |
| 서식 도구 상자 기본 보임 유지 | 충족 | `toolbarFormat=true`, 브라우저 TC1 |
| 저장된 표시값 복원 | 충족 | 버튼·단축키 저장 후 리로드 E2E |
| 버튼·메뉴·단축키 단일 커맨드 | 충족 | `data-cmd`, shortcut map, dispatcher 계약 테스트 |
| 버튼 접근성 상태 동기화 | 충족 | 단위 테스트와 브라우저 `aria-expanded`·label·title 확인 |
| 첫 페인트 깜빡임 없음 | 충족 | 숨김 상태 리로드 `0/35` visible frames |
| Studio 전체 회귀 | 충족 | 1,138 tests, 1,137 pass, 1 skip, 0 fail |
| 타입·프로덕션 빌드 | 충족 | `tsc && vite build` 성공 |

## 계획 대비 실제

- 예상대로 #5738의 커맨드, 설정, DOM 동기화와 FOUC 자산을 재사용했다.
- 버튼 아이콘을 새 이미지로 추가하지 않고 CSS와 기존 색상 토큰으로 구현했다.
- 단축키는 textarea 경로뿐 아니라 버튼 포커스의 전역 경로도 필요해 `GLOBAL_VIEW_SHORTCUTS`에
  포함했다.
- 서식 도구 상자 소형화는 사용자와 합의한 비범위로 남겼다.

## 변경 영향

- 제품 코드·마크업·스타일 9개 파일
- 테스트 4개 파일
- Rust와 문서 포맷 엔진 변경 없음
- 새 바이너리·이미지 자산 없음

## 검증 결과

- focused 테스트: 24/24 통과
- Studio 전체: 1,137 pass / 0 fail / 1 skip
- TypeScript·Vite production build: 통과
- 실제 브라우저 버튼 클릭, Ctrl+F1, 저장·리로드: 통과
- 숨김 첫 페인트: visible frame 0/35
- `git diff --check`: 통과

## 절차 감사

이슈 등록과 계획 승인 전에 구현 초안이 만들어져 Hyper-Waterfall의 순서를 지키지 못했다. 이 문서
묶음은 이를 사후 복구하지만 누락된 승인 게이트를 소급해 정상 처리로 바꾸지 않는다. 구현 commit을
단계별 과거 기록처럼 재작성하지 않고, 각 단계 문서는 최종 diff와 실행 로그를 대사한 감사 증거로
명시했다.

## 남은 작업

1. 완료 — 작업지시자가 수행·구현 계획, 단계 보고, 최종 보고와 절차 복구 문서를 검토하고 PR
   게이트 진입을 승인했다.
2. 최신 `upstream/devel`을 동기화하고 push 직전 공통 포맷·문서 검사를 실행한다.
3. 원격 branch push와 `devel` 대상 PR을 생성한다.
4. PR CI·리뷰·사용자 판정 뒤 merge와 #6112 close를 별도 승인으로 처리한다.
