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
- 기본 브랜치 main 적용은 정규 `devel → main` promotion 절차를 따른다. devel 병합만으로 기본 브랜치 경고가 해소됐다고 보고하지 않는다. main 적용 시 새 Actions 분석을 확인한다.
- 기존 자동 Actions 구성과 관련 경고·분석 증거를 보존한 다음 해당 구성만 삭제한다. 다른 언어의 자동 구성은 보존한다.

## 완료·rollback

최신 수동 Actions 분석이 성공하고 오래된 자동 Actions 구성 경고가 제거되면 완료한다. 분석 업로드를 확인하기 전에는 기존 구성을 삭제하지 않는다. workflow/policy 변경은 이 PR의 commit revert로 되돌릴 수 있다. 분석 삭제는 기록 손실이 있으므로 증거 보존과 대체 분석 확인을 선행한다.

## 증적

기존 구성 진단: 기본 작업공간의 ignored `output/pr-review/codeql-config-cfee-20261007/`. 이번 실행 결과는 이 worktree의 ignored `output/pr-review/codeql-actions-20261007/`에 보존한다. 제품·조판 경로 변경이 없으므로 Rust/WASM 빌드와 Visual Sweep은 비해당이다.


## 구현 및 로컬 검증 결과

- classifier version 8 / policy version 7로 전환했다. compact 상태의 Actions 축은 `ac`이며 세 consumer의 지원 버전도 함께 갱신했다.
- workflow 내 review-only 재사용, shared trusted post-merge 재사용, CI Impact Policy 감사, main promotion 실행 정책에서 Actions를 검사한다.
- 수정 전 검사: Actions 전체 범위·policy 감사 4개, workflow 선택·legacy candidate 3개, post-merge legacy coverage 1개, promotion 검사 1개(3개 하위 조건)가 의도한 원인으로 실패했다.
- 수정 후 Node 495개, 주요 CI/CodeQL 계약 Python 115개, 전체 workflow Python 185개, promotion Python 35개가 통과했다. 공식 actionlint 1.7.11 checksum 확인 후 변경한 세 YAML을 검증했고, Node 구문·YAML 파싱·diff 검사도 통과했다. shellcheck는 설치되어 있지 않아 actionlint의 shellcheck 외 검증을 실행했다.
- fixture 기대값 변경은 정책 버전과 전체 분석의 Actions 추가만 반영한다. 기존 선택적 언어·작업 범위는 변경하지 않았다.
- GitHub exact-head CI와 main 적용·자동 구성 삭제는 아직 수행하지 않았다. 기존 자동 구성 증거는 보존되어 있다.


## 적용 시점 결정

PR #7651 등록 후 메인테이너가 main 적용을 다음 `devel → main` 통합으로 지정했다. 이번 작업은 devel의 설정 반영까지이며, main 신규 Actions 분석 확인과 자동 구성 정리는 #7650의 후속 범위로 유지한다.
