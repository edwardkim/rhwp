# #3587 Stage 25 — 제출 증적 보존·최신 devel 통합 검증

- 승인: 메인테이너의 “남은 순서대로 진행하세요”.
- 시작 HEAD: `53818ad66`, 브랜치 `task_m100_3587`.
- 범위: 최종 검증 입력·한컴 기준 PDF 보존, 최신 devel 통합, 필수 제출 검증.
- 제외: 새로운 제품 기능, 별도 Studio 이슈 수정, 원격 push·PR 생성·댓글·병합·이슈 종료.
- 검증용 기존 `rhwp-review-3587` overlay와 고정 `target/pr-review`를 보존한다.

## 증적 선정

최종 B/C/D 및 Gym에서 실제 저장·재열기·비교한 파일과 파생 입력을 보존한다.
바이트가 같은 원본/중간/최종 파일은 한 파일을 재사용하고 MANIFEST에 원래 경로를 기록한다.
한컴 판정이 없는 자동 계약 입력은 한컴 정상 자료로 분류하지 않는다.
잘못된 용지 설정으로 폐기한 Stage 17 산출, 실패한 Gym 조사 실행, 반복 비용 측정 출력,
PNG/SVG/JSON/로그는 정식 sample로 추가하지 않는다.

`samples/issue3587/MANIFEST.json`은 출처·역할·SHA·기존 파일 재사용 대응표다.
새 기준 PDF는 `pdf/issue3587/`에서 대응 sample과 같은 stem 및 형식/2020 태그를 쓴다.
원본 연구노트 PDF는 `pdf/rnote/`에 보존하며 새 한컴 변환을 요청하지 않는다.

## 진행 상태

증적 73개 출처를 SHA로 중복 제거했다. 고유 파일 37개 중 기존 3개를 재사용하고
34개(문서 29개, PDF 5개)를 정식 경로에 새로 보존했다. source 파일은 이동·덮어쓰기하지 않았다.
초기 사본 `b-table-original.hwp`는 기존 `samples/hwp_table_test.hwp`와 SHA가 같아
MANIFEST를 기존 파일 재사용으로 정정하고 중복 사본만 제거했다(이전 commit에도 복구 가능).
MANIFEST의 각 `path`는 repository 상대 경로이며 `source`는 기존 로컬 증적 위치다.
새 파일은 일반 Git blob으로 보존하고 LFS filter가 없음을 확인했다.

현재 검증은 진행 중이다. 검사 종료 전에는 통과로 기록하지 않는다.

## 통합

- fetch: `upstream/devel=1ae5ca295bddcb31b846affc62834a2a3023d24d`.
- 작업 브랜치에 merge commit `ca67b5ff53f3fc170e3e49096e531bbd423db54a`로 통합했다. 충돌 없음.
- 기존 review overlay와 untracked 진단 seed는 stash
  `a9015e72c0b75a1f8a571e10db0dd2f251ded0a3`에 보존했다. 삭제하거나 주 작업 트리에 적용하지 않았다.
- 검증 worktree는 과거 overlay 대신 통합 후보의 실제 detached HEAD로 전환한다.
- PDF repository policy: 1,248개 검사, 크기 상한·LFS pointer 없음 PASS.

## 신규 PDF 쪽수 보호

정본 선택기의 `choose_canonical`과 `engine_for_product`로 신규 PDF 5개의 원본 형식·경로·
저장 제품 엔진 대응을 확인했다. 모아찍기 입력은 없다. `pdfinfo`의 독립 PDF 쪽수는
pi2 HWP/HWPX 각 1쪽, pi4 HWP/HWPX 각 1쪽, 연구노트 원본 2쪽이다.
통합 후보 native `rhwp info --json`의 쪽수도 각각 1/1/1/1/2로 일치했다.
전체 기존 원장을 재생성하지 않고 이 5개 신규 행만 oracle_page_count 원장에 추가한다.
기존 문서의 허용치를 늘리거나 불명 회귀를 등록하지 않는다. 파일 SHA는 MANIFEST를 따른다.

## Rust lint

제품 후보 `451de71f9`: fmt, native/WASM32/workspace all-target Clippy, workspace build,
manifest, unit tier 모두 PASS. 이후 README와 신규 PDF 쪽수 data 외 제품 변경은 없다.
순차 실행 명령·시간·종료 코드는 `output/3587/submission-stage25/lint-results.json`이다.
