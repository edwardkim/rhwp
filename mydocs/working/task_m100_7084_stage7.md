# #7084 Stage 7 — 최종 결과 승인·제출 전 통합 점검

- Issue: #7084
- 승인: 2026-09-14 「승인합니다」 — 최종 보고서 승인 및 최신 devel 확인·PR 준비 진입.
- 점검 head: `82948e6f389345da835915ed6d72d512a1843663`.
- 제품/테스트: `230f801136a4fc73725b847fe56c49b2c11c88bd`.
- 원격 push·PR 생성·댓글·병합·close 승인은 포함하지 않는다.

## 1. 선택한 절차

- base route: `collaborator_self_merge` — 내부 타스크의 self PR 준비, 아직 PR 번호 없음.
- modifiers: `intake_and_review`, `local_validation`, `visual_fixture_evidence`,
  `rework_and_exceptions`(누적 변경 1,000줄 초과).
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서.
- PR 번호를 예측해 review 문서를 만들지 않는다. 번호 기반 self-review와 오늘할일 갱신은
  별도 승인된 push·Open PR 생성 뒤 같은 PR의 후속 기록으로 한다.

## 2. 최신 devel 확인 결과

작업 트리가 깨끗한 상태에서 `git fetch upstream devel`을 수행했다.

- 기존 공통 base: `70bf40af2a2818e72bd58b4fa66e2d4c06de2b51`.
- 최신 원격 devel: `11860a9f4186438a3cb9c355e3cd8d2257162d3a`.
- `git rev-list --left-right --count HEAD...upstream/devel`: **25 / 42**.
  42는 원격에만 있는 merge·문서 커밋을 포함한 수이며 PR 42개라는 뜻이 아니다.
- 주요 유입: #6984 하이퍼링크 편집·PDF 보존, #7110 TAC 줄 소속/저장 위치,
  #7108 PDF 합성 굵게.
- `.github/` workflow 변경 없음. `scripts/` 차이는
  `scripts/tests/font_rule_projection_gen.test.mjs`이며 폰트 규칙 보정의 검사 변경이다.
- 로컬 devel과 작업 브랜치의 포인터는 변경하지 않았다. fetch를 로컬 devel 동기화 완료라고 쓰지 않는다.

`git merge-tree --write-tree upstream/devel HEAD`는 **종료 코드 1**이다.
결과 tree `94bcbdf760eb173c68c0a46d65faf77954ef96dc`에는 충돌 표식이 있으며 검증 후보가 아니다.
실제 checkout/index에 merge를 시작하지 않았고, 사용자 작업·WASM·공유 target을 변경하지 않았다.

| 충돌 파일 | 확인한 내용 | 제안하는 최소 해결 |
| --- | --- | --- |
| `src/wasm_api.rs` | 같은 삽입 위치의 `mod hyperlink;`와 `mod canvas_metrics;` | 두 모듈 선언 모두 유지 |
| `mydocs/orders/20260913.md` | #6936/#7108 기록과 #7084 기록의 같은 위치 삽입 | 양쪽 section을 독립적으로 유지 |
| `mydocs/orders/20260914.md` | 양쪽 신규 파일의 #7096/#7110 기록과 #7084 기록 | 날짜 제목 하나, 양쪽 section 내용 보존 |

`composer.rs`, `typeset.rs`, `layout.rs`, 코어 rendering과 Studio bridge 등은 텍스트 자동 병합됐다.
하지만 **자동 병합은 동작 검증이 아니다.** 특히 하이퍼링크의 SVG/PDF layer tree 경로와
#7084 portable 측정 문맥 복원, TAC의 측정/배치와 보충 메트릭 전달을 통합본에서 확인해야 한다.
현재 실행으로 새 회귀를 검출한 것은 아니며, 통합 검증 필요 지점이다.

## 3. 권고하는 다음 절편 — 해결 방침 승인 요청

1. 현재 `task_m100_7084`와 기존 검증 worktree를 유지한다. 새 고아 브랜치나 재작성 이력을 만들지 않는다.
   실행 직전 원격이 더 바뀌었는지 확인하고, 새 충돌·영향 범위가 있으면 다시 분리해 보고한다.
2. 최신 devel을 작업 브랜치에 일반 merge하고 위 3곳만 양쪽 내용을 보존해 해결한다.
   의미상 다른 제품 충돌이 발견되면 추정 구현으로 덮지 않는다.
3. 기존 `rhwp-rust-review-7084-c`의 clean 상태/이번 소유 파생물만 확인해 통합 후보로 갱신하고,
   고정 `/home/edward/mygithub/rhwp/target/pr-review`를 사용한다. target 이동·삭제는 하지 않는다.
4. 집중 메트릭/세션·기존 shaping 보호 → Rust 전체 회귀·Native Skia 3종·세 Clippy와 workspace
   build → Docker WASM·TypeScript/npm·실제 Chrome → 기본 Render Diff/직접 PDF 순서로 검증한다.
   공유 Cargo는 순차 실행한다. 실패하면 원인을 분류하고 이전 head의 통과로 대체하지 않는다.
5. 새 통합본에서 원본 이모지, VS16/ZWJ 경계, 자동 복구와 portable 출력 복원을 재확인한다.
   하이퍼링크/TAC 관련 기존 집중 검사도 병합 변경에 맞춰 선택한다. 기존 시각 판정은 과거 후보의
   유효한 증거로 보존하되 통합본 판정과 구분한다.
6. 검증 입력 보존을 마감한다. 원본 두 파일은 이미 `samples/issue3587/`에 추적되어 있다.
   한컴 기준 PDF는 기존 추적 파일과 내용 중복 여부를 확인한 후 누락분만 `pdf/` 정식 경로에
   보존하고 관련 manifest/쪽수 원장을 검사한다. PDF 재변환·신규 HWP/HWPX 추가는 필요하지 않다.
   대표 최종 이미지만 안정 경로에 보존하며 output 로그·JSON·폰트 bytes는 제출하지 않는다.
7. 실제 검증한 입력과 후보 commit의 hash를 대조하고 PR 본문 초안·제출 파일 목록을 확정한다.
   이후 remote push·Open PR 생성 승인을 요청한다. PR 채번 전 review 파일명은 만들지 않는다.

이 계획은 필수 통합 검증을 승인된 제품 범위 안에서 수행하기 위한 것이다. 폰트 기능 확대나
별도 이슈 생성을 추가하지 않는다. 새 소스가 통합되므로 기존 Stage 5/6 전체 결과를 통합본에
그대로 재사용할 수 없다. 이번 점검에서는 긴 테스트·빌드를 시작하지 않았다.

## 4. 승인 전 점검 상태 (과거 기록)

- 최종 보고서 승인 기록을 반영했다.
- fetch·정확한 SHA 비교·충돌 시뮬레이션과 변경 영향 점검을 완료했다.
- **PR 준비 완료는 아니다.** 실제 충돌 해결 방침 승인과 통합본 검증이 남았다.
- 제품 수정·실제 merge·원격 write는 하지 않았다. 문서 점검 결과만 로컬 커밋으로 보존한다.

## 5. 승인 후 실행 — 통합 및 기준 PDF 보존

해결 방침 승인 「네」 후 다시 fetch했다. devel은 `11860a9f4`로 동일했다.
일반 merge `27b28af13`에서 위 3곳을 양쪽 보존으로 해결했다. 기존 review worktree를 같은
commit으로 갱신했고, fmt 및 #7084 session 집중 11건이 통과했다.
이후 전체 게이트는 기준 PDF 보존·쪽수 원장 추가를 포함한 후보로 실행한다.

기존 `output/7084/oracle/` PDF 2개는 추적 중인 `pdf/`·`samples/` PDF와 크기/내용 hash를
대조했으며 같은 bytes의 파일이 없었다. 재변환 없이 다음 경로에 복사해 `d3a189d5d`에 커밋했다.

| 원본 (기존 Git 파일) | PDF 정식 경로 | PDF SHA-256 |
| --- | --- | --- |
| `samples/issue3587/c-form-labnote-001-stage11-filled.hwp` | `pdf/issue3587/c-form-labnote-001-stage11-filled-hwp-2020.pdf` | `ba1d5fbe6af800453143d8a7a16733339a25e3943764cb3d7f1e090fa820f83a` |
| `samples/issue3587/c-form-labnote-001-stage11-filled.hwpx` | `pdf/issue3587/c-form-labnote-001-stage11-filled-hwpx-2020.pdf` | `0d4dadfd9523991c722bbbdeb83641c26426820beb80f139409a2c5488125b86` |

PDF는 각 24,211/24,215 bytes, 2쪽, 595×841pt, Creator/Producer `Hancom PDF 1.3.0.550`이다.
출처는 Stage 1의 MCP engine/profile 2020 변환이며 실제 서버 Hancom 버전은 **12.0.0.4605**였다.
파일명 2020은 서비스 engine 선택을 뜻하며 실제 제품 버전을 11.x로 바꾸어 해석하지 않는다.
원본 저장 제품은 HWP null, HWPX `hancom-office-2020`이며 기존 변환 선택을 유지한다.

`regenerate.py`의 `rhwp_info`와 `pick_oracles`를 이번 두 입력에 한정해 호출했다.
통합본 native CLI는 각각 2쪽·모아찍기 false였고, 형식별 위 PDF를 정확히 선택했다.
독립 PDF의 2쪽을 기대값으로 기존 쪽수 원장에 **2/2 신규 행 두 개만** 추가했다.
기존 행이나 허용치는 변경하지 않고 관련 원장 검사는 전체 integration에 포함해 재실행한다.
두 원본은 새 sample이 아니며, 바이너리 입력은 변경하지 않았다.

## 6. 통합 후보 전체 검증 완료

검증 후보는 **`d3a189d5d964e4e6eb3b376062df846744c4c58d`**, 포함된 devel은 `11860a9f4`다.
Rust는 기존 `/home/edward/mygithub/rhwp-rust-review-7084-c`를 같은 SHA로 갱신해 실행했고,
고정 target `/home/edward/mygithub/rhwp/target/pr-review`를 재사용했다.
새 worktree·branch·target은 만들지 않았다. 아래 로그는 `output/7084/stage7/` 기준이다.

| 검사 | 결과 | 로그 |
| --- | --- | --- |
| session 단독 집중 | 11 PASS, merge `27b28af13`에서 실행 | `focused-session.log` |
| 메트릭·shaping·하이퍼링크·TAC·PDF 합성 굵게 집중 | 84 PASS | `focused.log` |
| release-test lib | 4,055 PASS / 13 ignored | `lib.log` |
| nextest 전체 `--tests` (lib와 integration 포함) | **9,814 PASS / 51 skipped / 0 FAIL**, 358.987초 | `integration.log` |
| Native Skia lib / 그림 / 직접 PDF 집중 | 4,112 PASS / 13 ignored; 2 PASS; 4 PASS | `native-skia-lib.log`, `native-picture.log`, `native-pdf.log` |
| fmt, native·WASM32·workspace all-target Clippy, workspace build | 모두 PASS, 세 Clippy `-D warnings` | `fmt.log`, `clippy-*.log`, `workspace-build.log` |
| doctest | 8 PASS / 3 ignored | `doctest.log` |
| integration manifest / source unit-tier | PASS, 48/48 targets / 4,205 test 분류 | `manifest.log`, `unit-tier.log` |
| 기준 PDF 저장 정책 | PASS, 1,265 PDFs, LFS pointer 없음 | `pdf-policy.log` |
| 통합된 폰트 projection 생성기 검사 | 14 PASS | `font-rule-projection.log` |
| Docker WASM | PASS, 전체 7분 24초 (Rust compile 4분 24초 포함) | `docker-wasm.log` |
| TypeScript / Studio production build | PASS / PASS | `typescript.log`, `production-build.log` |
| Studio 전체 npm test | **1,692 PASS / 2 skipped / 0 FAIL** | `npm-test.log` |
| E2E manifest | 134 tracked / 134 등록, PASS | `e2e-manifest.log` |
| 실제 Windows Chrome 원본·서식·portable 복원 | HWP/HWPX PASS; 장평 64%, 첨자, 120% 굵게/기울임 PASS | `connected/runtime.json` |
| VS16/ZWJ 서식 경계 | 두 형식 × 두 종류 PASS, 부분 요청·적용 없음 | `run-boundary.json` |
| Canvas 오류 복구 | 두 형식 × 6조건 **12/12 PASS**, 70 assertion | `browser-recovery.json`, `browser-recovery.log` |
| renderer backend 계약 / Undo 계약 | PASS / 24 assertion PASS | `renderer-contract.log`, `undo-contract.log` |
| 기본 Render Diff | **3/3 PASS**, 0.01593% / 0% / 0% | `render-diff-results.json` |
| 직접 PDF compatibility gate | **3/3 PASS**, 1.158954% / 0.390370% / 0.676720%, 각각 2% 이하 | `direct-pdf-results.json` |

nextest `--tests`의 9,814에는 lib가 포함된다. lib·집중 검사를 합산한 수를 고유 테스트 수로
보고하지 않는다. 최종 fmt 실행도 review tree에 tracked 변경을 만들지 않았다.

Rust 재현은 review tree에서 `node scripts/rust-test-suite-manifest.mjs --prepare` 후
`cargo test --locked --profile release-test --lib --target-dir <고정 target>`와
`cargo nextest run --locked --cargo-profile release-test --target-dir <고정 target> --tests --no-fail-fast`다.
집중 filter는 `issue_7084_`, `issue_4969_shaping_emitted_run_mapping`,
`issue_4969_shaping_atomic_activation`, `issue_6963_hyperlink_`, `issue_7103_tac_table_rewind`,
`issue_6936_pdf_synthetic_bold`를 OR로 선택했다. Native Skia 3종과 세 Clippy는
`pr_review/local_validation.md` 4.3의 명령을 같은 target에서 순차 실행했다.

WASM은 기본 checkout에서 `docker compose --env-file .env.docker run --rm wasm`으로 만들었다.
Studio에서 `npx tsc --noEmit`, `npm test`, `npm run build`, `npm run e2e:manifest-check`를 실행했다.
브라우저 복구 검사는 추적된 `canvas-metric-recovery.test.mjs`와 assertion이 동일한 로컬 사본으로
실행했다. 변경은 helper import 경로와 증적 출력 Stage 6 → Stage 7 두 곳뿐이다. 과거 증적을
덮어쓰지 않기 위한 경로 변경이며, `CHROME_CDP=http://localhost:19222`를 사용했다.
이번에는 음성 대조를 재실행하지 않았다. Stage 6의 음성 대조는 해당 후보의 과거 증적으로 남긴다.
renderer·Undo·Render Diff는 각각 추적된 E2E에 `--mode=headless`를 사용했다.
직접 PDF는 `RHWP_RENDER_DIFF_DIRECT_PDF=1`, `..._GATE=1`, `..._MAX_RATIO=0.02`,
`..._RASTER_DPI=144`, `RHWP_RENDER_DIFF_RHWP_BIN=<고정 target>/release-test/rhwp`로 실행했다.

기존 경고는 별도로 남긴다. nextest 0.9.137/권고 0.9.140 및 미지원 JUnit 설정 경고,
Vite의 CanvasKit `fs/path` 외부화·큰 chunk 경고가 있었다. PDF report-only 비교는 72 DPI
크기 차이로 **4 warnings / 0 errors**였으며, 직접 PDF gate 3/3 통과와 혼동하지 않는다.
이번 시간 수치는 단일 실행이며 빌드 또는 제품 성능의 인과 비교가 아니다.

## 7. 통합 WASM·시각 증거·입력 동일성

- WASM: **11,182,083 bytes**, SHA-256
  `61465d69c436028b5f5c8db93297a92fb7283cb58450ce0525b5b342943c1b9a`.
- 실제 Chrome `152.0.7977.83`이 HTTP 200으로 받은 WASM과 디스크 hash가 일치했다.
  새 하이퍼링크 등 devel 변경이 포함돼 이전 WASM과 bytes가 다른 것이 정상이다.
- 두 원본의 이모지 자연 진행폭은 18.302703857px, 활성 paint 배율은 1/1이다.
  같은 세션에서 비활성 대조는 기존 가로 압축을 재현했고 페이지 수는 각각 2쪽이었다.
- 통합본 `connected/{hwp,hwpx}-after.png`는 Stage 6의 각각 대응 PNG와 byte-identical하다.
  네 파일 모두 SHA-256 `5a502fcd29d83ffa10e59ad7055b8835a03de4174bc7beaa3259e6eb6a2167f8`.
  따라서 기존 시각 승인 화면이 보존됐다. 자동 비교를 새로운 인간 판정으로 쓰지 않는다.

표준 시각 비교는 `scripts/visual_sweep.py`의 `make_compares`, `make_overlay_compares`,
`make_review_panels`를 재사용했다. portable SVG 대신 실제 Canvas PNG 2쪽을 사용하고,
기준 PDF 2쪽은 `pdftoppm -f 2 -l 2 -singlefile -r 96 -png`로 래스터화했다.
기존과 같은 pixel threshold 32를 유지했다.

- `output/7084/stage7/visual/{hwp,hwpx}/compare/compare_002.png`
- `output/7084/stage7/visual/{hwp,hwpx}/overlay/overlay_002.png`
- `output/7084/stage7/visual/{hwp,hwpx}/review/review_002.png`

형식별 1쪽, 합계 2쪽을 비교했다. 구조 후보 탐지 전수 sweep을 실행한 것이 아니므로 자동 후보
0건이라고 보고하지 않는다. 두 형식 모두 pixel match **90.78317%**, 내용 픽셀 중심 보조값
**6.68869%**다. 대표 HWP review PNG를 직접 열어 한글 label·footer·수치가 판독 가능함을
확인했다. 이모지는 좁게 눌리지 않지만, 표 간격·글꼴 및 컬러/흑백 모양 차이는 남아 있다.
전체 fidelity 통과가 아니라 **#7084의 가로 압축 해소와 기존 승인 화면 유지**가 판정 범위다.

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 6.69%.
높을수록 좋음: 기준 PDF와 같은 위치·색의 내용 픽셀이 많다는 뜻이다.
낮을수록 다름: 표 위치·줄 위치·글꼴 모양 차이도 함께 반영된다.
단, 사람 판정 정확도가 아니라 픽셀 기반 보조값이며 이번 이모지 성공률이 아니다.

실행 원본 HWP/HWPX 및 기준 PDF의 bytes를 `git show d3a189d5d:<경로>`와 대조해 네 파일 모두
일치했다. 원본 SHA-256은 HWP `c8bae9677a6b4cdf6d7e79996523d7bbe04c31e61bdc8d04ef286d308ec74832`,
HWPX `3999f78c928f709d9ac93b9efa03001168fe00935da78594242480f8816fcce4`다.
PDF SHA-256은 §5를 따른다. 두 PDF 모두 PDF 1.6이며 SHA-1은 각각
`b143b6154f036b37903801168ef56b6eba0c21f5`, `420f1785b2e70e7a2ce91a87b9e3f3160c60f8eb`다.

## 8. 제출 준비 상태와 다음 승인 경계

승인된 통합 및 로컬 검증을 완료했다. 병합 충돌 외 새 제품 보정은 없었고,
검사·golden·래칫 허용치를 완화하지 않았다. source/test/PDF 입력은 `d3a189d5d`로 고정하고
뒤따르는 결과 문서-only commit과 구별한다.

PR 초안은 `output/7084/stage7/pr-body.md`에 준비한다. 제출 대상은 제품·테스트 원본,
계획/단계/결과 문서, 기준 PDF 두 개와 쪽수 원장 신규 두 행이다.
generated suite·manifest, pkg/WASM/dist, 로컬 폰트·접속정보·전체 output 증적은 제출하지 않는다.
PR 번호는 예측하지 않는다. 별도 승인으로 Open PR을 만든 뒤 실제 번호에 맞춘 self-review와
대표 PNG의 `mydocs/pr/assets/` 안정 파일명 보존·트리야지를 같은 PR에서 처리한다.
1,000줄 초과 변경이므로 코드 review·시각 근거·CI와 작업지시자 판단을 건너뛰어 merge하지 않는다.

원격 push·PR 생성·댓글·merge·close는 **미실행**이다. 다음 승인 요청은 원격 작업 branch
push 및 base `devel`의 Open PR 생성이다. 기존 review worktree는 그 검증·self-review에 사용 중이다.

## 용어

- merge simulation: 작업 트리를 바꾸지 않고 두 commit의 병합 결과·충돌을 계산하는 검사.
- head / base: 제출 후보의 끝 commit / 비교·병합 대상 기준 commit.
- portable: 브라우저 전용 측정에 의존하지 않는 출력 문맥.
- TAC (Treat As Character): 객체를 글자처럼 글줄 흐름에 참여시키는 속성.
- CI (Continuous Integration): 지속적 통합 검사. 로컬 통과와 원격 최신 head 통과는 별도다.
