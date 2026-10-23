# #8174 2단계 보완 — CLI·Rust 소비 수정

Issue: [#8174](https://github.com/edwardkim/rhwp/issues/8174)

- 착수 승인: 2026-10-23 사용자 「수정지점과 우선순위를 착수하는 승인을 합니다.」
- 착수 head: `f4342c4f1`.
- 제품 base: `def947c2149df5bc812d801e6e79be19a403b4d1`. 원격 `devel`과 일치함을 확인했다.
- 근거: [구조 재조사 A01–A07](../tech/investigations/issue-8174/README.md),
  [결정 대장·범위 v1](../plans/task_m100_8174_maintainer_decisions.md).
- 상태: 착수. 아래 결과는 실제 실행 뒤 갱신한다.

## 실행 순서와 완료 조건

1. **A01:** 기존 8개 crate 경계를 유지한다. 내부 path에 버전을 병기하고 `rhwp-svg2pdf`를 직접
   연결해 root patch 의존을 없앤다. 라이선스·고지·내장 파일을 포함하는 Cargo 파일 선별과 실제
   `.crate`를 확인한다. 로컬 검증 registry에서 포장한 source를 소비하며 repository path나 patch
   fallback으로 통과시키지 않는다. 기존 `publish = false`와 제품 0.8.7은 이번 보완에서 유지한다.
2. **A02:** `HwpError`에 표준 `Error`를 추가한다. variant·표시 문구는 유지하고 이미 재현한
   외부 소비 앱의 E0277을 수정 후 컴파일·실행으로 확인한다.
3. **A03·A06:** 기존 parser/IR/service/core와 형식별 저장·손실 보고·설치 bin의 사용 안내를 작성한다.
   `font-metric-gen`은 기본 설치에서 제외할 tools feature로 구분하고 기존 생성 명령을 함께 보정한다.
4. **A04:** `export-text`의 로더만 core 경로로 연결한다. 같은 core의 텍스트 추출을 사용하며
   실제 기본 출력·JSON·페이지 선택·절단·잘못된 입력·인자의 exit/stdout/stderr를 기존 CLI와 대조한다.
5. **A05·A07:** 현재 최소 feature가 parser-only가 아님을 안내하고 추가 crate type의 산출물을
   확인한다. 새 경량 feature·바인딩 crate 분리는 필요성과 비용을 확인하기 전 구현하지 않는다.

기존 PDF 보안 fork의 코드·기능 제한·feature 기본값·WASM start/export를 유지한다.
이번 실행은 원격 push·PR 생성·실제 registry 게시·인증 설정을 포함하지 않는다.

## 결과와 남은 조건

실제 코드·패키지·소비 검증 뒤 기록한다. PR 제출 게이트와 실제 게시 준비 전체의 완료는 각각
필수 검증·정책·별도 승인을 확인한 뒤 판단한다.
