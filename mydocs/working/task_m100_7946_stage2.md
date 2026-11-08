# #7946·#7855·#7856 2단계 — Studio PDF 직접 열기와 실제 Docker WASM 검증

- Issue: [#7946](https://github.com/edwardkim/rhwp/issues/7946), [#7855](https://github.com/edwardkim/rhwp/issues/7855), [#7856](https://github.com/edwardkim/rhwp/issues/7856)
- 기준: `4662c069e557e928ed7968d061121fde90b1f027`; 의존성 단계 `070503d3f` 이후 이번 변경
- 브랜치·작업공간: `task_m100_7946`, `/home/edward/mygithub/rhwp-issue7946`
- 증적: `output/pr-review/issue7946/` (ignored). 소스·실행 파일·패키지 해시는 `verification-state.json`에 연결한다.
- 판정: 아래 제품 연결·집중 검사·전쪽 시각 검증 충족. 2026-10-16 사용자의 직접 Studio 시각 판정 통과.
  최종 PR용 전체 Rust 회귀·Native Skia 3종과 원격 CI·PR·병합은 아직 수행하지 않았다.

## 1. 입력·저장 계약과 실제 소비 경로

기본 WASM에도 `importPdf()` export가 존재하므로 메서드 존재만으로 UI를 켤 수 없었다.
Rust의 실제 feature를 반환하는 `HwpDocument.supportsPdfImport()`를 추가했다. Studio의
capability helper는 이 응답이 boolean `true`인 경우만 PDF 입력을 허용한다. API가 없는
이전 bundle과 PDF-off 빌드는 기존 HWP/HWPX/HML 입력을 유지한다.

`main.ts`의 input/drop 및 `file-open-picker.ts`의 native/fallback picker가 같은 capability를
소비한다. PDF 바이트는 기존 `loadFile` → `loadBytes` → `WasmBridge.loadDocument` →
`new HwpDocument(bytes)` → `convertToEditable` → `initializeDocument` →
`CanvasView.loadDocument`로 전달된다. PDF→HWPX 임시 저장·재파싱은 없다. 기존 원자적 문서
교체를 재사용하므로 후보 생성 실패가 현재 문서를 해제하지 않는다.

새 좌표·높이를 생산하거나 덮어쓰는 분기는 추가하지 않았다. 실제 PDF 문서에서
`convertToEditable` 전후 12쪽 SVG 바이트가 모두 동일함을 별도로 확인했다
(`wasm/controls-editable/browser-controls.json`). parser/lowering·측정·pagination·paint의
규칙은 변경하지 않았으므로 분할/이어받기 경계 보정은 비해당이다.

PDF 원본 handle은 읽기·최근 문서·동일 파일 식별에 보존한다. `resolveSaveTarget`은 첫 저장을
별도 HWP/HWPX로 강제하고 실제 형식 선택 대화상자를 연결한다. 저장을 마친 출력 handle만
변환 완료로 표시하여 이후 저장에 재사용한다. `saveDocumentToFileSystem`의 기존 동일 파일
검사도 적용해 PDF가 `.hwpx`로 잘못 이름 붙여져 있어도 원본에 쓰지 않는다. PDF 가져오기
출처와 선택한 출력 형식을 혼동하지 않는다.

생성된 `rhwp-studio/public/rhwp.js` 변경은 새 capability 바인딩을 포함하므로 이번 범위다.
ignored WASM과 pkg는 source PR에 추가하지 않는다. 한국어·영어 저장 형식 안내와 실제
Docker 빌드 명령은 [개발 환경 안내](../manual/dev_environment_guide.md#studio에서-pdf-직접-열기-선택-기능)에 기록했다.

## 2. 구현·빌드·실제 실행 결과

| 검증 | 결과·증거 |
| --- | --- |
| 입력·첫 저장 계약 | 기존 코드에서 4 FAIL → 구현 후 PASS. PDF 허용·picker 노출·fallback accept·원본 handle 강제 분리의 assertion 실패였다 (`pdf-ui/contracts-before.log`, `contracts-after.log`) |
| 최종 집중 검사 | capability·파일 열기·File System Access·원본 쓰기 방지 23 PASS (`pdf-ui/focused-final.log`). 기존 HML 검사는 공통 오류 문구만 갱신 |
| Studio | 전체 2,021 PASS·2 skip, 타입 검사·최종 `npm run build` 성공. PWA service worker·60 precache 항목 생성 (`pdf-ui/studio-tests.log`, `types.log`, `build-final.log`) |
| WASM 바인딩·editor 경계 | 3 PASS (`pdf-ui/bindings-final.log`) |
| 의존성 소비 재확인 | 실제 CI 명령의 호환성 7 PASS, 최종 metrics JSON/summary 생성, VS Code compile 성공. 의존성 자체의 전후 동일 입력 분석·audit·package 증거는 [1단계](task_m100_7946_stage1.md)에 유지 |
| CI 배선 | frontend job에 의존성 호환성·metrics 실행과 lockfile cache key 추가. workflow 계약 4 PASS·정책/증거 검사 66 PASS·actionlint 성공. trigger/permission/required check 변경 없음 (`dependencies/workflow-*.log`, `actionlint.log`) |
| Rust 필수 lint | fmt, Native/WASM Clippy, workspace build, workspace/all-target Clippy, base 고정 suite manifest 검사 성공 (`rust/*.log`). Rust source-side unit test 변경 없음 |
| Native | 공용 `/home/edward/mygithub/rhwp/target/pr-review`에서 `--release --features native-skia,pdf-import` 빌드 성공. 검증 실행 파일은 `candidate/rhwp`로 보존 |
| Docker WASM | 같은 Docker 공용 target/cache로 default-off와 PDF-on을 각각 생성. locked wrapper로 루트 pkg와 public 동기화 성공 (`wasm/docker-build.log`) |
| 실제 Worker | 새 on/off 패키지에서 85 assertion·5개 실제 Worker, 12쪽 렌더/저장·요청 격리·한도 오류/취소·복구 검사 통과 (`wasm/runtime.log`) |
| 실제 Studio | 정식 열기 command·input·drop·Canvas 12쪽·첫 HWPX 저장·다음 저장·손상 입력 뒤 기존 문서 보존/복구 등 15항목 PASS, uncaught error 0 (`pdf-ui/browser-result.json`) |
| 기존 형식 | PDF on/off의 HWP 3쪽·HWPX 1쪽·HML 1쪽 SVG 전부 동일. PDF-on Studio input에서 세 문서 연속 교체 및 Canvas 쪽수 확인, 오류 0 (`wasm/controls*`, `pdf-ui/existing-format-ui.json`) |

브라우저의 native 파일 선택기·출력 handle은 자동 검사에서 제어한 대조군이다. 실제 OS
대화상자의 수동 조작 검증이라고 표시하지 않는다. 앱 command/input/drop·제품 parser·Canvas는
실제 Chrome에서 실행했다. 최초 WASM 응답 읽기의 CDP inspector cache eviction은
`browser-inspector-before.log`에 보존하고, browser fetch 사본의 SHA로 실제 수신 바이트를 확인했다.
형식 대조군은 파일명 갱신보다 늦은 Canvas 초기화까지 기다려 판정했다. 이 두 진단 문제를
제품 회귀로 세거나 코드/빌드를 반복 변경하지 않았다.

## 3. 새 패키지·독립 기준과 전쪽 시각 검증

입력과 독립 기준은 기존 한컴 Print
`pdf/issue5875/nested_table_text_caption-2020.pdf`다. Creator `Hwp 2020 0.0.0.0`, Producer
`Hancom PDF 1.3.0.550`, 12쪽·525,733 bytes이며 SHA-256은
`6308671fc56b1bfd9e72660edd6edf574a4e16f75e84e0bf35399339d9ce3702`다.
동일 PDF를 직접 가져온 결과를 이 원본 raster와 비교했으며 이전 HWP 저장본 검증을 대체 증거로 쓰지 않았다.

| 산출물 | SHA-256 |
| --- | --- |
| 새 pkg/public JS (동일) | `b18f02947b225d0e180b7e95fdb4cc9ecbd714aa462a67888f2cb891ff6dbdf9` |
| 새 pkg/public WASM (동일) | `de814d24b0d56924259e7ac2e277bedf57d0dbd6d5a91e067b32b097423d2246` |
| 새 Native 검증 CLI | `929dd4f8f5f5d47ac4bb335d5f730723685e8490ebc9e1d511e925248d8484a5` |

실제 Studio가 fetch한 WASM도 위 hash와 일치한다. 원본 글꼴은 기존 검증 경로의
`HANBatang.ttf`·`HANBatangB.ttf`를 실제 공급했다. 경로/바이트 hash는 sweep manifest에 남기고
글꼴 자체는 커밋하거나 검증 폴더의 공개 사본에 넣지 않았다.

| 직접 PDF 입력 | 전쪽 TSV | 전체 쪽수 (rhwp / PDF) | 최저 2px 내용 실루엣 | 미달·누락 |
| --- | --- | --- | --- | --- |
| 새 Native Skia | `native-scores/pdf/silhouette.tsv` | 12 / 12 | 90.14609% (11쪽) | 0 / 0 |
| fresh Docker WASM Canvas | `wasm-scores/pdf/silhouette.tsv` | 12 / 12 | 90.16832% (11쪽) | 0 / 0 |

명령은 작업공간에서 다음과 같이 실행했다. 전체 명령·실행 파일/패키지 해시는 각 log와
`run_manifest.json`에도 보존한다.

```bash
/home/edward/mygithub/rhwp/venv/bin/python scripts/visual_sweep.py --silhouette-only \
  --hwp pdf/issue5875/nested_table_text_caption-2020.pdf \
  --pdf pdf/issue5875/nested_table_text_caption-2020.pdf --key pdf \
  --rhwp-bin output/pr-review/issue7946/candidate/rhwp --compat 2022 --dpi 96 \
  --embed-fonts --font-path /home/edward/mygithub/rhwp-issue7911/output/pr-review/issue7911/hwp-save-20261016/p11/fonts \
  --page-workers 2 --out output/pr-review/issue7946/native-scores
```

WASM은 `--wasm-pkg pkg`를 추가해 `wasm-scores`에 산출했다. Native 빌드가 끝나기 전의
WASM 실행은 기존 #7913의 PDF-on CLI를 note-shape/글꼴 메타데이터 helper로만 사용했다
(해시 `28bd5498…b3ca9b3`, 정확한 경로는 manifest). 실제 12쪽 그림·텍스트·render tree는
새 Docker WASM에서 생성했다. 이 helper를 새 Native 검증으로 보고하지 않으며, Native TSV는
이후 새 CLI로 따로 산출했다.

1·3·11쪽은 별도 `native-review`/`wasm-review`에 compare·standalone overlay·review PNG를
생성했다. 대표 일반 모드의 gate도 양쪽 `passed`이며 전체 쪽수는 별도 export metadata로 12쪽을
대조했다. TSV 전용 모드의 `not_evaluated`를 승인 결과로 바꿔 읽지 않는다.
직접 판독에서 1쪽 표 외곽·그림/본문 순서, 3쪽 오른쪽 crop·캡션/다음 행 경계,
11쪽 그림 9/10·캡션 표시를 확인했다. 기존 문자 간격·raster 차이는 남지만 새로운
흐름/좌표 보정은 추가하지 않았다. 사용자는 실제 Studio에서 시각 판정 통과로 응답했다.

## 4. 확인 경로와 남은 절차

- Studio: `http://localhost:7791/?review=issue7946-pdf-direct&lang=ko`
- Windows 샘플: `E:\tmp\issue7946-studio-pdf-20261016\nested_table_text_caption-2020.pdf`
- 같은 폴더의 `review/index.html`: Native/WASM 대표 비교 PNG, 전쪽 TSV
- 원본·JS/WASM·CLI·실행 기록은 위 작업공간의 ignored 증적에 보존한다.

현재 결과는 Studio 채택과 두 의존성 보정의 로컬 결과다. 최종 PR 준비 때 정본의 전체
Rust 회귀·Native Skia 3종 및 code head/최신 devel 정합을 확인한 후, 별도 승인된 원격
push·한국어 통합 PR·CI·검토/병합 절차로 진행한다. 이번 단계에서 원격 변경은 하지 않았다.
`parserAllocationBoundsProven=false`, 기존 B01–B07·B09와 B10-1의 미완료 범위는 유지한다.
전체 Worker 편집기 전환·npm/확장 배포·PDF 기본 활성화는 완료했다고 표시하지 않는다.
