# #3587 Stage 25 — 제출 증적 보존·최신 devel 통합 검증

- 승인: 메인테이너의 “남은 순서대로 진행하세요”.
- 시작 HEAD: `53818ad66`, 브랜치 `task_m100_3587`.
- 범위: 최종 검증 입력·한컴 기준 PDF 보존, 최신 devel 통합, 필수 제출 검증.
- 제외: 새로운 제품 기능, 별도 Studio 이슈 수정, 원격 push·PR 생성·댓글·병합·이슈 종료.
- 검증용 기존 `rhwp-review-3587` overlay와 고정 `target/pr-review`를 보존한다.
- host: WSL2 Linux x86_64, Rust 1.93.1, nextest 0.9.137, Node 24.15.0, Python 3.12.3.
  nextest 권장판 0.9.140 알림과 미지원 `report-skipped` JUnit 설정 경고가 있었으며,
  테스트 실패나 CI-duration 보고서 생성 성공으로 해석하지 않는다. 이번은 default profile 실행이다.

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

최종 검증은 완료했다. 최초 실패와 독립 대조·보완 후 성공을 아래에서 구분한다.

## 통합

- fetch: `upstream/devel=1ae5ca295bddcb31b846affc62834a2a3023d24d`.
- 작업 브랜치에 merge commit `ca67b5ff53f3fc170e3e49096e531bbd423db54a`로 통합했다. 충돌 없음.
- 기존 review overlay와 untracked 진단 seed는 stash
  `a9015e72c0b75a1f8a571e10db0dd2f251ded0a3`에 보존했다. 삭제하거나 주 작업 트리에 적용하지 않았다.
- 검증 worktree는 과거 overlay 대신 통합 후보의 실제 detached HEAD로 전환했다.
- PDF repository policy: 1,248개 검사, 크기 상한·LFS pointer 없음 PASS.

## 신규 PDF 쪽수 보호

정본 선택기의 `choose_canonical`과 `engine_for_product`로 신규 PDF 5개의 원본 형식·경로·
저장 제품 엔진 대응을 확인했다. 모아찍기 입력은 없다. `pdfinfo`의 독립 PDF 쪽수는
pi2 HWP/HWPX 각 1쪽, pi4 HWP/HWPX 각 1쪽, 연구노트 원본 2쪽이다.
통합 후보 native `rhwp info --json`의 쪽수도 각각 1/1/1/1/2로 일치했다.
전체 기존 원장을 재생성하지 않고 이 5개 신규 행만 oracle_page_count 원장에 추가했다.
기존 문서의 허용치를 늘리거나 불명 회귀를 등록하지 않는다. 파일 SHA는 MANIFEST를 따른다.

## Rust lint

제품 후보 `451de71f9`: fmt, native/WASM32/workspace all-target Clippy, workspace build,
manifest, unit tier 모두 PASS. 이후 README와 신규 PDF 쪽수 data 외 제품 변경은 없다.
순차 실행 명령·시간·종료 코드는 `output/3587/submission-stage25/lint-results.json`이다.

## 통합 후보 집중·입력·Gym 검증

- 실제 검증 worktree HEAD: `dd9b539bf` (통합 후 fixture/쪽수 등록 포함).
- 집중 nextest: 192 PASS, 0 FAIL. #3587과 인접 식별자·HWP3 서식 보호 계약 포함.
- 신규 검증 입력 41개를 `RHWP_SECURITY_SWEEP_SAMPLES_JSON`으로 지정한 security 검사:
  6 PASS, 0 FAIL. 기본 source 12개와 이번 보존 문서 29개이며 PDF는 이 입력 목록에서 제외한다.
- 기존 `gym-labnote.py`를 통합 후보 release-test CLI로 재실행했다. 양 형식 각 13개 검사와
  무편집/복사 생략/마지막 채우기 생략 대조를 통과했다. 최종 HWP/HWPX SHA는 Stage 24와
  **바이트 단위로 동일**하다. 새 한컴 판정을 추정한 것이 아니라 기존 판정 파일과 동일함을 확인했다.
- 실행 명령·결과: `output/3587/submission-stage25/{focused,security}-results.json`,
  `output/3587/submission-stage25/gym/{identity,result}.json`.

## 기준 PDF 비교 — 자동 점수와 사람 판정 구분

통합 native debug CLI로 연구노트 원본 2쪽과 pi4 HWPX 가져오기 1쪽의 SVG/레이아웃을
다시 내보내고 기존 정식 한컴 PDF와 표준 fidelity/visual_sweep 방식으로 비교했다.
누락 페이지는 없다. pi4의 SVG 파일 번호 004는 원본 쪽 번호이며 PDF의 실제 1쪽과 대응한다.
Linux Chrome이 없어 `rsvg` rasterizer를 명시했다. Studio webfont/브라우저 검증으로 계산하지 않는다.

- 연구노트 2쪽: 표 사이 간격·선의 세로 위치 차이를 관찰했다. 내용 픽셀 보조 일치율 0.39%.
- pi4 글상자: 내용은 유지되지만 배경·선·글자 위치 차이를 관찰했다. 보조 일치율 64.48%.
- 자동 flagged 수는 두 입력 모두 0이지만 **시각 일치 PASS의 근거가 아니다**.
  이미 분리한 Studio 문제와 자동화 저장 계약을 구분하며, 이 관측만으로 신규 회귀나 해결을 단정하지 않는다.
- 증적은 `output/3587/submission-stage25/visual-{labnote,textbox}/`의 compare/overlay/review와
  `fidelity-{labnote,textbox}/`의 전체 SVG·layout ledger에 있다.
- 높을수록 내용 픽셀 위치·형태가 더 비슷하고 낮을수록 검토가 필요하다. 사람 판정 정확도는 아니다.

## 편집 API·UI 검증 경계

`edit_command_review_checklist.md`를 확인했다. 이번 공개 자동화 호출은 native/WASM/CLI/MCP
문서 API이며 새로운 Studio UI command나 Undo/Redo 라우터를 추가하지 않는다.
따라서 UI history·selection/caret·IME E2E를 구현 완료 항목으로 주장하지 않는다.
대신 공개 API 계약, 실제 WASM, 저장·재열기, 메인테이너의 한컴 정상 판정을 근거로 삼는다.

문단 블록 commit은 raw section stream을 무효화하고 section vpos 재계산 → section recompose →
pagination 순으로 실행한다. 외부 가져오기는 새 binary가 있을 때 epoch를 갱신하고 resolved styles를
재구축한 뒤 같은 commit 경로를 사용한다. 기존 clipboard 식별자 영향은 집중/전체 회귀에 포함한다.
이는 별도 Studio 이슈 #7065/#7084/#7090의 UI 재편집 해결을 뜻하지 않는다.

## 전체 회귀 1차 — 신규 body-overflow 원장 확인

nextest run `ae487679-9737-4e20-a01f-488c9edd0a72`: **9,727 PASS / 2 FAIL / 51 skipped**,
368.512초. 실패는 `body_overflow_baseline` partition 1/5의 신규 두 파일 각각 1건뿐이다.
IR·셀 overflow·off-canvas·text-overlap·oracle 쪽수 등 다른 검사는 통과했다.
`body_overflow`는 본문 하단 축을 검사하는 추가 래칫이며 기존 여섯 축 설명만으로 빠뜨리지 않는다.

같은 review worktree를 수정 없는 devel `1ae5ca295bddcb31b846affc62834a2a3023d24d`로
detach하고 `cargo build --locked --profile release-test --target-dir
/home/edward/mygithub/rhwp/target/pr-review --bin rhwp`로 독립 대조 바이너리를 만들었다.
후보와 base 각각 아래 세 파일에 `layout-anomaly <파일> --overflow-tolerance 2 --json`을 실행했다.
대조 시 sample byte는 동일하고 실제 바이너리 SHA 및 JSON은 `overflow-{base,candidate}.json`에 있다.

| 입력 | base → 후보 하단 넘침 | 실제 쪽(1 기준) | 판정 |
| --- | --- | --- | --- |
| `samples/hwp_table_test.hwp` | 1 → 1 | 2쪽 | 기존 원장 등록 문서 |
| `samples/issue3587/b-table-repeated.hwp` | 1 → 1 | 3쪽 | 같은 기존 넘침의 신규 보존 경로 |
| `samples/issue3587/b-table-repeated.hwpx` | 1 → 1 | 3쪽 | 같은 기존 넘침의 신규 보존 경로 |

모두 `Page/Body/Column0/Table9`, bbox `(x=113.3867,y=871.1467,w=555.36,h=142.76)`,
`overBottom=4.786666666666406px`로 동일하다. 원본의 넘치는 표가 복제 삽입 뒤 한 쪽 뒤로
이동했으며 같은 입력에 대한 base/candidate 기하는 같았다. 페이지 수는 원본 3, 복제본 각 4다.
용지 밖 넘침/글자 겹침은 세 파일 모두 0이다. 본문 여백 넘침이 해결되었다는 의미는 아니다.

- HWP SHA: `24e3c9d4709b70988671e95934c9f530b8bc414ef8f129bb7a185616d4f0b122`.
- HWPX SHA: `7c271c8532483d480c87f72d5ac4d505e6cb1fb70d0862ab76a5b230ac1cde4b`.
- base binary SHA: `9feadfd71915d7ffaa44ca86483a0a6a1d8e84b349f11679afefbd426eee804e`.
- candidate binary SHA: `50258741aa98c10ccd21dd718e570aec3a7027497d60a9b9b68b60b4e57243bf`.

따라서 두 **신규 경로만 각 1건** 원장에 등록한다. 기존 수치나 공차·검사 입력을 완화하지 않는다.
이후 후보로 돌아가 필수 lint·전체 회귀를 다시 수행한다. 1차 실패 기록은 보존한다.
대형 표 #2063은 273.132초에 완료되어 161쪽 pin을 통과했다. 이전 D3 174.527초와 실행 조건·
통합 source가 달라 이 두 시간만으로 제품 성능 증감을 단정하지 않는다.

## 원장 보완 후 전체 재검증

후보 `6a8aeb9ff`에서 필수 lint 묶음을 다시 순차 실행해 전부 통과했다
(`r2/lint-results.json`). 이전 후보 대비 Rust 제품·테스트 소스 차이는 없고
`tests/fixtures/body_overflow_baseline.tsv`의 신규 2행만 검증 데이터 변경이다.

전체 nextest run `cfafe6ae-ac41-4391-8adf-9887e2a19b90`:
**9,729 PASS / 0 FAIL / 51 skipped**, 실행 385.181초, 재빌드 포함 명령 691.856초.
앞선 실패 partition 1/5도 모두 통과했다. 테스트 제외를 추가하거나 입력을 빼지 않았다.
명령과 원문 로그는 `output/3587/submission-stage25/r2/regression{.log,-results.json}`에 있다.
이번에는 IR·overflow-cell·off-canvas·text-overlap뿐 아니라 body-overflow 16분할 dump도 남겼다.
전체 원장을 재생성하지 않았으며 다른 래칫 허용치는 그대로다.

Native Skia 3종은 전부 통과했다. root lib 3,930 PASS/13 ignored, 내부 crate 182 PASS,
그림 placeholder 2 PASS, 직접 PDF 4 PASS다. 명령별 결과는 `skia-results.json`이다.
2026-09-13 마지막 native 검증 후 `git ls-remote upstream refs/heads/devel`로 원격이
통합 기준 `1ae5ca295bddcb31b846affc62834a2a3023d24d` 그대로임을 재확인했다.
Docker 표준 `docker compose --env-file .env.docker run --rm wasm`도 PASS다.
Rust compile 4m19s, wasm-pack 전체 7m15s, Docker 명령 전체 461.742초이며 최적화를 생략하지 않았다.
실제 `pkg/`를 Node에서 로드한 가져오기 계약은 1.439초에 PASS:
HWP/파생 HWPX 두 입력, dry-run/실행 동일 결과, 잘못된 JSON/범위 등 오류 시 무변경,
JS 동일 핸들 사전 거부, count=0 무변경, HWP/HWPX 저장·재열기를 확인했다.
브라우저 UI나 새로운 한컴 시각 판정을 수행한 것으로 보고하지 않는다.

- WASM SHA-256: `c31a359626b096d5f38d44fe6b1ae9a7e58093655ed4bb6c698c969a3d3e854b`.
- JS SHA-256: `a6b6e1564d302881e7db02fd1a1b4864e65acc4b0ea737517240ac60e2fa708a`.
- 증적: `wasm-results.json`, `wasm-contract-results.json`, `wasm-contract/result.json`.

## 최종 판정과 제출 경계

승인된 로컬 제출 준비를 완료했다. 제품·테스트·원장 검증 기준은
`6a8aeb9ffac5d574e7a590ce66aba5eae9f7f374`이며 이후 변경은 보고·계획·fixture 안내 Markdown뿐이다.
정식 보존 입력 73개 대응/37개 고유 파일은 MANIFEST SHA와 실제 Git blob 바이트가 일치한다.
변경 Markdown 38개 링크 검사와 `git diff --check`, `git diff --check upstream/devel...HEAD`도 통과했다.
파생 suite/manifest, pkg/output, Cargo registry/lock, workflow는 source 제출에 추가하지 않는다.
외부 clipping controlset은 이 실행에서 검사하지 않았으며 통과로 주장하지 않는다.

PR 본문 초안은 `output/3587/submission-stage25/pr-body.md`에 준비한다.
다음은 **별도 승인 후 원격 push → devel 대상 Open PR → 트리야지·CI·self-review**다.
현재 원격 push·PR 생성·댓글·병합·이슈 close는 수행하지 않았다.
기존 review worktree는 제출 검증용으로 유지하고, 이전 overlay stash도 복구 가능하게 보존한다.
