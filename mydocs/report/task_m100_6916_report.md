# #6916 결과보고서 — 제품과 선택적 Gym의 의존 경계

- Issue: [#6916](https://github.com/edwardkim/rhwp/issues/6916)
- 날짜: 2026-09-09
- 브랜치: `task_m100_6916`
- 상태: PR #6930 생성 및 code candidate CI 성공 후 self-review 완료. 검토 기록 push·병합 승인 대기.
- 검증 code head: `369e789eea1839462674e8a96f09c453ff2ba111`
- 독립 빌드 source head: `882c0dfc9a9a73d3bf5b679436697a85b000cc5f` (위 코드 + 단계 문서만 변경)
- 기준 devel: `c72ad805cc60e4a5cf5689c18b44e7214cec68fe`

## 1. 결론

**Gym 폴더가 없는 소스에서 rhwp Linux 제품과 Docker WASM을 빌드했고,
별도 설치본의 CLI/MCP 동작 및 npm 배포 포함물을 확인했다.**
Gym은 같은 저장소에 보존하며, 평가할 때만 외부 rhwp 바이너리를 지정해 사용한다.
이번에 만든 설치본으로 별도 Gym의 core-cli 54개 과제를 모두 통과했다.

제품이 Gym을 요구하던 컴파일 의존을 제거했으며 의존 방향은 **Gym → rhwp CLI/API**다.
Gym을 삭제하거나 일반 문서 처리 API를 제거한 작업이 아니다.
Gym 자체의 평가 품질·온보딩·전수 점수를 이번 결과로 보증하지 않는다.

## 2. 원인과 최소 수정

기존 MCP 리소스는 `include_str!("../gym/README.md")`를 사용했다.
오프라인 문서 안내를 제공하려던 설계가 CLI 컴파일에 Gym 파일을 필수로 만들었다.
도입 이력과 Oracle sparse checkout의 과거 보정 계보는 [Stage 1](../working/task_m100_6916_stage1.md)에 있다.

- `rhwp://docs/gym` URI와 name/title/MIME·list/read·오류 계약을 유지했다.
- 본문은 제품 소유의 [선택적 Gym 안내](../manual/gym_optional_tool.md)로 바꿨다.
  description·본문·size 변경은 승인된 공개 계약 변경이다. 전체 Gym README를 복제하지 않았다.
- 제품에 Gym 파일 조회·환경변수 fallback·자동 HTTP 다운로드를 추가하지 않았다.
- Oracle advisory sparse 입력과 mirror test를 새 제품 안내 경로로 함께 바꿨다.
  trigger·권한·timeout·verdict·승격 정책은 보존했다.
- 신규 MCP 회귀 원본은 `tests/cases/issue_6916_gym_optional_resource.rs`에만 추가했다.
  generated suite·manifest, Cargo marker, 새 crate 의존성은 제출하지 않는다.
- Gym 운영 정본과 에이전트 지침에 제품과 선택적 도구의 경계를 반영했다.

일반 CLI/API, parser/renderer, Gym task/reference/check/score, workflow promotion policy,
CI impact classifier, package manifest는 변경하지 않았다.

## 3. 검증 정산

| 검증 | 실제 결과 |
| --- | --- |
| 수정 전/후 신규 MCP 계약 | 동일 최종 테스트 원본으로 수정 전 의도한 assertion 실패, 수정 후 1 passed |
| 기존 focused MCP 계약 | 33 passed |
| Oracle/Gym workflow·조직 계약 | Python 24 passed, CI classifier Node 44 passed |
| Gym 구조 단위·감사 | 3,170 passed, 기존 self-diff fixture 부재 1 skipped; audit/oracle probe/selftest 성공 |
| Gym 없는 Linux release | fresh target, exit 0, 7m 59s |
| Linux 릴리스 구성 포장·설치 | binary/LICENSE/README/README_EN만 포장·해제. 저장소 밖에서 version/help/MCP 성공 |
| 설치본 MCP | 광고된 리소스 15개 전부 읽기 성공. 정적 11개 size 일치, 생성형 4개는 기존 계약대로 size 생략 |
| Gym 없는 Docker WASM | Rust 1.93.1 / wasm-pack 0.15.0, fresh target, 성공, 7m 06s |
| npm dry-run | `@rhwp/core@0.8.6`, 허용된 7개 파일만 포함, Gym 파일 없음 |
| 외부 바이너리 지정 Gym canary | core-cli 54/54 passed, failed/skipped/missingArtifact/failedScore/buildError 모두 0 |
| Rust PR 선행 lint | fmt, native·WASM·workspace all-target Clippy, workspace build, manifest 전부 통과 |
| release-test 전체 nextest | **9,302 passed, 0 failed, 기존 ignored 46건**. 실행 303.119s, test build 3m 29s |

명령·SHA·로그·제외 사유·도구 버전은 [Stage 2](../working/task_m100_6916_stage2.md),
[Stage 3](../working/task_m100_6916_stage3.md)에 기록했다. 전체 nextest inventory는 9,348건이고,
46건은 모두 명시적 ignored였다. 실패를 필터로 제거하거나 기준값을 완화하지 않았다.

## 4. 한계와 오류 정정

- Windows/macOS 설치 실행은 미확인이다. frontend·VSIX·브라우저 확장 전체 재빌드는
  승인된 bin-only 영향 분석에 따라 생략했다. 조판 코드는 변경하지 않아 전수 시각 검증을 추가하지 않았다.
- 실제 npm 게시는 하지 않았다. 아래 후속 절차에서 원격 CI/Oracle 실행을 확인했으나 제품 배포 완료를 뜻하지 않는다.
- 제품 안내 원본은 22,509 → 1,250 bytes로 줄었다. 이는 문서 입력 크기이며 제품 성능 개선 계측이 아니다.
- nextest 0.9.137의 권장 버전/다른 CI profile 설정 경고는 남았다. default profile의 실행 결과는 위와 같다.
- Stage 2의 최초 테스트 dependency 오류, Stage 3의 로컬 검사 size 가정 오류와
  Docker 산출물 UID 가정 오류는 각 보고서에 정정 과정을 기록했다. 이들을 제품 회귀로 세지 않았다.
- 기존 장기 문서 4개의 metadata 누락은 기준 devel에도 존재하며 범위 밖으로 보존했다.
- 조직·프레임 감사에는 같은 저장소 Gym 자산 검사가 남는다. **제품 독립성**과
  **Gym을 뺀 checkout의 저장소 전체 CI 성공**은 다른 조건이다. 후자는 이번 목표가 아니다.

## 5. 증적·보존·다음 절차

- Linux 포장: `output/6916/stage3/rhwp-6916-linux-x64.tar.gz`.
- 독립 native binary SHA-256:
  `7186832ca29aaf4289e5b6b3caa1094098be59e91829b734a1eaeedc894fa68e`.
- Docker WASM SHA-256:
  `5eca359b98e8c16efa90d9fe09e95f302f52c0ada2a37302eecccbec6ae354be`.
- 로그·JSON·로컬 실행 보조 스크립트: `output/6916/stage2/`, `output/6916/stage3/`.
  이들은 로컬 검증 증적이며 source PR에는 올리지 않는다.
- 기존 Studio pkg 두 파일의 전후 SHA-256 일치를 확인했다. 개발 서버·기존 Gym 제출물·공유 target을 보존했다.
- 검증 전용 `/tmp/rhwp-6916-stage3.GhIEHx`와 `rhwp-6916-review`는 결과/PR 확인에 재사용할 수 있게 유지한다.
  타스크 종료 후 이번에 생성한 정확한 경로만 정리하고 공유 캐시는 삭제하지 않는다.

후속 승인으로 [PR #6930](https://github.com/edwardkim/rhwp/pull/6930)을 생성했고
`2577e3328`의 CI·CodeQL·관련 검사가 성공한 뒤 [self-review](../pr/archives/pr_6930_review.md)를 완료했다.
Oracle sparse 제품 빌드는 실제 성공했고 verdict=completed, compareExit=0이었다.
단, 쪽수 비교는 1,030쌍 중 999 match / 29 mismatch / 2 error(unpaired 414)로 전건 일치가 아니다.
비교 잔여 항목의 개별 원인·회귀 여부를 이번 분리 작업의 완료 주장에 포함하지 않는다.
다음은 검토 기록 push 승인 → 최신 trailing head CI → 병합·이슈 close 승인 절차다.
복구가 필요하면 제품 안내/include/Oracle sparse 입력·mirror test를 같은 단위로 되돌리며,
Gym 자산이나 사용자 문서를 삭제하지 않는다.
