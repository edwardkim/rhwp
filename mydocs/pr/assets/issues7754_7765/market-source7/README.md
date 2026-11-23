# 시장구조조사 소스 7의 전체 315쪽 검증

이 기록은 공통 보정을 커밋한 `a21e590a7e76e7c6d4be6205001d29f4a96c3e9c`와
생산 소스가 일치하는 고정 Native·최적화 fresh WASM의 실제 결과다. 9개 이슈의 최종
head 검증이나 PR 제출 완료를 뜻하지 않는다. 후속 HWPX 보정 뒤 최종 검증도 수행한다.

## 입력과 생산자

- 원본: `samples/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사.hwp`.
- 원본 SHA-256: `84d297375c48e1e903247a389e9ec3907e3cdc57a4a7e29a819a01c2a1e3a341`.
- 독립 Print: `pdf/issue7755/market-2017-manual-print-2024.pdf`.
- PDF SHA-256: `89e32b43af21bde3cf869e76b816d214fa0bc30e3f14512c4c3af184430cd616`.
- 사용자가 win10-ted의 `C:\Users\admin`에 제공한 원래 파일을 복사했다. MCP 산출물로
  표시하지 않는다. Hwp 2024 13.0.0.3901, Hancom PDF 1.3.0.550, 315쪽 A4, 정상 100% 용지 배율이다.
- Native SHA-256: `89254cbba9084435ca05bf775e0578fee7a2a17399b364e5be567d7a38ab502c`.
- 최적화 WASM SHA-256: `dca92c879f0d034a05175cec08aa9496e52f19d4b03394a614e49ea0ac042cde`.
- WASM JS SHA-256: `23130c8002fe1aa4e6545910d92ef5716ccb92097747662d2b64ef129ff27b08`.
- 빌드 시작 parent는 `24bfabd575b98dd998d69492f8062cb2fdbe3269`다. 빌드에 동결한
  생산 소스 17개는 아래 해시로 a21e590 커밋과 각각 일치함을 확인했다.
- Native 빌드 exit 0, 508.7초. 최적화 WASM 빌드 exit 0, 766.8초.

## 측정과 전체 범위

2026-11-21, Print, compat 2022, 96 DPI, PDFium RGB 평활화, 반경 2px,
색상 경계 보정 전 `silhouette_raw_match_percent`다. 평균으로 미달 쪽을 상쇄하지 않는다.

| 경로 | 쪽수 | 전쪽 최저 raw | 100.00% 쪽 | 실제 종료 |
| --- | ---: | ---: | ---: | --- |
| Native | 315/315 | 98.40% | 211 | exit 0, 1363.45초 |
| 최적화 fresh WASM | 315/315 | 98.40% | 212 | 10개 부분 실행 모두 exit 0, 병합 exit 0 |

Native는 1–315 전체를 한 번의 sweep으로 측정했다. WASM은 같은 고정 패키지로
1–32, 33–64, 65–96, 97–128, 129–160, 161–192, 193–224, 225–256,
257–288, 289–315를 실행했다. 쪽 누락·중복 없이 전체 315쪽임을 확인했다.
최종 부분 실행 종료는 `2026-11-21T13:05:49.523124+00:00`다.
부분 실행당 page workers 2, 동시에 2개 부분을 실행했다. 글꼴 제한을 완화하지 않았다.

[Native 전쪽 TSV](native.tsv), [WASM 전쪽 TSV](wasm.tsv),
[쪽별 점수와 Native·WASM·PDF PNG SHA-256](page-evidence.tsv)를 보존한다.
원래 부분 TSV는 `wasm-chunks/`에 보존한다. Native와 WASM의 PDF raster 315개도
전부 바이트까지 같았다. Native Python은 `/usr/bin/python3.12`, WASM은 저장소 venv이며,
둘 다 PDFium `152.0.7947.0`, wrapper `5.12.1`을 사용했다. 서로 다른 PDF 배율·래스터를
사용해 점수를 섞지 않았음을 실제 PNG 해시로 확인했다.

재현 명령에서 아래 환경 변수를 로컬의 동결 산출물 경로로 지정한다. 빌드 당시 산출물과
같은 SHA를 사용하고 현재 작업 트리의 바이너리로 조용히 바꾸지 않는다.

```bash
venv/bin/python scripts/visual_sweep.py --silhouette-only \
  --hwp 'samples/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사.hwp' \
  --pdf pdf/issue7755/market-2017-manual-print-2024.pdf --key issue7755 \
  --rhwp-bin "$MARKET_NATIVE" --compat 2022 --dpi 96 --page-workers 2 \
  --out output/market-native
# WASM은 각 부분 페이지에 대해 실행한다. 예: 첫32쪽.
venv/bin/python scripts/visual_sweep.py --silhouette-only \
  --hwp 'samples/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사.hwp' \
  --pdf pdf/issue7755/market-2017-manual-print-2024.pdf --key market-001-032 \
  --pages "$(seq -s, 1 32)" --rhwp-bin "$MARKET_NATIVE" --wasm-pkg "$MARKET_WASM_PKG" \
  --compat 2022 --dpi 96 --page-workers 2 --out output/market-wasm
```

## 직접 확대 판독과 회귀 쪽 선정

| 쪽 | Native / WASM raw | 같은 영역의 독립 Print 확대 판독 |
| --- | --- | --- |
| 3 | 99.37 / 99.32% | 제목 공백, 원본 목각 소제목, 이름·KDI 글리프·기준선·간격. 소제목의 미세 차이가 남는다. |
| 4·5 | 각각 100.00 / 100.00% | 목차 마지막 줄과 다음 쪽 첫 두 줄. 4쪽 바닥 아래에 5쪽 목차를 얹지 않는 소속·내용 순서를 확인했다. |
| 94·95 | 각각 100.00 / 100.00% | 표 Ⅵ-5의 머리행 실제 높이·첫 조각·반복 머리행·마지막 본문 행과 다음 절 제목. 전체 행 선언을 첫 조각 개체 높이로 축소하지 않는 것을 확인했다. |
| 100 | 100.00 / 100.00% | 제3절 소결 앞 빈 줄 점유, 제목과 다음 본문 순서·간격. base에서 제목·본문이 위로 당겨졌던 차이가 해소됐다. |
| 246 | 99.97 / 99.97% | 앞 표의 마지막 행·외곽, 실제 남는 빈 공간. 다음 부록을 이 쪽으로 당기지 않는다. |
| 247 | 100.00 / 100.00% | 부록 B-2 제목·단위·반복 머리행·첫 본문 행. base의 다른 내용 대응이 정식 쪽 소속으로 돌아왔다. |
| 315 | 100.00 / 100.00% | 마지막 표의 전기·전자 및 정밀기기 수리 행, 수치, 외곽 종료. 누락·불필요한 다음 쪽을 확인했다. |

위 쪽의 Native·WASM·독립 Print를 동일한 96 DPI 영역에서 2배로 직접 열어 대조했다.
100·247쪽은 base Native와 후보·독립 Print도 같은 영역에서 대조했다.
최종 회귀는 변경 동작을 검증하는 100% 쪽을 우선하고 246쪽 종료 경계를 보완한다.
쪽수만 고정하지 않고 문단·표의 소속, 내용 순서·보존, 포함·겹침을 함께 검사한다.
정식 회귀 복원과 최소 글꼴 실행은 `mydocs/working/task_m100_7754_stage3.md`에 기록한다.
최종 보정 head의 전체 검증과 PR 제출 완료를 뜻하지 않는다.

## 병합 뒤 이슈 코멘트에 사용할 대표 이미지 계획

#7755 해결 근거로 실제 전쪽 수치와 다음 이미지 5개만 연결한다. 원시 페이지 PNG·탐색용
중복 이미지는 커밋하지 않는다. 다른 이슈 해결 증거로 재사용하지 않는다.

- [3쪽 세 출력 확대 비교](p003-review.png): 원본 글꼴 공백과 글리프.
- [100쪽 base·후보·독립 Print](p100-before-after-print.png): 빈 줄 뒤 제목과 본문.
- [247쪽 base·후보·독립 Print](p247-before-after-print.png): 부록 시작·표 소속.
- [315쪽 Native·WASM·독립 Print](p315-review.png): 마지막 행과 표 종료.
- [94–95쪽 실제 표 분할과 머리행](p094-095-table-fragments.png): 원본 행 선언·반복 머리행·마지막 행과 다음 절 제목.

## 생산 소스와 최종 TSV SHA-256

| 파일 | SHA-256 |
| --- | --- |
| `src/paint/builder.rs` | `f78da185d4a4a32e5bc6c35f58d97747b130f7a9371e4db66036d29ba602404a` |
| `src/renderer/image_resolver.rs` | `28522ca3191d684ed7f868bccd2b932bde389f2e372f1a9727791dd7c99c5923` |
| `src/renderer/image_resolver/print_source_codec.rs` | `eee84a66803b6309ff45fe46a3f0b5c58fedbae7335185d084a02f058d014fea` |
| `src/renderer/inline_flow.rs` | `6d09685f1b1855392f85df330464a13498e1f3fe729b1975f1ffd9884a632f14` |
| `src/renderer/layout/paragraph_layout.rs` | `22aefd7a418df44b5b85a43d5139a596ca45c099ce00f011c482b5beafe96e4a` |
| `src/renderer/layout/paragraph_layout/stored_wrap_frame.rs` | `3125b5068f46c846c4249b56aad52bf91fc6bbb7de89ddb1d6cfb36b11f80be2` |
| `src/renderer/layout/text_measurement.rs` | `eec93aed747ccb60c1be0a58b1c3f997e2f3d5596ed3cccc10e7f841b51ab317` |
| `src/renderer/native_pua_fonts.rs` | `a6cb411454b50547b43b53950cd05336b15c197ccf617a695db039307739b6e4` |
| `src/renderer/pagination.rs` | `e537e5cb677e25db90baaea80c74d224cbaf71ca4f0efbb23eb0f5e4c359d049` |
| `src/renderer/pagination/engine.rs` | `20dcdf577b1b40afed24d4bfabfc3bc633ceb28abb7474b9112587df791a3f40` |
| `src/renderer/pagination/stored_empty_prefix.rs` | `0147d2843aa8a0fe9f5413054ac761dc9bc74e60a067900dc5f6dda188b93df5` |
| `src/renderer/render_tree.rs` | `4b75c379c58f5f1068a1a6e36100b92b1c8ad8da7f84cc2a9456c3494045b5bd` |
| `src/renderer/style_resolver.rs` | `32d8dae087339c8fa438030b72b485315b1d2a3630552bb139595ca99a05ebca` |
| `src/renderer/svg.rs` | `121ff0252824afa518a4aa60c344e2468fea4aff4383dcc0b77e89f965311388` |
| `src/renderer/typeset/paragraph.rs` | `50e4da706eb5dc8d7da2a893d87b5c146af877f26f5652f2af5a9fa8b6ae030a` |
| `src/renderer/typeset/paragraph/empty.rs` | `b992c98b19de38d0019f32092108ab1e9dbdaf154e001c459dc7446ea5d05e8a` |
| `src/renderer/typeset/paragraph/flow.rs` | `8a60e921e1874fb54718c89bd0d4d0f961c1a2ec923bff3f7d6124376eccd6bc` |
| `native.tsv` | `539562fe370c1da57c258c31c24f1f14ac00f96045295a5da8b0952a0c5a35d2` |
| `wasm.tsv` | `5ddbea24d26e328cf51c79396366c03f51b87b4639c338f28b71e997cd63132a` |
| `page-evidence.tsv` | `f10ad0bca046ed0e141481d08c9d3c75292fea029e2f1db19458083b4c03f101` |
