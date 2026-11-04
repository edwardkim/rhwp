# #8153: 한자 프로그램의 실제 전진과 뒤 글자 원점 보존

## 원본·사양·독립 관측

기존 두 편람 원본과 독립 2024 Print를 그대로 사용한다. HWP5 §4.2.6 표 33·34의
언어별 글꼴·장평·상대 크기를 구분한다. 166쪽 원본 HWPX의 charPr 426은 크기 1270HU,
한글·한자 장평과 상대 크기 모두 100%다. 원본 한자 슬롯의 KoPub바탕체 Medium을
한글 936/1000em으로 대신할 근거가 없다. 사양 오류가 아닌 공통 메트릭 적용 오류다.

`40d36c5599`의 실제 Native 원점과 독립 Print를 대조했다.

| 위치 | 현재 Native x | 독립 Print x | 차이 |
| --- | ---: | ---: | ---: |
| 첫 `官` | 187.08000 | 187.19999 | -0.11999px |
| 다음 `印` | 202.92000 | 202.39954 | +0.52046px |
| 뒤의 `(영 제40조)` | 458.38667 | 455.84001 | +2.54666px |

독립 Print의 한자 전진은 15.2px다. 실제 글꼴의 `cmap/hmtx`는900/1000em이며,
같은 크기의 한글은936/1000em이다. 현재 Native는 한자에 15.84px를 적용하므로
한자 네 글자의 누적 차이가 뒤 원점 이동을 설명한다. 양쪽 원본 Native 166쪽은 43픽셀,
fresh DEV WASM HWPX는 41픽셀이 미대응이다. 다른 페이지에 같은 상수를 덧붙이지 않는다.

## 기존 이슈·원 PR·통합 경로

- 기존 이슈[#8099](https://github.com/edwardkim/rhwp/issues/8099)의 본문·전체 댓글을 읽었다.
  원 PR은[#8111](https://github.com/edwardkim/rhwp/pull/8111)이며, 이후 통합
  [#8228](https://github.com/edwardkim/rhwp/pull/8228)의 `3e02fe2749866a8453af3eb3901b9c1a674cccaf`에 반영됐다.
- 그보다 앞선 원 PR[#7968](https://github.com/edwardkim/rhwp/pull/7968)의 원 기여
  `c18fb887f1e399253fa79966d0c50e48990867cb`는 통합 `1fa244cd5f2c3df5e71d01497e06613f9565d8d2`다.
  당시 목적은 KoPubWorld 인식과 **한글**936 전진이다. 이를 한자에 확대 적용하지 않는다.
- 현재 고정 base에는 이후 통합의 한자 보정이 없다. 사용자 지시대로 upstream을 fetch·동기화하거나
  rebase하지 않는다. 해당 PR 본문·댓글·원 diff를 읽기 전용으로 대조했으며, 표 이어받기 등
  무관한 통합 변경을 함께 적용하지 않는다. 이 단계는 #8153 두 문서의 실제 남은 결함 보정이다.

## 공통 메트릭 보정과 반례

설치된 KoPub바탕·돋움 Light/Medium/Bold 여섯 원본 프로그램의 `cmap/hmtx`를 읽었다.
한자 U+4E00–9FFF·F900–FAFF의 실제 수록 4,888자와 coverage는 여섯 파일에서 같았다.
바탕 Bold의 여섯 글자만 898·899·901이고 나머지는 900이다. 해당 차이도 보존한다.
Unicode 블록 전체를 900으로 추정하지 않고 실제 cmap 수록 비트맵과 전진 메타데이터를 사용한다.
KoPubWorld는 별도 프로그램이므로 이 데이터로 이름만 바꾸어 포함하지 않는다.

공통 `kopub_symbol_metrics` 조회가 확인된 한자 메타데이터를 소비하게 한다. 측정의
`kopub_char_advance_em` → `EmbeddedTextMeasurer` 전진 → 런 원점·배치 위치와 최종 paint가
같은 값을 사용한다. 글꼴 파일·윤곽·이름을 복사·수정하지 않는다. 관련 기록은 ignored
`continuous-text-shadow/kopub-hanja-all6-complete-metrics-proof.json`과
`p166-independent-exact-char-origins.json`에 보존한다.

수정 전 실제 166쪽의 뒤 원점 검사와 한자 전진을 실패로 확인한 뒤, 같은 원본·동작을 재검증한다.
한글·ASCII·기존 기호·서로 다른 굵기·cmap 결측을 정상 대조로 둔다. Native/fresh WASM의
166쪽과 핵심 경계를 먼저 직접 비교하고 전쪽을 검사한다. 이전136쪽 보정의 고정 producer로
진행 중인 전쪽 검증과 새 source의 결과를 혼합하지 않는다. 전체 1,532조건 100%·0픽셀,
필수 전체 검사와 PR 준비는 아직 완료되지 않았다.

## 원본 프로그램 신원과 메타데이터 검사

| 설치 프로그램 | SHA-256 |
| --- | --- |
| `KoPub Batang Bold.ttf` | `94de26a11aa25619526ac8183fde4f46e74e22c527acb42aea2e93f5313bbc5b` |
| `KoPub Batang Light.ttf` | `b7438d61595fe9584706ab2cb3c9aa2acdc486d61c8299374a30b3e77bc2045f` |
| `KoPub Batang Medium.ttf` | `7a798592beb0d35bb557b010c2794cbcc988e89fcbcee0eb5edbb35161141ab4` |
| `KoPub Dotum Bold.ttf` | `967efe18063dd971c63dde3e8fcb247e0fcf16486a24ba2e284a716e44b90275` |
| `KoPub Dotum Light.ttf` | `018a0f18613a98d446696ea3d36194e288d5d8004aa58327789c830f6179e23d` |
| `KoPub Dotum Medium.ttf` | `d0adfdcb40b7e73389128cc0222b53a91e5aa0e653f89967485a1f8b46124eb7` |

현재 compact 메타데이터를 실제 여섯 프로그램의4888자씩과 다시 대조했다.
독립 `cmap/hmtx` 29,328항목이 모두 같고, 한글·ASCII·기존 기호·미수록 한자를
이 보충 표가 가로채지 않는 검사도 통과했다. 이는 메타데이터 검증이며 제품의 전쪽 시각
완료로 대신하지 않는다. 실제 166쪽 뒤 원점과 한자 전진은 수정 전 진단 3건 모두 실패했다.

## source `ffc7cf81b4` 집중 검증

`ffc7cf81b4c661490abd0bf061521db9a282a62d`의 Native 52조건을 완료했다.
두 형식의 166쪽은 모두 raw 100%·정수 미대응 0픽셀이고 136쪽도 그대로다.
52조건에서 미대응 증가가 없었다. 뒤 원점은 양쪽 모두 455.82667px다.
수정 전·후와 독립 Print의 같은 좌표 crop을 확대해 한자·괄호·뒤 본문을 대조했다.

기존 정상 검사 33건과 보정 대상 3건, 총 36건이 통과했다. 진단용 합성 전진 검사에는
잘못된 전제가 한 번 있었다. 100px 글자에서 hmtx의 연속값 90px를 장치 격자 반올림 없이
그대로 기대했다. 4HU 장치 격자에서 이는 90.02667px다. 임의 허용치를 늘리지 않고
한 인치(96px) 글자 크기의 정확한 hmtx 전진 86.4px로 입력·기대값을 정정했다.
정정한 같은 검사도 수정 전 source40에서 3건 FAIL, source `ffc7cf81b4`에서 3건 PASS였다.
최초 실패 로그와 정정 후 로그는 함께 보존했다.

증거는 공통 `continuous-text-shadow/`의 `hanja-native/contracts-corrected-result.json`,
`hanja-native/native-focus/result.json`, `hanja-corrected-before40.log`,
`hanja-corrected-afterffc7.log`와 `p166-hanja-independent-word-origins-review.png`다.
전쪽 Native 766조건과 fresh DEV WASM 55조건을 이 source로 새로 실행 중이며,
새 source 전쪽·최종 표준 최적화 WASM·전체 Rust·lint 완료는 아직 주장하지 않는다.

## source `ffc7cf81b4` 전쪽 Native 완료와 166쪽 양쪽 backend

Native 전 766조건을 완료했다. HWPX는 363/383조건, HWP는 360/383조건이
raw 100%·정수 미대응 0픽셀이다. source40의 719조건에서 723조건으로 늘었고
미대응은 2,216 → 2,124픽셀로 줄었다. 두 형식의 166쪽 43픽셀·167쪽 3픽셀이
각각 0으로 바뀌었으며 미대응 증가 페이지는 없다. 최저 raw는 양쪽 99.46283%다.
Native 전쪽 source40·source `ffc7cf81b4` 결과를 다른 source WASM과 합산하지 않는다.

같은 source `ffc7cf81b4`의 fresh DEV WASM에서 두 형식의 166쪽도 100%·0픽셀을 확인했다.
원본 26쪽씩의 TSV를 완료했으며 탭 대조군까지 포함한 최종 집중 실행은 별도 확인한다.
정식 회귀 `tests/cases/issue_8153_hanja_source_advance.rs`는 96dpi 한 인치 글자의
실제 900/1000em 전진과 166쪽의 원문·줄 소유·뒤 글자 원점 ±2px를 함께 보호한다.
별도 글꼴 설치 경로나 글꼴 파일을 테스트에 포함하지 않는다. 최종 자동 수집·전체 suite는
후속 source의 필수 검증으로 남는다.

정식 한자 검사 파일을 새 공통 fit source `9bab87505c`의 동결 라이브러리에 실제 연결해
3건 PASS를 확인했다. 이전 진단 파일과 독립 기대값·검사 동작은 같으며, 파일 해시와
실행 기록은 `plain-fit-actual-advance-contract.log` 및 해당 contracts 결과에 남긴다.
이 검사 등록으로 전쪽 WASM·전체 Rust 검증을 완료 처리하지 않는다.
