---
kind: snapshot
status: historical
canonical: mydocs/manual/mcp_hwp2024Convert_usage.md
last_verified: 2026-10-20
---

# PR #8091 리뷰 — 한컴 변환 MCP status 사용 계약

## 최종 판정

**승인.** 문서 후보 `a113541d89fa7bc72a20d918c3f335aa99e5705e`의 JSON 예제 11개,
메타데이터·링크·공백 검사와 GitHub CI 성공을 확인했습니다. 사용자께서 병합과 후속 처리를
승인했습니다. 이 기록을 추가한 최신 head의 CI와 mergeability는 병합 직전에 별도로 확인합니다.

## 접수와 범위

- [PR #8091](https://github.com/edwardkim/rhwp/pull/8091), 작성자 `jangster77`, base `devel`의 self-review입니다. Reviewer를 지정하지 않았습니다.
- 기본 경로: collaborator_self_merge. 보조 경로: intake_and_review, local_validation, review_only_fast_pass, post_merge.
- [사용 문서](../../manual/mcp_hwp2024Convert_usage.md)의 status 필드, CLI/MCP envelope,
  실제 진행률 관측 시각, 인쇄와 결과 검증·다운로드의 차이를 배포된 계약에 맞췄습니다.
- 실제 2020 엔진과 저장 환경 미확인 시 라우팅 지침, 참조 OLE 사전 거절을 설명했습니다.
  기존 1건 직렬 변환과 큐 대기를 유지하며 병렬·대기 없는 접수 변경을 포함하지 않습니다.
- 관련 이슈 종료 요청은 없습니다. 구현 소스·테스트·fixture·기준 출력은 변경하지 않았습니다.

## 검증과 적용 판정

- 문서의 JSON 예제 11개를 파싱하고 저장소 검사기의 `validate_file`과
  `collect_broken_links`로 수정 문서를 검사했습니다. `git diff --check`도 통과했습니다.
- 후보의 [CI Build & Test](https://github.com/edwardkim/rhwp/actions/runs/39298971782)는 성공했습니다.
  문서 전용 preflight가 성공했고 heavy worker는 정책에 따라 skip했습니다.
  [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/39299058510)도 성공했습니다.
- 배포 소스 `527c4be`는 소프트웨어 검사 82개 중 80개 통과·2개 skip·실패 0개였습니다.
  서버·worker 31개 파일을 확인하고 설정·폰트를 보존했습니다. 실제 비동기 변환 4건과
  의도된 입력 거절 2건, 동시 접수 2건의 큐 대기·순차 실행·다운로드 서명/바이트/SHA를 확인했습니다.
  이 배포 검증은 문서 계약의 근거이며 rhwp 렌더링 일치 검증을 뜻하지 않습니다.
- 조판 원칙·Visual Sweep·검증 입력 커밋: 비해당. 문서 설명만 변경하고 rhwp의 조판,
  저장, 파서 또는 출력 동작과 fixture를 변경하지 않습니다. Rust/WASM 빌드도 비해당입니다.
- 사용자 지시에 따라 서비스 등록 후 전체 147건 재검증은 실행하지 않았습니다.

## 병합 후 처리 계획

최종 merge SHA와 CI 결과, 문서 전용 범위와 heavy worker skip을 PR comment에 남깁니다.
Duration 갱신은 성공 또는 증거 부족에 따른 보류를 확인하며 검증 CI를 다시 실행하지 않습니다.
추가 문서 PR은 만들지 않고 devel fast-forward, 이번 작업 브랜치 정리를 수행합니다.
공유 target과 소유가 불명확한 기존 output 자료는 보존하고 유지 사유를 보고합니다.
