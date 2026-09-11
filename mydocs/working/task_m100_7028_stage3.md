# #7028 Stage 3 — 제출 전 검증

- 범위: 온전한 분할 표 셀 및 반복 제목 셀의 대각선 방출. 빈 문단 텍스트는 후속
  [#7032](https://github.com/edwardkim/rhwp/issues/7032)로 분리한다.
- 승인: 메인테이너 대각선 확인 및 완료 절차 진행 지시.
- 최초 제품·테스트 후보: `1ae766455adeeaf9ee4779fa310de08dc2882bda`.
- 최종 검증 후보: `4592264c007a4f6616f38d7fda30784f7936fc51`.
- 메인 작업 branch: `task_m100_7028`.
- 검증 worktree: `/home/edward/mygithub/rhwp-review-7028` (동일 제품·테스트 후보, detached).
- target: `/home/edward/mygithub/rhwp-shared-review-target` (기존 공유 cache 유지).
- 검증 경로: 내부 변경의 PR 제출 전 검증. 정본 `pr_review_workflow.md` 및
  `pr_review/local_validation.md` 4.3, 시각 증적은 `pr_review/visual_fixture_evidence.md`.
  아직 PR 번호·정식 self-review 없음. 원격 push·PR·병합 승인으로 확대하지 않는다.

## 실행 순서

Docker WASM → review worktree prepare·fmt → native Clippy → WASM Clippy → workspace build
→ workspace all-target Clippy → manifest check → 전체 release-test nextest → Native Skia lib·PNG·PDF.
같은 공유 target의 Cargo는 순차로 실행한다. 호스트 16 논리 CPU / RAM 31GiB, 시작 시 가용 약
28GiB, 활성 Cargo 없음. 전체 nextest는 전수 래칫의 내부 worker를 고려해 8 threads를 사용한다.

새 sample·baseline 변경은 없으며 별도 보안 sweep 신규 입력 대상은 없다. Stage 2 focused 29개와
실물 6쪽 원장을 재사용한다. 실행 결과는 명령 종료를 확인한 뒤 이 문서에 반영한다.

## 결과

- 후속 이슈 #7032 등록·metadata·한글 본문 API 확인 완료.
- 최초 후보 Docker WASM 성공: Rust compile 4분 03초, 총 6분 53초. 최적화 포함 표준 서비스다.
- 최신 `upstream/devel` `c378fbe3c7`에서 추가된 9개 commit을 확인했다. 문서·Studio·문서 코어·테스트
  변경이 포함되므로 기존 후보의 결과를 통합 후보 전체 검증으로 재사용하지 않는다.
- 사전 merge simulation은 오늘할일 끝에 추가된 서로 다른 작업 기록만 충돌했다. 원격 lpaiu-cs
  기록과 로컬 #7028 기록을 모두 보존해 실제 병합 `4592264c00` 완료. 소스 충돌/임의 보정 없음.
- review worktree를 해당 후보로 전환했다. fmt·native Clippy·WASM32 Clippy·workspace build·
  workspace all-target Clippy·manifest check 모두 통과했다. fmt 후 tracked source 변화 없음.
- 전체 nextest: **9,481/9,481 통과, 기존 skip 46개**, 실행 시간 360.869초. 컴파일을 포함한
  명령 전체는 17:42:45~17:53:36 KST다. `--tests --test-threads 8 --no-fail-fast` 사용.
  IR sweep 등 전수 래칫을 제외하거나 baseline을 수정하지 않았다.
- nextest 0.9.137/권장 0.9.140 차이에 따른 버전 및 JUnit 옵션 경고는 있었지만 실행은 성공했다.
- Native Skia lib: root 3,930개, 내부 crate 15+165+2개, 합계 **4,112개 통과** / 기존 ignore 13개.
- Native Skia PNG: **2/2 통과**. 직접 PDF: **4/4 통과**. 필터로 제외한 다른 suite 검사는
  통과 수에 포함하지 않는다. 전체 회귀의 제목행 선택 4개·신규 대각선 9개도 통과했다.
- 검증 스크립트 전체 exit 0. review worktree tracked 변경 없음. 로그는
  `output/7028/stage3-validation.log`와 `output/7028/stage3/` 아래 gate별 `.log`다.
- 통합 후보 Docker WASM 재빌드 성공: Rust compile 4분 09초, 최적화를 포함해 총 7분 02초.
  증적은 `output/7028/stage3/wasm.log`다.
- 최종 WASM SHA-256: `9ddc53befe22db3cba060e6a69944ea6b560b7107143cb1aea07ff67ce97c9a5`.
- Studio 자동 확인 1차에서 캐시를 끈 HTTP WASM과 로컬 산출물의 해시 일치, 문서 6쪽 로드,
  첫 쪽 대각선 1개까지 확인했다. 다음 `svg.includes('직렬')` 검사는 실패했다. SVG 태그 분리를
  고려하지 않은 문자열 검사이므로 글자 누락으로 판정하지 않고 XML 텍스트 조회로 보정했다.
- 재확인에서는 Chrome 새 탭 응답이 지연됐다. 따라서 **최종 브라우저 6쪽 기하 검증과 화면
  캡처는 통과로 기록하지 않는다.** 메인테이너의 대각선 확인과 native 6쪽 증적은 별도로 유효하다.
  사용자 기존 탭·브라우저를 재시작하지 않는다. `output/7028/studio-check.mjs`는 로컬 확인용이며
  제품 변경이나 회귀 suite가 아니다.
- 위 결과는 로컬 검증이다. 원격 push·PR 생성·CI·병합·이슈 종료는 아직 수행하지 않았다.
