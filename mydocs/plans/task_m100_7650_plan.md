---
kind: plan
status: active
last_verified: 2026-10-07
---

# CodeQL Actions 분석 및 구성 정리 계획

Issue: #7650

## 근거와 범위

메인테이너가 설정 변경을 승인했다. O3 실행·보안 계약 변경으로 분류한다. 작업 기준은 최신 upstream/devel `b3c3047db575d146dff9d39098e6de98c4630b3e`다. 기존 자동 Actions 구성의 마지막 분석은 2026-04-07 성공 결과이며, Default setup은 not-configured다. 현행 수동 워크플로는 Actions를 분석하지 않는다.

## 구현

1. Actions를 네 번째 언어로 추가하고 기존 세 언어의 순서는 유지한다.
2. trusted base classifier의 전체 분석, workflow fallback·canonical selection, policy 상태 인코딩·감사와 재사용 요구 job에 같은 네 언어를 적용한다. 기존 base의 세 언어 full 응답은 fail-closed로 네 언어 전체 분석으로 확장한다.
3. Actions job의 누락·실패·선택된 Analyze 생략, 과거 세 언어 candidate 재사용을 거부하는 회귀 계약을 실행한다. 기존 선택적 Rust/JS/Python 경로는 유지한다.
4. 제품 코드·권한·보안 판정·push trigger를 완화하지 않는다.

## 검증 및 적용 순서

- 기존 코드에서 새 Actions coverage 검사가 실패하는지 확인하고, 구현 뒤 classifier·policy·report·controller 계약 Node 검사와 CI/CodeQL workflow Python 검사를 실행한다. YAML·actionlint·diff 검사도 수행한다.
- 한국어 PR과 self-review로 devel에 통합한다. 실제 Actions Analyze 및 SARIF 업로드 성공을 확인한다.
- 기본 브랜치 main 적용은 devel 전체 릴리즈와 구분한다. 최소 운영 변경만 적용하고 main의 새 Actions 분석을 확인한다.
- 기존 자동 Actions 구성과 관련 경고·분석 증거를 보존한 다음 해당 구성만 삭제한다. 다른 언어의 자동 구성은 보존한다.

## 완료·rollback

최신 수동 Actions 분석이 성공하고 오래된 자동 Actions 구성 경고가 제거되면 완료한다. 분석 업로드를 확인하기 전에는 기존 구성을 삭제하지 않는다. workflow/policy 변경은 이 PR의 commit revert로 되돌릴 수 있다. 분석 삭제는 기록 손실이 있으므로 증거 보존과 대체 분석 확인을 선행한다.

## 증적

기존 구성 진단: 기본 작업공간의 ignored `output/pr-review/codeql-config-cfee-20261007/`. 이번 실행 결과는 이 worktree의 ignored `output/pr-review/codeql-actions-20261007/`에 보존한다. 제품·조판 경로 변경이 없으므로 Rust/WASM 빌드와 Visual Sweep은 비해당이다.
