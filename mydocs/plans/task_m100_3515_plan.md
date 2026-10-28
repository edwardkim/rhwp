# Issue #3515 수행·구현 계획 — 선택 실행 E2E

- Parent: #3512, 선행: #3513 설정 수명주기 검사.
- 2026-09-27 사용자 지시와 [메인테이너 동의](https://github.com/edwardkim/rhwp/pull/7283#issuecomment-5844951296)에 따라 범위를 축소한다.
- 현재 구현: 개발자·에이전트가 배포 후보 폴더를 지정해 실행하는 보조 도구. 수동 배포 확인은 유지한다.

## 구현 범위

1. PR이 추가한 자동 Chrome E2E, 영향도 분류, Frontend 승격, 필수 집계, cache workflow를 제거한다.
   일반 CI의 기존 검사는 유지하며 main/tag 전용 자동 실행이나 새로운 dispatch를 추가하지 않는다.
2. smoke/download/lifecycle와 진단은 유지한다. `npm --prefix rhwp-chrome run test:e2e -- --dist <후보>`를
   단일 진입점으로 제공하고 기존 산출물을 다시 빌드하지 않는다. npm/editor 검사를 호출하지 않는다.
3. 후보 파일 목록·SHA-256, 도구 환경, suite별 성공/실패/미실행을 JSON으로 보존한다.
   후보 변경·잘못된 입력·timeout·이전 결과 폴더 덮어쓰기는 성공으로 처리하지 않는다.
4. ZIP을 직접 처리하지 않는다. 압축을 해제한 폴더를 지정한다. 스토어 설치/업데이트·Edge/Firefox·
   실제 문서 표시 품질과 최종 배포 판단은 수동 확인 범위다.

## 검증과 제출

- 기존 CI Node/Python 계약과 새 실행기의 후보 전달·실패·timeout·결과 보존 계약.
- 현재 source로 준비한 실제 확장 폴더를 저장소 밖으로 옮겨 전체 E2E와 의도적 실패 실행.
- common base 대비 `.github/`, `scripts/` 변경 없음 및 최신 devel merge simulation 확인.
- 결과는 [보고서](../report/task_m100_3512_report.md)에 기록하고 PR 제목·본문을 현재 범위로 다시 쓴다.
- 자동 CI 도입을 전제로 한 #3515의 원래 완료 조건이나 #3512 전체 완료를 주장하지 않는다.
  이슈 종료·외부 증적 보고서 연동은 별도 후속 판단이다.

## 이전 설계

2026-09-20~26의 CI 설계와 실행 경계 보완은 [당시 계획](https://github.com/edwardkim/rhwp/blob/0b3da1cbd2ca6881bd760303312966f03cfabbbe/mydocs/plans/task_m100_3515_plan.md)에 보존돼 있다.
현재 실행 절차는 [확장 매뉴얼](../manual/chrome_edge_extension_build_deploy.md#39-배포-후보-선택-실행과-결과-보고-3515)을 따른다.
