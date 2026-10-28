# #6852 Stage 3 — 최소 수정의 최종 검증

- Issue: [#6852](https://github.com/edwardkim/rhwp/issues/6852).
- 2026-09-08 메인테이너의 후속 절차 승인에 따라 시작했다.
- 상태: **최종 후보의 Rust·Skia·Docker WASM 검증 완료. PR·원격 push·merge·close는 아직 수행하지 않았다.**
- 계획: [수행계획](../plans/task_m100_6852.md), 선행 증거: [Stage 2](task_m100_6852_stage2.md).

## 범위와 기준선

일반 흰 사각형에 적용되던 경험적 선 억제 B 제거만 검증한다. 5쪽의 메인테이너 시각 판정은 통과했다.
IR 구조 보존과 글상자 A 조건 재검토는 [#6856](https://github.com/edwardkim/rhwp/issues/6856)으로
분리했으며 이번 구현에 추가하지 않는다.

단계 변경 전에 수용 범위·후속 계획 문서 3개를 `d93d2dc9b`로 커밋했다.
제품·시험 내용은 선행 검증 후보 `9f4f451b7`과 같다.
`git fetch upstream devel`로 확인한 원격은 `7138fe7848a0cda157bce33f73c9d9d7e977209a`다.
`HEAD...upstream/devel`은 7/0이고 merge-tree도 충돌 없이 통과했다. 불필요한 merge commit은 만들지 않았다.

내부 타스크의 PR 사전 준비는 `docs_and_git_workflow.md`와 `local_validation.md` 4.3,
renderer/fixture는 `visual_fixture_evidence.md`를 적용한다. 외부 contributor PR 검토가 아니며
기존 GitHub Full CI 재사용 예외는 적용하지 않는다.

## 실행 환경과 계획

- 기존 `rhwp-review-6852` worktree를 깨끗한 상태에서 `d93d2dc9b` detached로 전환했다.
- 기존 고정 Cargo target `rhwp-6812-review-target`를 재사용하며 이동·삭제하지 않았다.
- 시작 시 16 logical CPU, RAM 31 GiB 중 available 24 GiB, 디스크 여유 135 GiB였다.
  다른 Cargo/Rust 빌드가 없음을 확인했다. nextest는 기본 동시성을 사용한다.
- prepare → fmt → native Clippy → WASM32 Clippy → workspace build → workspace all-target Clippy →
  manifest → release-test 전체 nextest → Native Skia 3종 순으로 실행한다. 한 단계라도 실패하면 멈춘다.
- 신규 sample 보안 입력은 `RHWP_SECURITY_SWEEP_SAMPLES_JSON`에
  `["samples/hwpx/156160455-social-pig-farm-income.hwpx"]`를 명시했다.
- 전체 회귀에 포함된 자동 탐색 래칫 4종과 oracle 범위를 판정한다.
  새 기준 PDF는 추가하지 않았고, clipping은 controlset 입력의 실제 존재 여부와 구분해 보고한다.
- Docker 최적화 WASM 및 출력 확인은 Rust 검증 뒤 순차 수행한다. 기존 Studio 서버는 유지한다.

실행 스크립트·원시 로그는 로컬 `output/6852/stage3/`에 보존한다. generated suite·manifest,
실행 로그와 중간 산출물은 source PR에 포함하지 않는다. 최종 결과는 실행 종료 후 기록한다.

## 1차 결과와 신규 sample 원장 정정

`d93d2dc9b`에서 prepare, fmt, 세 Clippy, workspace build, manifest 검사는 통과했다.
전체 nextest는 compile 4분 09초, test 330.013초, 합계 약 579초에 종료했다.
**9,224 실행 / 9,223 PASS / 1 FAIL / 46 skipped**이며 실패는 IR field sweep 한 건이다.
신규 HWPX를 명시적으로 전달한 `new_sample_documents_are_clean_across_all_three_detectors`와
#6852 회귀 6건은 통과했다. 이후 Skia·WASM은 이 실패를 확인하기 위해 진행하지 않았다.

실패는 신규 `samples/hwpx/156160455-social-pig-farm-income.hwpx`의 HWPX 왕복이다.
수정 전 제품 `ee794278f`를 같은 review worktree·고정 target에서 locked release-test CLI로
대조 빌드했다(1분 45초). 수정 후 CLI도 별도로 보존해 같은 원본을 각각 `export-hwpx`로 저장하고
`ir-sweep --json`으로 전수 대조했다. 원본을 덮어쓰지 않았다.

| 정규화 필드 경로 (`hwpx` lane) | 수정 전 | 수정 후 |
| --- | ---: | ---: |
| `sections[].paragraphs[].controls[].cells[].paragraphs[].controls[].paragraphs[].raw_header_extra[]` | 1 | 1 |
| `sections[].paragraphs[].controls[].cells[].paragraphs[].raw_header_extra[]` | 403 | 403 |
| `sections[].paragraphs[].controls[].paragraphs[].raw_header_extra[]` | 10 | 10 |
| `sections[].paragraphs[].raw_header_extra[]` | 212 | 212 |

합계 626건의 경로·원본값·재생성값이 전후 모두 동일하다. 626건 전부의 상세 값도 확보했으며
`raw_header_extra`의 6·7·9번 바이트, 즉 HWPX 문단 ID를 담는 6..10 영역에만 차이가 있다.
`src/parser/hwpx/section.rs`는 원본 `hp:p/@id`를 이 영역에 보존하고,
`src/serializer/hwpx/context.rs::next_para_id` 및 section serializer는 문서 전역 ID를 순차 발급한다.
파서·직렬화기·IR sweep 구현은 수정 전후 동일하며, 재생성 HWPX를 다시 왕복한 결과는 **IR 차이 0**이다.
따라서 선 수정으로 증가한 손실이 아니라 기존 문단 ID 재부여의 신규 fixture 관측값으로 분류했다.
이 사실은 모든 외부 소비자에서 문단 ID 변경이 무해하다는 보증을 뜻하지 않는다.

승인된 신규 sample 원장 처리 범위에서 위 네 행만 `tests/fixtures/ir_field_sweep_baseline.tsv`에 추가한다.
다른 sample의 임계값이나 원장·탐지기 정책은 바꾸지 않는다. 입력 SHA-256은
`3194188fb93047684c560f346d13015f31bf77a4d1ca02eed6ba3a19f0fa9f53`이다.
증거는 `ir-before.json`, `ir-after.json`, `ir-after-second.json` 및 대응 HWPX에 보존했다.
수정한 원장으로 재검증하기 전에는 전체 검증 통과로 표시하지 않는다.

clipping controlset 92개 원본은 이 checkout에 0개 존재하며 신규 sample도 그 목록에 없다.
따라서 clipping gate 실행 성공으로 주장하지 않는다. 새 PDF를 추가하지 않아 oracle 원장 재생성도
이번 입력 추가 범위에 해당하지 않는다. 나머지 자동 탐색 래칫은 전체 회귀 결과로 확인한다.

실행 환경 참고: nextest 0.9.137은 최소 요구 버전을 충족하지만 권장 0.9.140보다 낮아
JUnit `report-skipped` 키 경고가 났다. 테스트 실패와 별개이며 도구 버전을 임의 변경하지 않았다.
진단 중 worktree 상대경로로 주 checkout의 JSON을 읽으려던 조회 1회는 실패했으며,
주 checkout 경로에서 다시 조회해 위 결과를 확정했다. 제품·시험 실행의 실패와 구분한다.

## 2차 검증 최종 결과

네 원장 행과 대조 증거를 `393401e5a`로 커밋하고 같은 review worktree에서 재검증했다.
제품 코드는 추가 변경하지 않았다. `output/6852/stage3/pass2/` 및 `status-pass2.log`에
1차 기록과 분리하여 명령별 종료 결과를 보존했다.

| 게이트 | 결과 |
| --- | --- |
| prepare, fmt 적용·check, native Clippy, WASM32 Clippy, workspace build, all-target Clippy, manifest | 전부 PASS |
| 전체 nextest | 9,224 PASS / 0 FAIL / 46 skipped, test 323.175초, compile 포함 약 565초 |
| 신규 sample 보안 | 실제 HWPX 1개를 전달한 검사 PASS |
| IR field sweep | 125.957초 PASS, 1차 실패 해소 |
| Native Skia lib | root 3,930 + workspace member 15·165·2 = 4,112 PASS / 13 ignored |
| Native Skia 그림 | 2 PASS, 필터 제외 169 |
| Native Skia 직접 PDF | 4 PASS, 필터 제외 167 |
| diff check | PASS |
| Docker 최적화 WASM | exit 0, wasm-pack 보고 6분 42초, Rust compile 3분 50초 포함 |

실행한 Docker 명령은 주 checkout의 `docker compose --env-file .env.docker run --rm wasm`이다.
기존 named volume과 `.env.docker`를 사용했다. 새 WASM SHA-256:
`f64fdc47e0847cde8adb2b7044c22ab281ab3dd6bade9411e8f405f9b8423bd0`.

새 WASM을 Node에서 직접 로드하여 두 원본의 11쪽 유지, 5쪽 앞쪽 실선 존재를 확인했다.
각 문서의 1·5·7쪽 SVG는 기존 후보 CLI 출력과 모두 바이트 동일하다. 5쪽 출력은 이미 메인테이너가
시각 수용한 SVG와 같은 hash다. 이는 새 브라우저 시각 판정을 받았다는 주장이 아니다.

기존 7700 서버의 HTML은 HTTP 200이고, Vite의 실제 `/@fs/.../pkg/rhwp_bg.wasm` 경로로 받은
파일이 WASM 형식이며 로컬 산출물과 hash까지 동일함을 확인했다. 서버는 종료·재시작하지 않았다.
초기 `/pkg/` 조회는 SPA HTML fallback이었으므로 검증으로 인정하지 않고 alias의 실제 경로로 정정했다.
WASM 진단 선택자도 처음에는 x좌표만 사용해 같은 쪽 아래 도형까지 2개를 셌다. 대상 y좌표를 포함해
정확한 원본 그룹을 선택한 뒤 재실행했다. 테스트 기대값이나 제품 코드를 바꾼 것이 아니다.

마지막 fetch에서도 원격 devel은 `7138fe784`이며 후보와의 merge-tree는 clean이다(8/0).
이후 작성되는 결과보고 commit은 문서 전용이다. 원격 게시 전에는 head와 devel을 다시 확인한다.
검증 worktree는 `393401e5a`에서 tracked 변경 없이 깨끗하며, 파생 파일은 source에 stage하지 않았다.
