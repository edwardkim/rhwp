# #8371 1단계 — 로컬 글꼴과 HWP3 조판 분석

- 작업 기준: `f125c735c7d2d44eaa44b78c70465bc89af45fb1`, 브랜치 `fix/issue8371-installed-hft-font`.
- 상태: Native와 fresh WASM 전쪽 검증 및 Studio 단위 테스트를 수행했다. 전체 Rust 회귀 재검증과 나머지 제출 전 검사가 진행 중이므로 PR 제출 준비 완료가 아니다.
- 원본: `samples/hwp3-sample10.hwp`, SHA-256 `d9ceb35d8abfb73e9afbe349bccb2a986cf552d66ae7f3f34dcfe4d9d385cb48`.
- 독립 기준: 사용자가 한컴 2020에서 원본 그대로 Print한 전체 763쪽 PDF. SHA-256 `310e3251e910ac8c7c6f34d0e50d85c4a0ecbbadd65e69fe4f5071370931a918`, 96,185,197바이트. 지침의 50 MiB 이상 PDF 보존 규칙에 따라 `/home/tsjang/rhwp-local-pdf/issue8371/hwp3-sample10-2020.pdf`에 보존하며 Git과 PR 첨부에서 제외한다. 기준 PDF는 재생성하지 않았다.

## 원인과 공통 처리

- 설치된 딸기와 신명 디나루 OTF는 원본 HFT 이름의 실제 프로그램이다. Native는 실제 paint가 선택한 CFF 프로그램의 glyph advance를 우선 사용하고, 프로그램이나 glyph가 없으면 원래 HFT 폭을 유지한다. 글꼴 파일 자체는 저장소에 추가하지 않는다.
- `#세나루`의 원 이름 프로그램을 먼저 찾고, 확인된 HFT 계열에서만 접두어를 제외한 설치 후보를 다음으로 조회한다. 실제 공급된 face만 선택하며, 동일한 원본과 기준 PDF에서 23쪽 89.68% → 100.00%, 71쪽 83.47% → 99.75%, 139쪽 83.02% → 100.00%를 확인했다.
- 원본 구역의 첫 빈 줄 감추기 설정과 한컴 저장 HWPX의 `hideFirstEmptyLine=1`을 확인했다. 숨김으로 물리 공간이 달라진 뒤 저장된 자동 쪽 경계를 텍스트 fit으로 재판정하고 명시 경계와 개체 문단은 유지한다. 실제 쪽 진입 후에도 빈 줄을 검사하고, 조건을 충족한 문단 내부 저장 경계도 같은 물리 예산을 사용한다. 312~314쪽과 692·724쪽은 기존 90% 미만에서 각각 99.94% 이상으로 개선됐다.
- Windows에서 가져온 설치 OTF의 저작권 파일을 추가하거나 글꼴 바이트를 저장소에 포함하지 않는다. 브라우저에는 사용자가 허용한 실제 프로그램만 공급한다.
- 로컬 글꼴 감지·초기화·목록·현재 문서의 재조판은 Studio 세션과 문서의 실제 공급 상태에 연결한다. 감지 권한이 없으면 해당 바이트를 자동 공급하지 않는다.
- Visual Sweep의 WASM 경로는 준비된 기본 HFT 프로그램과 CFF를 감지하고 Studio와 같은 Canvas 메트릭 세션을 사용한다. 연결 그림 `oracle.gif`, `rdb02.gif`, `s1.jpg`는 해시 확인 후 원본을 주입하며 비교 영역을 제외하지 않는다.

## 현재 검증

- Native: 2026-10-30 KST, 96 dpi, `--compat 2022`, 같은 원본·기준 PDF. 전체 763쪽의 raw 실루엣 최저 97.21%(66쪽), 90% 미만 0쪽, 미대응 픽셀 0인 쪽 311쪽. 원문 TSV는 `output/issue8371/auto-hft-full-native/sample10-auto-hft-full/silhouette.tsv`, 로그는 `output/issue8371/auto-hft-full-native.log`에 있다. 66쪽에는 글머리표 크기·간격 차이가 남아 있어 직접 판독 결과를 별도로 기록한다.
- WASM: OTF 공급 이전의 전쪽 비교는 41쪽이 90% 미만이었다. OTF 프로그램과 공통 메트릭 공급을 구현한 새 WASM 빌드 및 재출력은 진행 중이다. 이전 WASM TSV는 완료 증거가 아니다.
- Studio: Node 하위 프로세스가 샌드박스에서 `EPERM`으로 차단된 실행은 실패로 기록했다. 정상 실행 환경의 전체 Node 테스트는 2,671개 통과, 실패 0개다. 로그는 `output/issue8371/studio-all-tests-unsandboxed.log`에 있다.
- Studio OTF 세션과 Visual Sweep 글꼴 공급 focused 테스트 및 TypeScript 타입 검사가 통과했다. 로그는 각각 `output/issue8371/studio-otf-focused-tests-v2.log`, `output/issue8371/sweep-otf-supply-tests.log`, `output/issue8371/studio-otf-types-final.log`에 있다.
- 작업 산출물과 로그는 `output/issue8371/`에 보존한다. 빌드와 전체 검증은 공용 `target/pr-review`를 순차적으로 사용하며 Cargo 빌드 작업은 8개로 제한한다.

## 남은 일

1. 실제 CFF 프로그램을 공급한 fresh WASM의 전체 쪽수·전쪽 TSV와 90% 미만 쪽 및 직접 판독을 완료한다. 진단용 `--no-opt` 빌드는 최종 검증을 대신하지 않는다.
2. 같은 독립 PDF로 Native와 WASM이 모두 90% 이상인 뒤 실제 변경 경계를 보호하는 관계형 회귀를 추가한다. 완료된 해당 페이지가 양쪽 98% 이상이고 직접 판독까지 끝나면 지침의 ±2px 조건을 검토한다.
3. 영향 문서의 정상 대조군과 전체 Rust·Native Skia·Studio·최적화 WASM 필수 검사를 완료하고 결과를 갱신한다. 미해결 항목이 남으면 PR을 제출하지 않는다.

## 2026-10-30 진단 WASM 추가 확인

- 최적화 생략 진단 빌드에서 한컴 Print PDF와 전쪽을 비교했다. 763쪽 중 90% 미만 0쪽, 최저 66쪽 97.14%, 미대응 픽셀 0인 쪽 438쪽이다. TSV는 `output/issue8371/fresh-wasm-otf-diagnostic-scores/silhouette.tsv`, 출력·연결 그림·글꼴 공급 기록은 `output/issue8371/fresh-wasm-otf-diagnostic/`에 있다.
- 71쪽은 선택한 실제 세나루 OTF와 Canvas 측정 세션을 공통 조판에 공급한 뒤 82.99% → 99.99%가 됐다. 139쪽과 312쪽은 100.00%다. Canvas 측정 없는 기존 결과는 이 수치의 증거로 재사용하지 않는다.
- 66쪽의 글머리표 세 줄에는 Native와 WASM 모두 기준 PDF와 크기·간격 차이가 보인다. 90% 통과만으로 이 차이를 해결했다고 판정하지 않는다.
- 최적화 포함 빌드·해당 산출물의 재출력과 전체 Rust 검증은 계속 진행한다. 진단 빌드의 전쪽 점수는 최종 fresh WASM 검증을 대신하지 않는다.

## 최적화 WASM 전쪽 검증

- 2026-10-30 KST에 공용 `target/pr-review`와 Cargo 작업 8개를 사용해 최적화 포함 WASM 빌드를 완료했다. 빌드 로그는 `output/issue8371/wasm-hft-otf-optimized-build.log`이다.
- 최적화 WASM SHA-256은 `360c16f99234145f823abc6bd1cbdf0e41d0bb60ff243cdbc5f7fbec8e93ff10`, JS SHA-256은 `d516f564d0c822a1936bd893c7850ff39c29ff935ac4f1e058506158ddf1e5e7`이다. `pkg`와 Studio public의 대응 산출물 해시가 각각 일치했다.
- 같은 Print PDF와 실제 연결 그림을 사용한 fresh WASM 전763쪽은 raw 최저 97.14%(66쪽), 90% 미만 0쪽, 미대응 픽셀 0인 쪽 438쪽이다. 312쪽은 100.00%이며 Native에서도 100.00%다. TSV는 `output/issue8371/fresh-wasm-optimized-final-scores/silhouette.tsv`, 실행 기록은 `output/issue8371/fresh-wasm-optimized-final/wasm-export-complete.json`이다.
- 선택한 312쪽과 66쪽은 Native 및 WASM의 review·overlay를 PDF와 직접 대조했다. 312쪽의 선행·후속 문단 소속과 첫 줄의 한컴 PDF 문자 원점은 글꼴에 독립적인 관계형 및 ±1px 검사로 보호한다. 해당 Rust 회귀 검사의 실행 결과와 66쪽 차이 판정은 이어서 기록한다.

## 글꼴 비의존 ±1px 회귀

- 312쪽의 첫 `SQL)` 문단과 선행 쪽 제목, 같은 쪽 후속 본문을 원본 텍스트와 쪽 소속으로 확인한다. 독립 한컴 Print PDF의 첫 글자 원점은 PDFium `FPDFText_GetCharOrigin`으로 읽은 96 dpi 쪽 좌표 `(113.3786, 142.4044)`이며, x와 기준선에 각각 ±1px을 적용한다.
- `tests/cases/issue_8371_hwp3_page_boundary_pdf_origin.rs`는 설치 글꼴 파일을 적재하지 않는 `DocumentCore` 경로를 사용한다. 기본 환경과 빈 Fontconfig 설정 및 `RHWP_FONT_PATH`를 제거한 환경에서 각각 1개 테스트가 통과했다. 로그는 `output/issue8371/font-independent-boundary-nextest.log`, `output/issue8371/font-independent-empty-fontconfig-nextest.log`다.
- Native·최적화 WASM 전쪽 TSV와 312쪽 review·overlay를 `mydocs/pr/assets/issue_8371/`에 보존한다. Native의 이전 전쪽 캡처는 이후 변경한 Native 소스와 최종 head에서 다시 출력해 대조할 예정이다. 테스트와 전쪽 검증을 구분하며 전체 Rust 검증은 아직 완료하지 않았다.

## 전체 회귀와 OTF 세션 보존 확인

- `c6db7a6783`에서 공용 `target/pr-review`, Cargo 작업 8개, 테스트 스레드 12개로 전체 nextest를 실행했다. 12,040개 실행 중 12,037개 통과, 3개 실패, 45개 건너뜀이고 종료 코드는 100이다. 로그는 `output/issue8371/full-rust-nextest.log`다.
- 영월 수납대장 4쪽은 기존 한컴 Print와 비교할 때 표 원점이 기대값보다 위에 있다. 현재 release-test 실행 파일의 SVG 예비 대조에서는 raw 35.52%이며 기존 검증 기록은 해당 쪽의 Native·WASM 100%다. 기준 `upstream/devel`과 현재 head를 같은 환경에서 비교하고 원인을 고친다. 예비 review와 overlay는 `output/issue8371/regression-triage/yeongwol-svg-allowed/yeongwol/`에 있다.
- #3308 연속 하이픈 검사는 기대한 SVG 선분을 찾지 못했다. 현재 SVG에는 하이픈 글자 요소가 보인다. 같은 원본·Print와 기존 수정 이력을 확인했으며, 기준 브랜치와 글꼴 환경을 대조해 실제 출력과 검사의 타당성을 판정한다. SVG 예비 review와 overlay는 `output/issue8371/regression-triage/outline-svg/outline/`에 있다.
- 새 OTF 공급 함수는 문서 IR이 아닌 세션 글꼴과 파생 배치를 갱신한다. 무효화 검사에 세션 상태로 분류하고, 합성 OTF 공급·해제 전후 IR 및 HWP 저장 바이트가 유지되는 검사를 추가했다. 집중 검사 3개는 `output/issue8371/regression-triage/otf-session-tests-v2.log`에서 통과했다.
- 전체 테스트의 실패와 최종 head의 Native/fresh WASM 재검증이 남아 있으므로 PR 준비 완료가 아니다.

## 빈 문단과 실제 OTF 폭의 회귀 보정

- 고정 기준 `upstream/devel` `f125c735c7d2d44eaa44b78c70465bc89af45fb1`에서 수납대장과 연속 하이픈 기존 검사 두 개는 같은 환경에서 통과했다. `output/issue8371/regression-triage/base-failing-tests-focused.log`에 기록했다.
- 수납대장 앞에는 저장 줄 정보가 없는 빈 문단이 있다. 새 쪽 첫 줄 감춤이 이 문단의 실제 줄 상자를 숨겨 뒤 표를 약 22.5px 위로 이동시켰다. 저장 줄이 없는 문단은 새 숨김 표식으로 추정하지 않고 기존 배치에 맡긴다. 이전 SVG 진단 35.52%에서 보정 후 99.98%이며, 새 Native 전4쪽 최저는 99.60%다. 독립 좌표와 소속 관계를 보호하던 기존 검사는 유지하고 통과했다.
- HWP5 원본의 휴먼명조 OTF 굵기 표현에 확인되지 않은 0.05em을 더해 제목이 벌어진 사례를 발견했다. 실제 OTF 전진을 사용하면서 독립 Print로 확인된 원본 굵기 계약만 추가한다. HWP3 원본 및 변환 계보는 별도로 확인한 HFT 굵기 전진을 보존하며, 원본 HWP3 출처를 해소된 스타일에서 측정까지 전달한다. 해당 HWP5 원본 17쪽은 Native 83.37%에서 94.93%로 개선됐고 전17쪽 최저도 94.93%다.
- sample10 66쪽의 원본 명조 글머리표는 독립 Print에서 1em을 점유한다. 실제 선택 OTF가 해당 글리프를 제공하면 OTF 폭을 우선하고, 없으면 원본 폭을 사용한다. Native 66쪽은 97.21%에서 99.70%로 개선됐고, 전763쪽 최저는 98.29%, 90% 미만 0쪽이다. Native TSV는 `output/issue8371/regression-triage/sample10-native-v6/sample10/silhouette.tsv`다.
- 최종 후보의 집중 검사는 수납대장, sample10 312쪽 경계, OTF 공급 전후 저장 보존과 무효화 분류를 포함해 5개 통과했다. 로그는 `output/issue8371/regression-triage/focused-v6.log`다. 하이픈은 Native에서 첫 획이 한컴 Print와 같은 행에 있지만 기존 검사가 특정 SVG 요소만 요구해 실패한다. 해당 원본의 Native 전쪽은 90% 이상이고, fresh WASM 전쪽과 실제 잉크를 확인한 뒤 관계형 검사로 재검토한다.
- 위 결과는 Native 및 집중 검사다. 최적화 WASM 전쪽, 최종 전체 회귀와 PR 필수 검증은 아직 완료하지 않았다.
- 변경한 Rust 파일의 서식을 정리하고 `rustfmt --check`를 통과했다. 로그는 `output/issue8371/regression-triage/rustfmt-check-v7.log`다.

## 시스템 글꼴 공급과 새 WASM 검증

- 최적화 WASM SHA-256은 `1fae6535a3608cd68aa1cd808b528b80b270e0be5ad537a7a852d8ea214ffa59`이며, `pkg`와 Studio public의 WASM 및 JS 해시가 각각 같다. 빌드 로그는 `output/issue8371/regression-triage/wasm-build-v7-allowed.log`다.
- sample10의 같은 Print PDF와 실제 연결 그림을 사용한 fresh WASM 전763쪽은 최저 99.02%, 90% 미만 0쪽이며 쪽수도 763쪽으로 일치했다. 312쪽은 Native와 WASM 모두 raw 미대응 픽셀 0이다. TSV는 `output/issue8371/regression-triage/sample10-wasm-scores-v7/silhouette.tsv`다.
- Linux에서 별도 경로가 없을 때 WASM Visual Sweep의 기존 기본 목록은 비어 있었다. 이 상태에서 두 번째 하이픈 원본 17쪽은 87.80%였으나, 실제 설치된 휴먼명조 OTF를 공급하면 94.70%였다. 빈 공급 결과를 정상 글꼴 환경의 회귀나 통과 증거로 쓰지 않는다.
- 명시한 글꼴 경로가 없으면 Fontconfig에 실제 등록된 글꼴 파일을 자동 열거한다. 같은 family·스타일·굵기에서 실제 CFF OTF를 TTF보다 먼저 공급하며, 프로그램 바이트의 SFNT 서명으로 형식을 확인한다. 캐시된 글꼴 목록에도 같은 확인을 적용한다. 실제 전17쪽 자동 공급 검증은 최저 96.35%이며 휴먼명조 OTF 공급 해시를 `output/issue8371/regression-triage/nested-wasm-auto-v9/nested/wasm-export-complete.json`에 기록했다.
- Visual Sweep Python 검사 33개와 OTF 우선 공급 Node 검사 파일 2개가 통과했다. 로그는 `output/issue8371/regression-triage/sweep-system-font-tests-v3.log`, `output/issue8371/regression-triage/sweep-otf-order-tests.log`다. 나머지 원본의 자동 공급 WASM 재검증과 하이픈 관계형 회귀·전체 테스트는 계속 진행한다.
- 시스템에 등록된 Trebuchet MS는 `post` 이름 표의 글리프 수가 `maxp`와 달라 브라우저가 거절했다. 검증용 공급본의 이름 표만 보정하고 원본·공급본 해시와 보정 사유를 기록한다. 합성 손상 입력에서 `glyf`·`hmtx`·`cmap` 보존을 검증했으며, 실제 수납대장의 자동 공급 WASM 전4쪽 최저는 99.59%다. 로그는 `output/issue8371/regression-triage/sweep-post-repair-tests.log`다.
- 연속 하이픈 검사는 두 원본 모두 Native/fresh WASM 전쪽 90% 이상과 3쪽·8쪽의 Print 비교를 확인했다. 두 쪽의 Native는 각각 95.12%·96.63%이므로 절대 좌표 대신 원문 하이픈 수, 표 셀 소유와 숨긴 글자의 대체 선분 관계를 검사하도록 변경했다. 새 검사와 수납대장·sample10 경계 검사를 함께 실행해 3개 모두 통과했다. 로그는 `output/issue8371/regression-triage/focused-relational-v10.log`다. 전체 회귀 재검증은 남았다.
- 시스템 글꼴 자동 공급으로 다시 출력한 sample10 WASM 전763쪽은 최저 99.02%, 90% 미만 0쪽이며 312쪽은 실제 미대응 픽셀 0이다. 자동 목록을 사용한 66·312쪽 PNG는 앞서 직접 판독한 PNG와 각각 SHA-256이 일치한다. TSV는 `output/issue8371/regression-triage/sample10-wasm-auto-scores-v10/silhouette.tsv`다.

## 전체 검사 컴파일 보완

- 전체 nextest 재실행 중 라이브러리 단위 테스트의 기존 `ResolvedStyleSet` 초기화 여섯 곳에서 새 `source_hwp3_native` 필드가 누락되어 컴파일이 실패했다. 이 입력들은 HWP3 원본이 아니므로 `false`를 명시했다. 실패 로그는 `output/issue8371/full-rust-nextest-final-v11.log`이며, 라이브러리 테스트 전체 컴파일과 전체 nextest를 다시 확인한다.
- 최신 WASM 산출물의 sample10 전763쪽 TSV는 `output/issue8371/final-head/sample10-wasm-scores/silhouette.tsv`이며 최저 99.02%, 90% 미만 0쪽이다.
- 제출용 sample10 TSV와 312쪽 Native·WASM review 및 overlay를 현재 구현의 결과로 갱신했다. 최신 WASM 66·312쪽 PNG는 직접 판독한 이전 출력과 각각 SHA-256이 동일하다.
- 회귀 실패와 직접 관련된 원본 세 건의 전쪽 TSV도 `mydocs/pr/assets/issue_8371/regression/`에 보존한다. 영월 수납대장 4쪽의 최저는 Native 99.60%·WASM 99.59%, `issue3307_outline_number.hwpx` 9쪽은 93.25%·97.18%, `issue2007_nested_cell_pagination_42065.hwp` 17쪽은 94.93%·96.35%다. 각각 같은 원본과 독립 Print PDF를 비교했다. 최종 WASM 검증 로그는 `output/issue8371/final-head/{yeongwol-wasm-retry2,outline-wasm,nested-wasm}.log`다.
- 테스트용 초기화 보완 후 `CARGO_BUILD_JOBS=8 cargo test --lib --profile release-test --target-dir target/pr-review --no-run`은 종료 코드 0이었다. 이어서 `CARGO_BUILD_JOBS=8 cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`를 실행해 12,041개 전체가 통과했고 45개를 건너뛰었다. 종료 코드 0이며 로그는 `output/issue8371/full-rust-nextest-final-v12.log`다. 수납대장과 연속 하이픈의 보완 검사도 이 실행에서 통과했다.
- 글꼴 메트릭 추가로 인라인 텍스트 렌더 노드 크기가 Clippy의 variant 크기 차이 기준을 넘어, 빈번한 텍스트 노드마다 별도 힙 할당을 만들지 않도록 해당 enum에 한정한 lint 예외와 이유를 추가했다. 실행 동작 변경은 없다. `cargo fmt --all -- --check`와 Native·WASM·workspace 전체 Clippy가 통과했으며 로그는 `output/issue8371/fmt-final-v2.log`, `output/issue8371/clippy-{native,wasm,workspace}-final-v2.log`다.
- 단위 테스트 분류 정책 검사는 4,099개 테스트를 확인해 통과했다. 현재 생성된 suite와 정책으로 재산출한 suite는 모두 동일한 1,865개 원본 테스트 파일을 포함하지만 suite별 배정이 달라 `--check`가 drift를 보고했다. 실행 중인 전체 검사를 종료한 뒤 `--prepare`와 정책 검사를 수행한다. 로그는 `output/issue8371/unit-test-tiers-final.log`, `output/issue8371/suite-manifest-final-allowed.log`다.
- lint 예외를 포함한 소스에서 전체 nextest를 다시 실행해 12,041개 모두 통과하고 45개를 건너뛰었다. 로그는 `output/issue8371/full-rust-nextest-final-v13.log`다. 그 뒤 파생 suite를 `--prepare`로 재생성하고 `--check`를 통과했으며, workspace 전체 빌드도 통과했다. 로그는 `output/issue8371/suite-prepare-final.log`, `output/issue8371/suite-manifest-final-prepared.log`, `output/issue8371/workspace-build-final.log`다.
- 이후 `upstream/devel`이 `ad4bf269ffef5adf2142b33004a9981fad7ce804`로 바뀐 것을 확인했다. 재생성된 suite의 집중 검사를 마친 다음 리베이스하고 새 base의 조판·편집 기능과 이 작업의 출력·회귀를 함께 다시 검증한다. 위 통과 결과는 이전 base의 증거다.

## 새 기준 기능 보존 검증과 남은 후퇴

- `ad4bf269ffef5adf2142b33004a9981fad7ce804` 위로 리베이스한 소스 `6aa38f7d6d`에서 전체 nextest 12,049개가 통과했고 45개를 건너뛰었다. #8040·#7905와 기존 실패를 포함한 집중 검사 11개도 통과했다. 로그는 `output/issue8371/rebase-full-nextest.log`, `output/issue8371/rebase-focused.log`다.
- sample10의 새 Native와 최적화 WASM 전763쪽은 각각 최저 98.29%, 99.02%이며 두 TSV 모두 리베이스 전 검증본과 바이트가 같다. 66·312쪽 PNG도 각각 이전 직접 판독본과 바이트가 같다. TSV는 `output/issue8371/rebase-native-sample10/sample10/silhouette.tsv`, `output/issue8371/rebase-wasm-sample10-scores/silhouette.tsv`다.
- #8040의 원본·삽입·삽입 후 삭제·HWPX 저장본을 Print와 전쪽 비교한 결과 Native 최저 98.64·98.61·98.61·97.33%, WASM 최저 98.77·98.73·98.73·97.47%다. 저장본의 재열기 출력이며 실제 편집 직후 출력과 혼동하지 않는다. 같은 환경의 새 base와 비교하면 Native 일부 쪽이 0.1%p 이상 하락해 원인 보정이 남았다. 결과는 `output/issue8371/rebase-{base-native,native-base-features,wasm-8040}/`이다.
- #7905 원본의 동일 환경 Native 비교는 base 대비 1쪽 91.44→88.35%, 2쪽 93.51→86.32%, 3쪽 38.22→38.04%, 4쪽 57.57→60.26%, 5쪽 36.28→50.51%, 6쪽 42.35→40.61%다. 기준부터 3~6쪽은 미달이지만 현재 변경으로 1·2·3·6쪽이 후퇴했으므로 보정이 필요하다. 기준과 현재 비교 이미지는 `output/issue8371/rebase-law-{base,head}-panels/`에 있다.
- 실제 시스템 OTF의 같은 family에 영문 전용 `ENGGT.otf`와 한글 전용 `HGGGT.otf` 등이 존재한다. 영문 family 별칭을 기존 굵기 판정에 추가하는 실험은 1쪽 88.35→88.37%로 효과가 없어 되돌렸다. 한글 전용 OTF를 별도 공급한 진단 역시 1쪽 86.94%, 2쪽 88.72%로 해결되지 않았다. 글꼴 이름이나 파일 순서만 원인으로 확정하지 않으며 원본 선언→실제 선택 글리프·메트릭→paint를 계속 대조한다. 실험 로그는 `output/issue8371/rebase-weight-candidate.log`, `output/issue8371/korean-otf-probe.log`다. 후퇴가 남아 PR은 준비 완료가 아니다.
- Native renderer snapshot 추적에서 원본 한양신명조 한글 768자와 한양견고딕 한글 51자가 같은 family의 한글 전용 OTF를 찾지 못하고 `Malgun Gothic`으로 그려진 것을 확인했다. 같은 스타일의 분할 CFF 후보를 모두 글리프별 paint 순서에 포함하고, Native 측정도 같은 순서로 글리프가 있는 프로그램의 전진을 사용하게 했다. 영문 `Myeongjo`/`Myungjo` family의 결측 글리프에는 명조 대체군을 사용한다. 합성 CFF 두 프로그램의 서로 다른 문자 선택 및 명조 계열 분류 단위 검사는 통과했다. 로그는 `output/issue8371/diag-native-font-trace.json`, `output/issue8371/rebase-split-otf-unit.log`, `output/issue8371/rebase-romanized-unit.log`다.
- 위 보정의 sample10 Native 전763쪽 최저는 98.25%, 312쪽은 100%다. 이전 후보 대비 193쪽은 0.16%p 하락했고 미대응 픽셀 0인 쪽은 311→687쪽이다. #7905 1·2쪽은 90.68%, 93.37%로 개선됐지만 새 base 대비 각각 0.76%p·0.14%p 낮다. #8040도 일부 쪽이 0.1%p 넘게 낮다. 후퇴 보정, fresh WASM의 동일 규칙 적용·전쪽 검증, 최종 전체 회귀 및 lint는 남았다. TSV는 `output/issue8371/rebase-myeongjo-sample10/sample10/silhouette.tsv`, `output/issue8371/rebase-myeongjo-native/`에 있다.
- 보정 후 #8040·#7905와 보고된 회귀를 포함한 집중 nextest 16개는 통과했다. Native Skia·WASM·workspace 전체 Clippy도 각 로그 `output/issue8371/multiface-{skia-clippy-v2,wasm-clippy-v2,workspace-clippy-v3}.log`에서 통과했다. Clippy가 지적한 기존 `and_then`과 테스트 모듈 위치를 동작 변경 없이 정리했다. 전체 회귀와 후퇴 해소는 아직 남았다.
- #8040 원본 2쪽의 `㉠∼㉤`를 같은 좌표계에서 추적했다. 독립 Print PDF에서 각 글자 시작은 113.76·124.20·134.64pt로 간격이 10.44pt다. 새 base 렌더 트리에서 `∼`는 13.9px 전각이나 분할 OTF 후보에서는 원본 기호가 없는 상태에서 7.0px 반각 휴리스틱으로 바뀌어 뒤 문장이 이동했다. 실제 글리프 폭이 없는 HFT 슬롯의 `∼` 대체 전진을 전각으로 보정했으며, 확인된 글리프 메트릭은 계속 우선한다. 재출력과 전체 회귀 결과를 확인하기 전에는 개선 완료로 판단하지 않는다. 원본 trace는 `output/issue8371/diag-exam-native-trace.json`, 2쪽 비교는 `output/issue8371/multiface-8040-p2/exam/`이다.
- 위 보정의 Native 실행 파일로 재출력한 시험지 원본 2쪽은 99.03→99.71%이며 `∼`는 다시 13.9px로 측정된다. 그러나 새 base의 99.88%와 0.17%p 차이가 남고, 원본 3·4쪽과 삽입 후 삭제·HWPX 저장본에도 0.1%p 초과 하락이 남는다. #7905 6쪽의 수치는 바뀌지 않았고 3~6쪽은 새 base에서도 90% 미만이다. 기준과 후보를 혼동하지 않으며 나머지 차이를 계속 분석한다. 로그와 결과는 `output/issue8371/hft-symbol-native-sweep.log`, `output/issue8371/hft-symbol-native/`, 2쪽 review·overlay는 `output/issue8371/hft-symbol-8040-p2/exam/`이다. 해당 sweep의 종료 코드는 #7905 미달로 1이며 PR gate 통과가 아니다.
- 2쪽 머리글의 괄호 다음 `사`가 새 base 121.4px에서 후보 120.6px로 이동했다. 독립 Print의 `사` 시작은 91.44pt(96dpi에서 121.92px)다. 같은 머리글의 1쪽 trace에서 원본 `신명 신그래픽`의 굵게 속성은 일반 OTF의 합성 굵기를 선택하고 HFT 경로의 굵기 전진은 OTF 경로에서 누락되는 것을 확인했다. 실제 굵은 OTF에는 더하지 않고 원본 HFT 메트릭 계약과 합성 굵기가 함께 있는 경우 기존 전진을 보존하도록 수정했다. `output/issue8371/diag-exam-page2-trace.json`은 파일명과 달리 페이지 인자를 무시한 이전 진단 실행 파일의 1쪽 trace이며 앞의 `∼` 보정 전 소스다. 페이지별 위치는 각 렌더 트리와 Print에서 별도로 확인한다.
- 두 보정을 반영한 Native 전쪽 재비교에서 시험지 원본 2쪽은 99.77%로 더 개선됐으나 새 base 99.88% 대비 0.11%p 하락이다. 원본 3·4쪽과 삽입 후 삭제·HWPX 저장본도 허용폭 0.1%p를 넘는다. #7905 각 쪽은 앞 비교와 동일해 새 base 대비 후퇴가 남는다. `output/issue8371/hft-bold-native-sweep.log`의 종료 코드 1은 #7905의 기존 90% 미달을 포함한다. sample10 전쪽 재출력, 현재 변경에 대한 fresh WASM 및 회귀 검증은 아직 진행 중이다.
- 시험지 4쪽 첫 보기의 한컴 Print `A` 시작은 122.24px이며 새 base는 122.2px, 후보는 124.5px이다. 공통 문자 측정에서 한글 run의 ASCII 구두점은 영문 슬롯을 사용하지만 OTF 폭 조회만 한글 슬롯에 고정되어 있는 결함을 발견했다. 영문 슬롯의 OTF도 같은 글꼴 선택과 exact-source 조건으로 준비하고 측정·합성 굵기가 해당 슬롯을 소비하도록 수정했다. 이것이 이 차이를 해소하는지는 재빌드와 독립 Print 비교로 확인해야 한다. 기존 `hft-bold-sample10` 전쪽 실행은 이 추가 수정 전 실행 파일의 결과다.
- 영문 슬롯 보정의 Native 재출력에서 위 `A` 원점이 124.5→122.2px로 돌아와 한컴 Print의 122.24px에 근접했고, 원본 4쪽 raw는 99.40→99.50%로 개선됐다. 새 base의 99.63% 대비 0.13%p 하락이므로 나머지 차이를 계속 확인한다. 원본 Native 전쪽 TSV는 `output/issue8371/hft-latin-native/original/silhouette.tsv`다. 작업 중 서식 검사도 `output/issue8371/hft-latin-slot-fmt-v2.log`에서 통과했다.
- 영문 슬롯 보정까지 반영한 sample10 Native 전763쪽은 기존 한컴 Print PDF와 쪽수가 일치하고 최저 98.25%, 90% 미만 0쪽이다. 직전 중간 빌드와 TSV가 바이트 단위로 동일하다. `output/issue8371/hft-latin-sample10/sample10/silhouette.tsv`, `output/issue8371/hft-latin-sample10.log`에서 확인했다. fresh WASM은 아직 재검증하지 않았다.
- 시험지 3쪽 `※ 질문에 따라`의 `※`는 HFT 원본에 기호가 없어 반각 추정으로 내려간다. Print에서 `※`와 다음 공백 시작 간격은 9pt(96dpi 12px)이며, 후보의 뒤따르는 `A` 시작은 645.2px로 Print 649.4px보다 앞선다. `output/issue8371/diag-exam-p3-v3.json`의 페이지 번호 2를 확인했다. 이 원인에 대한 수정은 진행 중인 검사가 끝난 다음 수행한다.
- #7905 1쪽은 새 base와 후보의 TextRun 원점·폭 중 1px 이상 달라진 항목이 없으나, 본문을 확대하면 후보에서 한컴 Print보다 획이 옅다. 분할 OTF 보정 후 trace `output/issue8371/diag-law-p1-v3.json`은 원본 한양신명조 1,251자의 paint와 폭이 `HY Sinmyeongjo` OTF를 선택한 것을 보여 준다. `output/issue8371/law-font-diagnostic-p1.png`는 새 base·후보·Print의 같은 영역을 보여 주며, OTF 획과 Print 차이의 원인은 아직 확정하지 않는다.
- 영문 슬롯 보정까지 적용한 소스에서 사용자 보고 실패 두 건을 포함한 집중 nextest 10개가 모두 통과했다. 로그는 `output/issue8371/hft-latin-focused-nextest.log`다. 이후 3쪽의 원본 HFT 기호 결측 `※`에 전각 대체 전진을 적용했으므로 그 변경의 출력과 회귀는 새로 확인해야 한다.
- `※` 보정 후 시험지 3쪽의 `A` 시작은 645.2→649.4px로 이동해 독립 Print의 649.4px과 일치하고 raw는 99.53→99.61%다. 새 base는 99.72%여서 0.11%p 차이는 남는다. 최신 쪽별 비교는 `output/issue8371/hft-reference-native/`, 3쪽 review·overlay는 `output/issue8371/hft-reference-8040-p3/exam/`이다. PDF Type3 `영` 윤곽과 선택한 `HGSMJ.otf`의 같은 글리프는 두 cubic 곡선의 좌표가 역방향까지 동일하다. 따라서 해당 글리프의 굵기 차이를 다른 윤곽의 선택이라고 단정하지 않으며 래스터화·변환 차이를 확인한다. 진단 로그는 `output/issue8371/law-glyph-outline-diagnostic.log`다.
- #7905 `영`의 PDF Type3와 OTF 윤곽을 수치 적분한 채움 면적은 각각 약 145,299와 145,391 글꼴 단위²다. 288dpi 확대 영역은 `output/issue8371/law-glyph-288-diagnostic.png`, 계산 코드는 `output/issue8371/law-glyph-area.py`다. 이 확대는 진단일 뿐 기본 96dpi의 gate를 대체하지 않는다. 같은 글리프 형태가 실제 출력에서 다르게 보이는 이유와 나머지 본문 전체에 같은 판단을 적용할 수 있는지는 추가 확인한다.

## 분할 OTF의 WASM·Studio 공급 경계

- Native는 같은 family/style의 한글·영문 CFF 프로그램을 글리프별로 선택하지만 기존 WASM 폭 공급과 Studio 호스트 공급은 단일 face만 반환하거나 같은 family/style의 중복을 모호하다고 거절했다. Visual Sweep의 Canvas 등록도 같은 family/style을 첫 face 하나로 제한했다. 따라서 Native의 분할 공급을 WASM 검증 결과로 추정할 수 없다.
- WASM 공급자는 기존 단일 `Uint8Array`를 그대로 받고 분할 프로그램 배열도 같은 순서로 받도록 확장했다. Studio는 동일 family/style의 서로 다른 PostScript 식별자를 가진 후보를 모두 준비한 뒤 CFF를 TTF보다 먼저 표시하고, 확인된 CFF만 그 순서대로 폭 공급에 사용한다. 중복 식별자와 준비 중인 일부 묶음은 자동 확정하지 않는다. Visual Sweep은 같은 family의 후보를 역순 등록해 CSS의 마지막 등록 우선순위가 측정 후보 순서와 일치하도록 했다.
- 위 비동기 공급·이름 모호성·기존 단일 face 계약의 Node 검사 35개와 Studio TypeScript 검사는 통과했다. 로그는 `output/issue8371/split-otf-node-tests-v2.log`, `output/issue8371/split-otf-typescript.log`다. 이 검사는 실제 Canvas 글리프 순서나 한컴 Print 일치 증거가 아니며, 최적화 WASM 빌드와 실제 입력의 Native/fresh WASM 비교는 진행 중이다.
- Chromium에 저장소 합성 CFF 세 개를 실제로 등록해 같은 글리프가 겹칠 때 첫 후보가 선택되고 첫 후보에 없는 문자는 다음 face로 넘어가는 것을 확인했다. `output/issue8371/split-otf-browser-contract.log`다. family당 한 프로그램을 소유하는 기존 윤곽 저장소는 첫 후보만 등록하고, 단일 cmap만 보유하는 coverage 경로는 분할 family에 부분 cmap을 확정하지 않도록 했다. 합성 글꼴 검사만으로 실물 조판을 통과한 것은 아니다.
- 재감지 시 같은 문서 객체에 이전 단일 face의 coverage가 남을 수 있으므로, 분할 family를 등록하기 전에 해당 family의 기존 coverage·윤곽을 지우는 WASM API를 추가했다. 관련 없는 family는 보존하고 새 단일 face를 다시 등록할 수 있게 한다. 기존 빌드가 최적화 중일 때 이 경계를 추가했으므로 `split-otf-wasm-build.log`의 빌드는 중간 결과이며 최종 소스로 다시 빌드해야 한다. 분할 PUA도 첫 face가 기호를 공급하지 못할 때 다음 후보를 검사한다.
- 같은 family에 CFF와 TTF가 같이 설치된 경우도 처리한다. 표시 후보는 CFF를 먼저, 같은 family의 나머지 프로그램을 그다음에 두고 각 형식 안의 순서는 보존한다. Native 측정에서 앞선 실제 TTF가 글리프를 소유하면 뒤 CFF의 폭으로 덮지 않고 원본 HFT 폭 계약으로 돌아가도록 했다. 서로 다른 합성 프로그램의 선택·폭 경계 검사를 추가했으며 아직 Rust 실행과 전쪽 시각 재검증은 남았다.
- 이후 합성 글꼴을 사용한 Rust 단위 검사 3개는 통과했다. 첫 TTF의 글리프가 뒤 CFF의 전진을 막는 관계, 분할 CFF별 glyph 선택, 한 family의 coverage 제거·재등록과 다른 family 보존을 확인했다. 로그는 `output/issue8371/split-otf-rust-unit.log`다. 관련 Node 검사 38개와 TypeScript 검사도 통과했다(`output/issue8371/split-otf-node-tests-v11.log`, `output/issue8371/split-otf-typescript-v7.log`). 이 결과는 Native 전체 문서나 fresh WASM 시각 검증의 대체가 아니다.
- 추가 검토에서 Native 시스템 후보의 형식 판정은 첫 4바이트만 읽어 TTC 내부의 CFF/TTF face를 구분하지 못한다. face 인덱스를 이용한 실제 SFNT 판정과 합성 collection 검사가 필요하다. 현재 실행 중인 빌드와 Native 전쪽 검증이 끝난 뒤 해당 경계를 보완한다.
- 시스템의 `NotoSansCJK-Regular.ttc` 0번 face는 CFF이며 `gulim.ttc` 0번 face는 TrueType이다. 별도 설치에 의존하지 않도록 저장소의 합성 OTF·TTF로 collection 생성 방법을 `output/issue8371/synthetic-ttc-manual.log`에서 확인했다.
- Native의 CFF 선택과 측정 공급이 TTC 컨테이너의 표식 대신 실제 face 인덱스의 윤곽 형식을 판별하도록 보완했다. 같은 collection의 여러 face는 바이트를 공유하고 크기 상한에서도 한 번만 계상한다. 합성 collection의 CFF·TTF 분류 및 공유 검사를 추가했으며 Rust 실행 전이다. 위 작업은 최적화 중인 `split-mixed-final-wasm-build.log`의 빌드에는 포함되지 않아 빌드 종료 후 다시 검증해야 한다.
- 혼합 OTF·TTF 후보를 반영한 Native 빌드의 sample10 전763쪽은 최저 98.25%, 90% 미만 0쪽이며 이전 TSV와 동일했다. 로그·TSV는 `output/issue8371/split-mixed-sample10.log`, `output/issue8371/split-mixed-sample10/sample10/silhouette.tsv`다. 이 출력은 뒤에 추가한 TTF 전용 중복 방어 전 바이너리이며 TTC 보완도 포함하지 않는다. #8040과 #7905의 같은 base 대비 남은 차이는 `output/issue8371/split-mixed-base-comparison.tsv`에 보존했다.
- #7905의 옅은 본문에 대해 Native Print 윤곽 경로를 추적했다. 같은 family의 첫 영문 CFF만 `HftOutlineReplay`로 준비하고 실제 선택된 한글 CFF와 Typeface ID가 달라 일반 glyph 래스터로 내려가고 있었다. 인쇄·고품질 경로에서 실제 선택된 같은 family의 face를 글리프별로 지연 준비하도록 보정하고, 저장소 합성 분할 CFF의 두 번째 glyph가 소수 원점 순서를 보존하는 관계형 검사를 추가했다. 수정 전 FAIL/수정 후 PASS 및 한컴 Print 대조는 아직 실행 전이다. WASM의 단일 윤곽 저장소도 같은 분할 경계를 실제 FontFace 공급과 함께 검증해야 한다.
- 수정 전 관계형 검사의 최초 실행은 Skia 빌드 의존성 다운로드 중 DNS 실패로 종료돼 검사 결과가 아니다(`output/issue8371/split-hft-outline-before.log`). 네트워크 접근이 가능한 환경에서 해당 suite만 다시 실행 중이다(`output/issue8371/split-hft-outline-before-network.log`). 수정 후 파일은 `output/issue8371/split-hft-text-replay-after.rs`에 임시 보존하고 검사 종료 직후 복원한다.
- 재시도에서 의존성 다운로드가 성공했고, 수정 전 분할 CFF 검사에서 원점 4.1과 4.35의 잉크 중심이 모두 24.3551로 같아 실패했다. 검사 종료 뒤 수정 후 파일을 복원했으며 TTC·coverage 검사를 포함해 보정 후 검사를 실행 중이다. 로그는 `output/issue8371/split-hft-outline-before-network.log`, `output/issue8371/split-hft-outline-after.log`다. 중간 fresh WASM의 실제 Chromium 공급 계약은 `output/issue8371/split-otf-wasm-contract.log`에서 통과했지만 그 뒤의 TTC·Native 윤곽 보정을 포함한 최종 빌드는 아니다.
- 보정 후 같은 분할 CFF 윤곽 검사와 합성 TTC face별 형식·바이트 공유, 앞선 TTF 폭 경계, family coverage 초기화 검사 4개가 모두 통과했다. 수정 전 FAIL과 수정 후 PASS는 각각 `output/issue8371/split-hft-outline-before-network.log`, `output/issue8371/split-hft-outline-after.log`에서 확인한다. 실물 Native Print 재출력은 이어서 수행한다.
- #7905 종합부동산세법 시행규칙 1쪽의 QR은 원본 그림의 `orgSz`에 합성 `renderingInfo`를 적용하면 `sz` 틀이 되지만, 공통 그림 크기 함수가 중간 `curSz`와 최종 틀의 축별 최댓값을 선택해 과대 출력했다. 원본의 합성 변환과 틀이 반올림 범위에서 일치하는 경우 최종 틀을 사용하도록 보정했다. 기존 한컴 2018 Print와 같은 입력의 Native 1쪽 raw는 90.44075%에서 98.85059%로 올랐다(`output/issue8371/qr-composed-frame-law-p1.log`). QR의 크기는 개선됐으나 본문 중간의 줄 차이와 전체 페이지·fresh WASM·그림 대조군 검증은 남았다. 이 1쪽 결과를 전체 문서의 통과로 표시하지 않는다.
- QR 보정 전 Native 바이너리의 sample10 전 763쪽은 최저 98.29%, 90% 미만 0쪽, 정확히 100%인 쪽 687개다(`output/issue8371/split-hft-outline-sample10/sample10/silhouette.tsv`). QR 보정 후 법령 문서는 1~6쪽 각각 98.85%, 93.74%, 37.53%, 57.18%, 36.03%, 41.76%다(`output/issue8371/qr-composed-frame-law-all/law/silhouette.tsv`). 3~6쪽은 #7905 통합 전부터 미달인 문단 이월 결함이지만, 최신 base 대비 변동과 새 수정의 영향을 별도로 확인해야 한다.
- 그림 크기 공통 함수의 기존 그림 흐름 검사 4개는 보정 후 통과했다(`output/issue8371/qr-frame-existing-regressions.log`). #7905에서 사용한 정상 그림 흐름 대조군 13문서·26쪽의 Native 최저 raw는 99.99%다(`output/issue8371/qr-frame-controls.log`). 최종 소스의 fresh WASM 빌드와 WASM 대조는 진행 중이다.
- 설치 글꼴이 없는 환경을 확인하기 위해 빈 디렉터리만 가리키는 임시 Fontconfig 설정에서 `fc-list`가 0개임을 확인했다. 같은 환경에서 이미 빌드한 분할 CFF 윤곽 회귀 바이너리를 직접 실행해 통과했다(`output/issue8371/fontless-split-cff-test.log`). 이 검사는 시스템 OTF 대신 저장소의 합성 글꼴 바이트를 직접 사용하며 실물 PDF 시각 비교와 구분한다.
- 같은 글꼴 격리 환경에서 HFT 윤곽 회귀 모듈의 네 검사도 모두 통과했다(`output/issue8371/fontless-hft-contract-tests.log`).
- QR의 축소 합성과 비일치 폴백을 검증할 글꼴 비의존 단위 검사를 준비했다. 합성 행렬과 원본의 HWPUNIT 크기 계약만 검사하며 시스템 글꼴을 읽지 않는다. fresh WASM 및 검사 실행이 끝나기 전에는 통과로 판정하지 않는다.
- 빈 Fontconfig 환경에서 그림 크기 계약 검사가 통과했다(`output/issue8371/fontless-qr-frame-contract.log`). 기준 PDF와 전체 fresh WASM 비교는 별도로 확인한다.
- 현재 소스의 fresh WASM 빌드는 완료됐다(`output/issue8371/qr-frame-fresh-wasm-build-retry.log`). 법령 문서의 fresh WASM 1쪽은 98.86%, 2쪽은 93.75%이나 3~6쪽은 37.51%, 57.15%, 36.00%, 41.87%로 여전히 미달이다(`output/issue8371/qr-frame-law-wasm-all-retry/law/silhouette.tsv`). 1쪽 QR은 PDF 틀에 맞고 연락처를 덮지 않지만 본문 중간에 남은 줄 차이를 확인했다(`output/issue8371/qr-frame-law-wasm-p1-retry/law/overlay/overlay_001.png`). 전체 문서 통과나 PR 준비 완료가 아니다.
- 그림 크기 계약은 시스템 글꼴이 없는 동일 환경에서 수정 전 실제 `(10202, 12280)`을 선택해 실패했고 수정 후 통과했다. 두 로그는 각각 `output/issue8371/fontless-qr-frame-before.log`, `output/issue8371/fontless-qr-frame-after.log`다. 수정 전 실행 뒤 구현 소스의 바이트 복원을 확인했다.
- 법령 문서의 같은 환경 Native base와 후보 raw 차이는 `output/issue8371/qr-frame-law-base-delta.tsv`에 기록했다. 1~2쪽은 개선되지만 3~6쪽은 0.25~0.69%p 하락하므로 작업자가 지정한 0.1%p 기준을 만족하지 않는다. 3쪽 렌더 트리의 텍스트 런은 양쪽 모두 270개이고, 일부 글꼴 전진 및 위치가 미세하게 다르다. 원래 쪽 이월 결함과 현재 변경에 따른 차이를 분리해 계속 검토한다.
- QR 보정 후 sample10 Native 전 763쪽은 최저 98.29%, 90% 미만 0쪽, 정확히 100% 687쪽이며 보정 전 TSV와 바이트가 동일하다(`output/issue8371/qr-frame-final-sample10-native/sample10/silhouette.tsv`). 현재 source로 빌드한 fresh WASM 전쪽 비교는 진행 중이다(`output/issue8371/qr-frame-final-sample10-wasm-retry.log`).
- PR 준비 중 원격 `upstream/devel`이 `ad4bf269ffef5adf2142b33004a9981fad7ce804`에서 `698906bf4aeaa6d0d7101a4fd4d7c4a9229f6bb2`로 갱신됐다. 신규 #8165 머리글·꼬리글 및 이어지는 구역 기능과 기존 작업 양쪽을 리베이스 후 재검증해야 하며, 이전 base의 성공 결과를 새 base의 최종 증거로 사용하지 않는다.
- sample10 fresh WASM 전쪽 재실행은 Visual Sweep 옵션에 원본 연결 그림 세 개가 빠져 1쪽에서 decode 실패로 종료됐다(`output/issue8371/qr-frame-final-sample10-wasm-retry.log`). 기존 검증에서 해시 고정한 실제 그림 세 개를 공급한 1쪽 재현을 실행하고, 새 base 리베이스 후 같은 조건으로 전쪽을 재실행한다.
- 새 base `698906bf4aeaa6d0d7101a4fd4d7c4a9229f6bb2` 위로 리베이스했다. sample10은 원본 HWP·기존 PDF와 연결 그림을 그대로 사용하며 HWPX로 변환하지 않는다. 연결 그림 세 개를 공급한 기존 base의 WASM 1쪽 출력은 정상 완료됐다(`output/issue8371/qr-sample10-linked-p1.log`).
- #8165에서 공개한 HWPX 두 개는 sample10과 별개 재현 문서다. 사용자의 승인에 따라 한컴 2024 MCP에서 실제 Print 경로 `frame_print_to_pdf_ex_one_up`, `pdf_print_method=0`, 버전 `13.0.0.3901`으로 각 1쪽·2쪽을 확보했다(`pdf/issue_8165_rebase/`). 각각 입력·PDF의 짝을 유지해 후보 Native 전쪽을 실행한 결과 머리말·꼬리말은 91.12%, 바탕쪽 두 쪽은 100%다(`output/issue8371/rebase-8165-candidate-native.log`). 새 base와 후보의 같은 환경 비교와 fresh WASM은 아직 진행 중이다.
- #8165의 새 base `698906bf4a`와 리베이스 후보 `df61391817`은 동일한 MCP Print PDF와 이 서버의 동일 글꼴 환경으로 비교했으며, 두 문서의 전체 Native PNG와 TSV가 바이트 단위로 동일했다(`output/issue8371/rebase-8165-base-native.log`, `output/issue8371/rebase-8165-candidate-native.log`). PDF SHA-256은 머리말 `f220f2c69ecf4069bb3bb4752f0484c1bd95632b524008b41fdaa67c92175168`, 바탕쪽 `69a9f2ac192593f3c19a32a2ab2a08b09de8d44d4396439f58f4a84e82310dce`다. 새 base의 fresh WASM과 최종 전쪽 검사는 별도로 수행해야 한다.
- 리베이스 후 자동 수집 파일을 `--prepare`로 갱신해 #8165의 두 테스트 모듈을 포함했고 `--check`가 통과했다(`output/issue8371/rebase-8165-suite-prepare.log`). `CARGO_BUILD_JOBS=8`, `target/pr-review`, `--test-threads 12`로 전체 Rust nextest를 실행 중이다(`output/issue8371/rebase-8165-full-nextest.log`).
- 새 base와 후보를 법령 입력·기존 한컴 Print PDF로 재실행했다. Native 1쪽은 91.44%→98.85%, 2쪽은 93.51%→93.74%이나, 3~6쪽은 각각 38.22%→37.53%, 57.57%→57.18%, 36.28%→36.03%, 42.35%→41.76%다(`output/issue8371/rebase-8165-law-base/law/silhouette.tsv`, `output/issue8371/rebase-8165-law-candidate/law/silhouette.tsv`). 0.1%p 초과 하락은 새 base에서도 해소되지 않았으며 PR 준비 완료로 판정하지 않는다.
- 리베이스 후보 `df61391817`에서 sample10 Native 전 763쪽과 기존 Print PDF를 다시 비교했다. 최저 98.29%, 90% 미만 0쪽, 정확히 100% 687쪽이며 리베이스 전 같은 코드의 TSV와 바이트가 같다(`output/issue8371/rebase-8165-sample10-native/sample10/silhouette.tsv`). 새 base 자체와의 전쪽 비교 및 리베이스 후 fresh WASM은 별도로 실행한다.
- 동일 원본·PDF·글꼴 환경에서 새 base의 sample10 Native 전쪽을 실행했다. base는 108쪽이 90% 미만이고 후보는 0쪽이며, 후보의 base 대비 하락이 0.1%p를 초과하는 쪽은 0쪽이다. 쪽별 비교는 `output/issue8371/rebase-8165-sample10-delta.tsv`, base 원본 TSV는 `output/issue8371/rebase-8165-sample10-base/sample10/silhouette.tsv`다. 이 결과는 fresh WASM 검증을 대체하지 않는다.
- 전체 nextest에서 #2724 뮤테이터 분류 검사가 `clear_text_font_family_native` 미등재로 실패했다. 해당 메서드는 `hft_outline_fonts`의 paint 공급·coverage만 지우며 저장 IR을 바꾸지 않는다. `SessionState`에 근거를 추가하고 `hft_outline_font_contract`의 공개 API 실행에 해제·재등록 전후 문서 IR과 저장 바이트 보존을 포함했다. 진행 중인 전체 nextest 종료 후 새 테스트 바이너리로 재검증한다.
- #2724 가드의 원 PR #2797은 문서 편집이 원본 스트림 재사용에 가려지는 문제를 예방한다. 현재 serializer는 추가로 provenance 일치를 확인하지만, 변경 메서드를 편집·세션 상태로 명시적으로 분류하는 검사는 유지한다. `clear_text_font_family_native`는 그림용 font registry와 coverage만 지우므로 `SessionState`에 해당하며, 실제 HWP 원본 스트림이 존재하는 `saved/blank2010.hwp`를 열어 등록·해제 후 IR과 직렬화 바이트가 같음을 검사했다. 시스템 글꼴이 없는 환경에서 #2724 가드 5개와 이 보존 검사 1개가 통과했다(`output/issue8371/passthrough-validity-fontless.log`). 보정 전 전체 nextest는 12,056개 통과·1개 실패·45개 건너뜀으로 종료했다(`output/issue8371/rebase-8165-full-nextest.log`); 이 집중 검사를 전체 최종 실행으로 대체하지 않는다.
