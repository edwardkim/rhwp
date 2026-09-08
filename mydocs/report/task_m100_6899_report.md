# #6899 결과보고서 — CI Impact Policy Controller 실패 증적 보고

- Issue: #6899. 날짜: 2026-09-08.
- 상태: 구현·로컬 검증 완료, 결과 및 push·PR 생성 승인 대기. **운영 미적용**.
- [수행계획](../plans/task_m100_6899.md) · [구현계획](../plans/task_m100_6899_impl.md) ·
  [원인 조사](../working/task_m100_6899_stage1.md) · [구현 검증](../working/task_m100_6899_stage2.md) ·
  [최종 로컬 검증](../working/task_m100_6899_stage3.md).

## 결과

Controller의 실패 판정은 유지하고, 실행 요약에서 선행 검증 실패와 Controller 자체 오류를 구분하도록
보조 reporter를 구현했다. 원본 run/attempt·실패 job/step 링크와 인식 가능한 테스트명·오류 코드를
제공한다. 조회에 실패하거나 증적이 불명확하면 그 제한을 표시하며 근본 원인을 추정하지 않는다.

기존 보고 부족은 최초 정책 도입 `86b966ac5b`부터 존재한 미구현 영역이다. 사례 run
`34221894722`는 Controller 자체 장애가 아니라 선행 CI의 글자 겹침 검사 실패를 전달한 것이었다.
`baseline 없음` 검출만으로 제품 회귀라고 판단할 수는 없다.

| 상황 | 새 요약에서 확인할 내용 |
| --- | --- |
| 선행 CI/CodeQL/Render Diff 실패 | 원본 실행·attempt, 실패 worker·step, 허용된 핵심 오류 |
| Controller 내부 오류 | 실패 단계 식별, 증적 확인 불가 안내 |
| 로그 부재·권한·timeout·크기 제한 | 미확인/부분 수집 사유와 확보된 원본 링크 |
| stale·성공·pending·정책 차단 | 별도 유형 표시; 불필요한 로그 조회 없음 |

정상 경로 추가 진단 API는 0회이며 실패 경로에 24요청/45초 상한을 둔다. 원문 로그·문서 내용은
복제하지 않는다. 인증 토큰은 로그 storage로 전달하지 않고 기존 CI verdict와 권한은 변경하지 않는다.
Controller 내부 오류는 단계 수준 진단이며 모든 예외의 상세 원문을 자동 해석하는 구현은 아니다.

## 검증과 보완

- Node 정책·분류·진단 112건, 재사용 계약 83건, Python workflow 계약 227건 통과.
- actionlint v1.7.12로 변경 YAML 2개 통과. node 구문·diff 공백 검사 통과.
- 새 테스트 CI 미배선은 Stage 3에서 발견·수정했다. 검사 범위를 넓혀 재실행했으며 누락 검사를 유지한다.
- upstream/devel `e7e978589`를 `adfa14b8a`로 충돌 없이 통합한 상태에서 검증했다.
- 기존 판정 함수는 변경하지 않았고 진단 메타데이터 추가 전후 판정 동일성을 검사했다.
- 실제 과거 실패 job 로그에서는 테스트명·신규 검출 1건·exit 100을 추출했다. live PR head 변경 시에는
  stale로 중단했다. 이는 원격 workflow 적용 후 실행 검증과 구분한다.

제품 source·샘플·baseline·Rust test 변경은 없으므로 Cargo/WASM/시각 검증은 수행하지 않았다.
외부 ShellCheck는 미설치로 별도 실행하지 않았다. 원격 PR CI는 아직 실행하지 않았다.

## 남은 절차와 완료 조건

1. 승인 후 원격 최신 devel 재확인 → 작업 브랜치 push → devel 대상 PR 생성.
2. 채번 후 self-review·필요한 오늘할일 문서를 동일 PR에 포함하고 최신 CI 확인.
3. 승인 후 병합. 이 시점은 devel 구현 반영이며 #6899 운영 완료가 아니다.
4. 별도 승인된 main 배포 뒤 live 요약·원본 링크·판정 보존을 확인하고 #6899 종료.

main 직접 push/dispatch, 보호 규칙 수정, 자동 재실행·comment, 원본 제품 오류 수정은 하지 않았다.
복구가 필요하면 reporter 배선/helper만 되돌리고 기존 CI 정책과 required check는 유지한다.
