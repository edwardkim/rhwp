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
