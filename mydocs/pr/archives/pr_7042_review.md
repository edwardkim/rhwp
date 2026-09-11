---
kind: record
status: archived
canonical: mydocs/pr/archives/pr_7042_review.md
last_verified: 2026-09-12
---

# PR #7042 검토

## 병합 및 후속 기록

- 메인테이너의 merge commit 병합 및 후속 절차 승인에 따라 2026-09-12 02:14 KST 병합했다.
- 원 코드 merge SHA: `2f41565ceca649003d94a714a98321314773edf2`.
- 기여자 원 head `9c7a8c427a9285ee01e5ff01f6b71e44c561c0c0`를 고정한 admin merge를 사용했다.
- 아래 원격 상태/미게시 설명은 각 검토 시점 기록이다. 원 PR의 최종 상태는 MERGED다.
- 새 MCP 기준 PDF를 포함하므로 maintainer 문서 직접 push 대신 후속 기록 PR로 보존한다.
  문서·대표 PNG·신규 PDF뿐이며 제품 코드·test·golden·baseline은 변경하지 않는다.
- 기록 준비 시 #7017은 OPEN이었다. 기록 반영·devel 동기화 뒤 해결 comment와 close를 수행한다.
- 후속 기록 PR 자체의 self-review: 원 PR 검증 결과/해시/시각 판정을 대조했고 허용 경로 4개만 포함한다.
  신규 PDF는 검토 증적이며 이번 기록 PR에서 테스트 원장을 변경하지 않는다.
  새 PDF의 oracle page-count 원장 자동 등록은 수행하지 않았으므로 자동 보호 추가를 주장하지 않는다.
- contributor fork branch는 보존한다. 리뷰 worktree는 증적 반영 및 필수 후속 처리 완료 후 제거하고,
  고정 공유 review cache와 다른 PR의 worktree·Studio 환경은 유지한다.

## 접수 및 판정

- PR: https://github.com/edwardkim/rhwp/pull/7042 — 관련 이슈 #7017, 본문 `closes #7017`.
- 작성자: planet6897, 기존 기여자. Reviewer: edwardkim 지정.
- head: `9c7a8c427a9285ee01e5ff01f6b71e44c561c0c0`.
- 최신 조회 base: `upstream/devel`, `d14f44eba7035a5604acf9c236b81a294722691f`.
- 경로: maintainer_general + intake_and_review + local_validation + visual_fixture_evidence.
- 최종 리뷰 판정: **승인**. MCP 기준 PDF 직접 비교, 메인테이너 시각 판정, 로컬 집중 검증
  18건 및 Docker WASM/Chrome 검증을 통과했다. 정확한 code head의 Full CI도 성공했다.
  공통 측정 경로의 0폭 예외 제거는 수용 가능하며, 아래 잔존 주석 정정은 비차단 권고다.
  이는 GitHub approve 게시 또는 병합 승인이 아니다. 원격 게시·병합은 별도 승인 후 진행한다.

원격 상태는 OPEN/non-Draft/MERGEABLE. 댓글, GitHub review event, push, merge, close는 수행하지 않았다.

### 메인테이너 시각 판정

2026-09-12, 기준 PDF·PR 비교 결과와 원본 샘플 경로를 제공한 뒤 메인테이너가
“시각 판정은 통과입니다.”라고 확인했다. PR #7042의 점선 정렬·잘림 개선에 대한 시각 게이트를
통과로 기록한다. 확인에 사용한 앱이나 WASM 버전은 별도로 지정되지 않았으므로 추정하지 않는다.
이 판정은 전체 회귀 테스트 통과, 원격 게시 또는 병합 승인으로 확대하지 않는다.

## 변경의 성격: 공통 측정 규칙 복구

`src/renderer/layout/text_measurement.rs`의 `char_width_decision`에서 `U+F081C`를
`U+FFFC`와 함께 0폭으로 취급하던 예외를 제거한다. 오브젝트 자리표시자 `U+FFFC`의 0폭 규칙은 유지한다.
문서명, 문단 번호, 표 개수 등으로 대상을 골라내는 조건은 추가하지 않는다.

`hancom_pua.rs`는 원문 `U+F081C`를 출력 시 `┈`로 투영한다. 실제 그리는 점선에는 전진 폭이
있는데 가운데 정렬의 측정에서만 0으로 제외했던 불일치를 해소하는 방향이다.
따라서 #7040/#7044와 같은 문서 속성 조합을 추가하는 보정으로 분류하지 않는다.

다만 이번 PR이 모든 PUA의 측정·출력 정책을 하나로 통합하는 것은 아니다.
측정 함수는 여전히 원래 문자의 임베디드 폰트 메트릭을 먼저 확인하고, 없으면 이 샘플에서
`heuristicHalfwidth` 경로를 사용한다. 모든 폰트 환경에서 무조건 0.5em이라고 일반화하지 않는다.

### 원인 계보와 유지되는 보호

0폭 예외는 `1c74d5ab5efdbeb546b25fea36e52039e87f5fc9`의 #677 처리에서 도입되었다.
당시 TAC의 이중 y 누적·PUA 폭·워터마크 문제가 함께 다뤄졌다. 이후 측정 공통화와 trace 작업을
거쳐 예외가 유지되었다. 이번 PR은 그중 폭 예외만 제거하며 TAC y 처리나 워터마크 정책은 바꾸지 않는다.
과거 수정이었다는 이유만으로 유지하거나 제거하지 않고, 기존 복학원서도 함께 확인했다.

## 직접 실행한 재현

현재 devel의 기존 Docker WASM으로 before를 생성하고, PR head를 native CLI로 직접 빌드해 after를 생성했다.
현재 Studio용 `pkg`와 dev 서버는 교체하지 않았다.

| 신규 접수증 1쪽 | devel WASM | PR native |
|---|---:|---:|
| 출력 점선 글리프 수 | 60 | 60 |
| 첫 점선 x | 354.8533px | 145.8533px |
| 마지막 점선의 시작 x | 849.4867px | 640.4867px |
| 문서 쪽수 | 1 | 1 |

페이지 오른쪽 약 793.3px를 넘던 증상은 해소된다. 마지막 수치는 **글리프 시작 좌표**이며
잉크의 오른쪽 끝 좌표가 아니다. 신규 테스트도 마지막 `<text x>`를 검사하므로 이 구분이 필요하다.
기여자 제시 한컴 시작 좌표 145.4px와는 가깝다. 최초에는 PDF가 없었으나 아래 MCP 보완에서
독립 생성한 한컴 PDF의 첫 점선 ink-left 145.422px도 확인했다.

기존 `samples/복학원서.hwp`도 전후 1쪽이다. 접수증의 `접` 시작 x가 403.42→398.2867px로
약 5.13px 왼쪽 이동했다. 기존 한컴 PDF와 전후 표준 비교 PNG를 실제 열어 확인했다.
접수증의 수평 정렬 보정은 확인되지만, 기존 로고 표시·폰트·세로 위치 등 잔여 차이가 있으므로
페이지 전체가 한컴과 일치한다고 판정하지 않는다.

이 결과는 제품 WASM과 PR native 사이의 비교다. 동일 backend에서 parent/head만 바꾼
단일변수 실험, PR WASM 검증 또는 메인테이너 시각 판정 통과로 표현하지 않는다.

## 기준 자료 및 산출물

- 신규 원문: `samples/issue7017/2528375-science-invention-contest-receipt.hwp`.
  SHA-256 `20f18916495e7fb815928ec73fd73a193b76a9ce6b575cefe0dd0515804d1da7`.
  마지막 저장 정보: hancom-office-2010 8.5.8.1677.
- 기존 원문: `samples/복학원서.hwp`.
  SHA-256 `da81b4010331bcac290f900c7cf224c97ee8355399614725ce46c197ff1a22a4`.
- 재사용 PDF: `pdf/복학원서-hwp-2020.pdf`, 새로 복사하거나 생성하지 않음.
  Creator `Hwp 2022 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`, PDF 1.6, 1쪽, 595×841pt.
  파일명의 engine bucket과 실제 Creator를 구분한다.
  SHA-256 `ed28f2655a27d22acdbf214520115522ad3124a3643d18fba58beebba1935068`.
  SHA-1 `d0e4119ff2a7b7591d4de3ecdc382db47fecaba9`.
- 신규 원문의 한컴 PDF: PR/이슈 첨부는 없었으나 아래 MCP 보완으로 확보함.
- 로컬 산출물 루트: `/home/edward/mygithub/rhwp/output/7042/`.
  `baseline-receipt/`, `after-receipt/`의 SVG;
  `baseline-bokhak/cmp-p001.png`, `after-bokhak/cmp-p001.png`의 비교 패널.
- 비교는 `tools/fidelity_compare`의 `pdf_to_png`, `svg_to_png`, `sheet`를 호출했다.
  한컴 PDF/렌더 나란히 보기 1쪽씩 확인이며 전체 visual sweep이나 pixel 지표 검증은 아니다.
  after 캡처 첫 시도는 Chrome exit -5였고 도구의 재시도에서 성공했다. 제품 실패로 집계하지 않는다.

## 최초 검토 시점의 검증 상태와 남은 절차

아래는 MCP 보완 및 후속 로컬 집중 검증 전의 기록이다. 현재 결과는 마지막 후속 검증 절을 따른다.

1. `git diff --check` 성공, 최신 devel과 `git merge-tree --write-tree` 충돌 없음.
2. `cargo build --locked --profile release-test --bin rhwp --target-dir /home/edward/mygithub/rhwp-shared-review-target` 성공(2분 06초).
   바이너리 SHA-256 `1453f8a43c53cbcdad65c5d00a015cef5427f62265cef87b591b6ab52b5f892c`.
3. CLI `export-svg <sample> -p 0 --font-style -o <output>`를 두 샘플에 실행, 모두 exit 0.
4. 정확한 head의 CI [34611036304](https://github.com/edwardkim/rhwp/actions/runs/34611036304) 성공 확인.
   Lint/Build & Test/CodeQL/Canvas visual diff 성공. WASM Build는 skipped이며 성공으로 세지 않는다.
5. 로컬 집중 integration test·전체 회귀·Clippy 묶음·PR Docker WASM은 아직 실행하지 않았다.
   이번 검토에서는 코드를 수정하지 않았으며, 원격 CI 성공과 로컬 재실행을 구분한다.
6. 신규 샘플의 MCP 기준 PDF와 PR native 출력의 직접 대조를 완료했다. PDF 재요청은 필요 없다.
   메인테이너 시각 판정도 통과했다. 남은 로컬 집중 검증과 병합 승인 절차를 구분해 진행한다.

## 메인테이너 요청에 따른 MCP 보완

2026-09-12, 매뉴얼 `mydocs/manual/mcp_hwp2024Convert_usage.md`를 따라
정식 client 0.9.0의 `start → status → download`를 실행했다.
사용 가능한 서비스로 보완하기 전에 PDF 미첨부를 보류 사유로 제시한 초기 검토를 정정한다.

- 입력: 위 신규 원문, HWP5 5.0.3.4, 14,336bytes, 1쪽 예상.
- 마지막 저장 제품 2010에 따라 `--engine 2020`, `--timeout-seconds 600` 명시.
- Job: `1922bbfc-9afe-4248-8e00-3139ac735f3e`.
- `queued → succeeded → success`; start/status/download engine 모두 `2020`.
- 실제 `engine_profile: 2020`, `hancom_version: 12.0.0.4605`, worker 32bit,
  backend `hwp-managed-direct-dll-host`, 입력 전처리 없음.
- 변환 약 15초. 서버 font_scope verified, failed_font_count 0.
- PDF: `pdf/2528375-science-invention-contest-receipt-2020.pdf` — review worktree에 저장.
- 60,153bytes, client/server/local SHA-256 모두
  `ee0f1312284ce04c683b69cbdfd70157760ff42ea57a2a864a1d3aaf93ff8e3b`.
- SHA-1: `bc5d58d429d98b8b5bc8becac32b42f665ac3b99`.
- pdfinfo: Creator `Hwp 2022 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`,
  PDF 1.6, A4 595×841pt, 1쪽, 비암호화, JavaScript 없음.

`output/7042/compare.py after-receipt <MCP PDF 절대 경로>`로 프로젝트
fidelity_compare의 비교 패널을 만들고 `after-receipt/cmp-p001.png`를 실제 열었다.
점선은 한컴 PDF와 PR 출력 모두 두 접수증 사이 중앙에 있으며 우측 용지 밖으로 잘리지 않는다.
기여자의 핵심 개선 주장은 독립 한컴 출력으로 확인되었다.

PDFium text layer에서 점선 60자는 U+F000으로 추출된다. 같은 코드로 추출되는 별도 인장 1자는
다른 y에 있으므로 제외했다. 96dpi 환산 점선 ink-left 145.422px, 마지막 ink-right 647.748px.
PR SVG 첫 글리프 origin x=145.853px와 약 0.43px 차이다. ink bbox와 origin의 차이를
고려해야 하므로 이를 정확한 동일 좌표 또는 glyph 전진폭의 직접 측정이라고 부르지 않는다.

폰트 굵기와 인장 표시 등 다른 차이는 비교에 남아 있다. 점선 문제 확인을 문서 전체 fidelity
통과로 확대하지 않으며 메인테이너 최종 시각 판정을 대신하지 않는다.
이 산출은 로컬 보완 검증이고, 기준 PDF·리뷰 문서를 커밋하거나 원격 게시하지 않았다.

### 문서 정정 권고

`src/renderer/composer.rs:3341`에 여전히 “0폭 규칙”, “실제 출력에서는 숨긴다”라는 설명이 남아 있다.
상단의 수정된 주석 및 실제 renderer의 점선 투영과 구분되도록, `expand_pua_display_text`라는
특정 helper의 동작과 실제 측정/paint 경로를 정확히 설명해야 한다.
주석 오류를 새로운 렌더링 회귀로 주장하거나 이번 범위를 PUA 전면 재설계로 확대하지 않는다.

## 후속 로컬 검증 — 2026-09-12

`git fetch upstream devel` 뒤 base와 head가 위 SHA에서 변경되지 않았음을 확인했다.
`git merge-tree --write-tree upstream/devel HEAD`는 exit 0,
tree `01888dad94df074b3ed621a685608167699ea992`로 충돌 없이 종료했다.
제품 코드·기여자의 테스트·baseline은 수정하지 않았다.
추가 PDF/PNG는 리뷰 증적으로만 로컬에 두었고 source candidate에 반영하지 않았다.
아래 검증과 CI 재사용의 대상은 원 PR의 정확한 code head다. 이후 source/fixture/asset을
포함한 별도 commit을 만들 경우 그 새 head의 검증 범위를 다시 결정해야 한다.

| 검사 | 결과 |
|---|---|
| integration suite `--prepare`, `--check` | 성공; 원본 1,259개, 28 suites + 20 exceptions |
| `rust-unit-test-tiers.mjs --check` | 성공; 기존 테스트 4,205개 |
| `cargo fmt --all -- --check` / `git diff --check` | 성공 |
| 신규 #7017 점선 회귀 | 1/1 통과, 188개 필터 제외; build 2분 17초, 실행 0.014초 |
| 폰트 측정 coverage + SVG snapshot + 신규 sample 보안 | 17/17 통과, 518개 필터 제외; build 11.92초, 실행 3.264초 |

집중 검증 명령(worktree=`rhwp-review-7042`, 표기 target은 실제 고정 공유 target):

```bash
node scripts/run-rust-test.mjs issue_7017_pua_dotted_line_measure_width -- \
  --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp-shared-review-target
RHWP_SECURITY_SWEEP_SAMPLES_JSON='["samples/issue7017/2528375-science-invention-contest-receipt.hwp"]' \
  cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp-shared-review-target \
  --test regression_suite_020 --test regression_suite_007 --test regression_suite_016 --no-fail-fast \
  -E 'test(/issue_4962_font_metric_coverage::/) | test(/svg_snapshot::/) | test(/new_sample_documents_are_clean_across_all_three_detectors/)'
```

suite 번호는 이번 `--prepare`의 결과이며 향후 고정 번호로 재사용하지 않는다.
17건에는 복학원서 #677 golden, 동일 프로세스 반복 SVG 결정성, 신규 원문 1개를 실제 전달한
세 보안 탐지기 검사가 포함된다. golden 갱신 없이 통과했다.
로컬 nextest 0.9.137이 권장 0.9.140보다 낮고 `report-skipped` 설정을 무시한다는 경고가 있었다.
default profile의 선택 테스트가 실제 실행됨을 확인했고 경고를 제품 실패로 세지 않았다.

원 PR head의 Full CI `34611036304` 성공과 clean merge에 근거해 매뉴얼 3.2.2/4.3.0에 따라
광범위 전체 nextest·Native Skia와 동일 head의 lint는 재사용했다. 로컬에서 재실행해 통과했다고
기록하지 않는다. WASM은 별도로 리뷰 worktree에서 Docker 표준 경로로 검증한다.

### Docker WASM 및 실제 Chrome 검증 완료

- review worktree에 예제 `.env.docker`를 준비한 뒤
  `docker compose -p rhwp --env-file .env.docker run --rm --no-deps wasm` 실행, exit 0.
  기존 Docker image/named volume을 재사용하되 `/app` bind mount는 `rhwp-review-7042`임을 확인했다.
- `release` compile 4분 08초, wasm-pack/wasm-opt 포함 7분 01초.
- PR WASM: `pkg/rhwp_bg.wasm`, SHA-256
  `e873a8859784cd3aa8cddf528cbff18a36874bdd8a57cbf33fb9fc73ffb4539a`.
- `node output/7042/verify-pr-wasm.mjs`: Node에서 실제 WASM을 로드해 두 원문을 각각 열고
  SVG를 생성했다. 두 문서의 점선 x 배열이 앞서 native CLI로 만든 배열과 완전히 일치했다.
- `node output/7042/verify-browser.mjs`: loopback 임시 HTTP 서버에서 PR의 JS/WASM과
  지정된 두 원문만 제공했다. Chrome/146.0.7680.31에서 실제 WASM 모듈을 로드하고
  `HwpDocument → renderPageSvg → DOMParser` 경로로 검증, exit 0.
- 신규 접수증: 1쪽, 점선 60개, 첫 origin x=145.85333333333338,
  마지막 origin x=640.4866666666659. 한컴 시작값 근접 및 우측 용지 범위 검사 통과.
- 복학원서: 1쪽, 점선 2개, origin x=56.693333333333335/63.36.
- 임시 브라우저와 HTTP 서버는 검사 후 종료했다. Studio UI의 메뉴/편집 여정까지 실행한 것은 아니다.
- 기본 작업트리 `pkg/rhwp_bg.wasm` SHA-256은
  `f0d34fd4c82ed07ed0a59250460c21786e59d131c826c5d40eb20bfc0be777b5`로 유지됐다.
  메인테이너가 사용 중인 Studio dev 서버나 WASM을 바꾸지 않았다.

마지막 원격 재조회에서 head 동일, OPEN/CLEAN, 실패·진행 중 check 없음.
`git diff --check` 성공, 추적 파일 변경 없음. 파생 suite/manifest는 stage하지 않았고
추적 상태에 복원할 변경도 없다. 미추적 추가는 리뷰 문서·대표 PNG·MCP 기준 PDF 3개뿐이다.

### 최종 시각 증적 준비

- fidelity text/layout ledger: `output/7042/receipt-ledger/`, 요청/완료/누락 = 1/1/0.
  body-footnote, table-footer, table/image-frame, square-wrap, table-cell-text-overlap 후보 모두 0.
- 표준 sweep: `scripts/visual_sweep.py --key pr7042-receipt --hwp <원문> --pdf <MCP PDF>
  --page 1 --rhwp-bin <위 release-test CLI> --out output/7042/sweep`.
  `VISUAL_SWEEP_CHROME`은 로컬 Chrome 146 경로를 명시, Studio webfont projection 사용.
- 1쪽/1쪽, flagged=0/1, pixel match=94.75081%, visual_accuracy_proxy_percent=13.13587%.
- 출력 파일명의 `2528375`는 샘플 파일명에서 추출된 식별값이다. **물리적으로는 PDF/SVG 각 1쪽**이며
  도구가 단일 산출물 1:1 fallback으로 대응했다. 2,528,375쪽이라는 뜻이 아니다.
- 임시 compare: `/home/edward/mygithub/rhwp/output/7042/sweep/pr7042-receipt/compare/compare_2528375.png`.
- 임시 overlay: `/home/edward/mygithub/rhwp/output/7042/sweep/pr7042-receipt/overlay/overlay_2528375.png`.
- 임시 review: `/home/edward/mygithub/rhwp/output/7042/sweep/pr7042-receipt/review/review_2528375.png`.
- review PNG를 실제 열었다. 한글 본문·하단 지표는 판독 가능하며 우상단 긴 진단 label의 끝은 잘린다.
  이를 제품 clipping으로 오인하지 않는다. 최종 대표 asset은 label이 짧은 compare 패널을 선택했다.
- 대표 보존 경로: `mydocs/pr/assets/pr_7042_receipt_p1_compare.png` (로컬 복사, 미커밋).
  낮은 내용 픽셀 일치율은 폰트·선 두께·미세 좌표 차이도 포함하는 보조값이다.
  이번 점선 정렬/잘림의 메인테이너 통과 판정과 문서 전체 fidelity 판정을 구분한다.

### Merge 후 contributor PR comment 계획

병합 승인 및 실제 병합 이후에만 실행한다. 이번에는 원격 게시하지 않는다.

1. 위 1쪽·flagged 0/1·pixel match 94.75081%·내용 픽셀 보조값 13.13587%와 한계를 명시한다.
2. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 인용한다.
3. 대표 asset이 저장된 실제 commit SHA로 다음 URL을 고정한다.
   `https://raw.githubusercontent.com/edwardkim/rhwp/<asset-commit-sha>/mydocs/pr/assets/pr_7042_receipt_p1_compare.png`.
   아직 commit이 없으므로 URL을 실제 존재하는 링크처럼 게시하지 않는다.
4. 승인된 증적 반영 경로에서 문서·대표 PNG·기준 PDF를 보존한 뒤 UTF-8 `--body-file`로 게시하고,
   API 재조회로 본문·한글·URL을 확인한다. output 중간 SVG/JSON/로그는 커밋하지 않는다.
