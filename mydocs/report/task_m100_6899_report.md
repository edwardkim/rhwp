# #6899 결과보고서 — CI Impact Policy Controller 실패 증적 보고

- Issue: #6899. 날짜: 2026-09-08.
- 상태: [PR #6903](https://github.com/edwardkim/rhwp/pull/6903) CI 실패 뒤 재설계 R2 구현·실제 실패 재생 검증 완료. 수정 push·원격 CI·운영 적용 미수행.
- [수행계획](../plans/task_m100_6899.md) · [구현계획](../plans/task_m100_6899_impl.md) ·
  [원인 조사](../working/task_m100_6899_stage1.md) · [구현 검증](../working/task_m100_6899_stage2.md) ·
  [최종 로컬 검증](../working/task_m100_6899_stage3.md).

## R2 결과 (현재 판정)

후속 반영 승인에 따라 최신 devel `9e4f504fe`를 통합했다. 제출 전 재검증은 Node 122건,
재사용 계약 120건, Python workflow 230건 및 actionlint 통과다. 아래 432건은 R2 최초 검증 기록이다.
판정·재평가·트리거를 유지하고 보고 개선만 기존 PR에 반영한다. 새 head 원격 CI·main 적용은 별도 gate다.

이전 구현은 main에 미적용인 문제 외에도 **설치 네트워크 오류 상세와 GHAS CodeQL 실패를 누락하는
설계 결함**이 있었다. 로컬 테스트 개수가 실제 보고 품질의 충분한 증거가 아니었다.

- 실행 실패와 보안 check를 독립 수집하고, 모든 실패 목록을 먼저 읽은 뒤 worker 로그를 보강한다.
- 보고를 실패 위치·관측 오류·미확인 범위·다음 조치·원본 링크로 구성한다.
- CodeQL Analyze workflow 성공이어도 동일 head의 GHAS 실패를 표시한다. 기존 policy 판정은 변경하지 않는다.
- `checks: read`를 추가했다. audit는 성공/pending·CodeQL workflow 증적 유무와 독립적으로 조회하므로 항상 API 0회 주장은 철회한다.
- 실제 #6903 head `38bb7bb87` 재생: 기존 보고는 exit 1만 표시하고 GHAS 경고를 누락했다.
  R2는 설치 연결 재설정·다운로드 실패·테스트 미실행 및 GHAS High 1건의 규칙·경로/202행·설명을 표시했다.
- 실제 R2 진단 비용: 8요청, 최종 재생 약 3.88초(첫 재생 약 3.65초). 준비 API 3회 별도. 소수 관측이며 CI 성능 목표가 아니다.
- CodeQL이 지적한 테스트 assertion은 태그명 정규식 대신 `<`/`>` 부재와 대소문자 escape 검증으로 보완했다.
  sanitizer 제품 결함으로 단정하거나 alert dismiss하지 않았다. CodeQL 경고 해소는 수정 head의 원격 검사로 확인해야 한다.
- Node 122건, 재사용 계약 83건, Python workflow 227건 통과. 변경 YAML actionlint 통과.

[R2 증적·제한·명령](../working/task_m100_6899_rework_stage1.md)을 기준으로 판단한다.
아래 최초 결과의 항상 API 0회·권한 불변·CI 미실행 표현은 당시 기록이며 R2 현재 계약이 아니다.

## 최초 결과 (R1 기록)

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

## 남은 절차와 완료 조건 (R2)

1. 완료: 원격 최신 devel `e7e978589` 재확인 → `bdd1e8a6e` push → devel 대상 PR #6903 생성.
2. R2 수정 결과 승인 후 최신 devel/PR head 확인, 기존 PR에 수정 push·본문 현행화 후 CI 재검증.
   [리뷰 접수 기록](../pr/archives/pr_6903_review.md)은 보류 상태를 유지하며 최종 self-review는 새 CI 확인 뒤 진행.
3. 승인 후 병합. 이 시점은 devel 구현 반영이며 #6899 운영 완료가 아니다.
4. 별도 승인된 main 배포 뒤 live 요약·원본 링크·판정 보존을 확인하고 #6899 종료.

main 직접 push/dispatch, 보호 규칙 수정, 자동 재실행·comment, 원본 제품 오류 수정은 하지 않았다.
복구가 필요하면 reporter 배선/helper만 되돌리고 기존 CI 정책과 required check는 유지한다.
