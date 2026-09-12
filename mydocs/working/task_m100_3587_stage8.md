# #3587 Stage 8 — B 종료 통합 검증

- 승인: 메인테이너 「다음 절차 진행을 승인합니다」.
- 검증 HEAD: `04015b8f5` (`task_m100_3587`). 제품/테스트는 Stage 7의 `ed147d8ac`와 같다.
- 상태: **B 종료 통합 자동 검증 완료, 결과 승인 대기. #3587 전체·PR 준비 완료는 아니다.**
- 선행: [Stage 7](task_m100_3587_stage7.md), [B 계획](../plans/task_m100_3587_impl_b.md).

## 범위

복사·저장 구현은 유지한다. Enter 경계의 Studio 재편집 페이지네이션 결함은
메인테이너 결정에 따라 [#7065](https://github.com/edwardkim/rhwp/issues/7065)에서 나중에
처리하며 이번 검증에서 고치거나 PASS로 바꾸지 않는다. C/D 구현·Gym 실행·원격 push·PR은
이번 승인 범위가 아니다. Stage 6의 1/10/100회 비용 계측은 재수행하지 않는다.

## 실행 계획과 환경

- 기존 `/home/edward/mygithub/rhwp-review-3587`을 clean 확인 후 위 SHA로 전환했다.
- 고정 target `/home/edward/mygithub/rhwp/target/pr-review` 116 GiB를 재사용한다.
- 16 CPU, RAM 31 GiB / available 28 GiB, 디스크 여유 350 GiB. 기존 Cargo 실행 없음.
- 전체 nextest는 자체 worker를 쓰는 코퍼스 검사와 자원을 공유하도록 8 threads로 실행한다.
- 로그: `output/3587/b-integrated/`. 동일 target의 Cargo 검사는 순차 실행한다.
- prepare → fmt → native Clippy → WASM lib Clippy → workspace build → workspace/all-target
  Clippy → manifest → 배정 도구 계약 → 전체 nextest → Native Skia 3종 순서다.
- 시작 기준 `59a11f180ad1bd5cadcbbf0a6dc9d0162f4a0a21` 이후 신규 samples 12개를
  `RHWP_SECURITY_SWEEP_SAMPLES_JSON`으로 전체 nextest에 전달한다. 실제 목록과 래칫 dump를
  같은 로그 폴더에 남긴다. 기존 baseline을 통과시키기 위해 임의 변경하지 않는다.
- Stage 7 Docker WASM 및 브라우저 검증 이후 변경은 문서뿐이다. 제품/테스트 diff가 없음을
  확인하고 SHA-256 `6f0c57aae5e72757d74b8a6d7c0c1ce95d05519923101f0cf3af1d81c85f881b`의
  기존 Docker 산출물을 재사용한다. 이는 #7065 시각 실패를 해소했다는 의미가 아니다.

## 결과

| 검사 | 결과 | 로그 |
| --- | --- | --- |
| generated suite prepare | PASS, 1,273 sources / 48 integration targets | `prepare.log` |
| fmt 적용 / check | PASS, review tracked 변경 없음 | `fmt.log`, `fmt-check.log` |
| native Clippy | PASS, 0.792초 | `clippy-native.log` |
| WASM lib Clippy | PASS, 43.802초 | `clippy-wasm.log` |
| workspace build | PASS, 80.720초 | `build-workspace.log` |
| workspace/all-target Clippy | PASS, 68.588초 | `clippy-workspace.log` |
| generated manifest | PASS | `manifest.log` |
| suite 배정 도구 계약 | 23 PASS / 0 FAIL | `manifest-contract.log` |
| 전체 nextest | **9,579 PASS / 0 FAIL / 47 skipped**, 78 binaries | `nextest.log` |
| Native Skia lib | **4,112 PASS / 0 FAIL / 13 ignored**, 186.947초 | `skia-lib.log` |
| Native Skia picture placeholder | **2 PASS / 0 FAIL**, 대상 밖 180 skipped, 140.998초 | `skia-picture.log` |
| Native Skia direct PDF | **4 PASS / 0 FAIL**, 대상 밖 197 skipped, 6.222초 | `skia-pdf.log` |

전체 nextest의 컴파일은 4분 18초, 실행은 353.631초, 명령 전체는 613.670초다.
3건 slow 알림은 모두 최종 통과했으며, skipped 47건을 통과 수에 더하지 않는다.
신규 samples 12개를 전달한 보안 테스트도 PASS다. 목록은 `sample-inputs.json`에 있다.
실행별 SHA·인자·종료 코드·시간은 `results.json`에 기록한다.

nextest 0.9.137 / 권고 0.9.140 차이와 `report-skipped` 미지원 경고를 그대로 기록한다.
이번 검증을 위해 도구·테스트 정책·baseline은 바꾸지 않았다. 변경 문서 5개의 로컬 링크 검사와
`git diff --check`도 PASS다. #7065의 Enter 중간 상태 실패는 위 정규 테스트가 포괄하지
못하는 별도 결함이며, 9,579 PASS를 근거로 해소된 것으로 보지 않는다.

## 판정과 다음 순서

- #3587의 자동 계약 81건은 위 전체 nextest에 포함되어 모두 통과했다. 별도 중복 실행을
  추가해 통과 건수를 부풀리지 않는다. Native Skia lib 합계는 root 3,930건과 내부 lib
  15/165/2건이며, 전체 nextest와 중복되므로 두 합계를 하나의 독립 테스트 수로 더하지 않는다.
- 같은 제품/테스트의 Docker WASM은 Stage 7 결과를 재사용했다. 현재 `pkg/rhwp_bg.wasm`의
  해시도 일치한다. 이번 회차에서 새 WASM 빌드·한컴 출력·시각 승인을 받았다고 쓰지 않는다.
- `samples/rnote/labnote-001.hwp` 반복 결과를 한컴에서 정상 편집할 수 있다는 메인테이너
  판정과 이번 저장·참조 계약 검증을 B 결과로 정리한다. 모든 컨트롤·템플릿의 호환성이나
  #7065를 포함한 모든 재편집 동작을 보증하지 않는다.
- 이번 회차에는 **제품 source·test·baseline 수정이 없다**. main 작업 폴더와 review
  worktree의 기존 cache·generated 자료를 삭제/이동하지 않았다. review tracked 상태는 clean이다.
- 진행 시점이 뒤처진 수행계획/구현계획/B 계획의 상태를 현행화했다. C/D·Gym은 여전히
  미착수이며, 다음은 **B 결과 승인 후 C의 반복 내용 채우기·공개 API 상세계획 작성**이다.
- 원격 push·PR·GitHub 댓글·이슈 close는 수행하지 않았다. 신규 API를 외부 실행 경로로
  연결하거나 다른 문서 가져오기를 이번 종료 검증에 섞지 않았다.
