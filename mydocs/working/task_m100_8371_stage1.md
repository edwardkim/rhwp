# #8371 1단계 — 로컬 글꼴과 HWP3 조판 분석

- 작업 기준: `f125c735c7d2d44eaa44b78c70465bc89af45fb1`, 브랜치 `fix/issue8371-installed-hft-font`.
- 상태: Native 전쪽 검증과 Studio 단위 테스트를 수행했다. fresh WASM 시각 검증과 전체 Rust 회귀 검증은 진행 중이며 PR 제출 준비 완료가 아니다.
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
