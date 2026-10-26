# #8153: 한자 프로그램의 실제 전진과 뒤 글자 원점 보존

## 원본·사양·독립 관측

기존 두 편람 원본과 독립 2024 Print를 그대로 사용한다. HWP5 §4.2.6 표33·34의
언어별 글꼴·장평·상대 크기를 구분한다. 166쪽 원본 HWPX의 charPr426은 크기1270HU,
한글·한자 장평과 상대 크기 모두100%다. 원본 한자 슬롯의 KoPub바탕체 Medium을
한글936/1000em으로 대신할 근거가 없다. 사양 오류가 아닌 공통 메트릭 적용 오류다.

`40d36c5599`의 실제 Native 원점과 독립 Print를 대조했다.

| 위치 | 현재 Native x | 독립 Print x | 차이 |
| --- | ---: | ---: | ---: |
| 첫 `官` | 187.08000 | 187.19999 | -0.11999px |
| 다음 `印` | 202.92000 | 202.39954 | +0.52046px |
| 뒤의 `(영 제40조)` | 458.38667 | 455.84001 | +2.54666px |

독립 Print의 한자 전진은15.2px다. 실제 글꼴의 `cmap/hmtx`는900/1000em이며,
같은 크기의 한글은936/1000em이다. 현재 Native는 한자에15.84px를 적용하므로
한자 네 글자의 누적 차이가 뒤 원점 이동을 설명한다. 양쪽 원본 Native166쪽은43픽셀,
fresh DEV WASM HWPX는41픽셀이 미대응이다. 다른 페이지에 같은 상수를 덧붙이지 않는다.

## 기존 이슈·원 PR·통합 경로

- 기존 이슈[#8099](https://github.com/edwardkim/rhwp/issues/8099)의 본문·전체 댓글을 읽었다.
  원 PR은[#8111](https://github.com/edwardkim/rhwp/pull/8111)이며, 이후 통합
  [#8228](https://github.com/edwardkim/rhwp/pull/8228)의 `3e02fe2749866a8453af3eb3901b9c1a674cccaf`에 반영됐다.
- 그보다 앞선 원 PR[#7968](https://github.com/edwardkim/rhwp/pull/7968)의 원 기여
  `c18fb887f1e399253fa79966d0c50e48990867cb`는 통합`1fa244cd5f2c3df5e71d01497e06613f9565d8d2`다.
  당시 목적은 KoPubWorld 인식과 **한글**936 전진이다. 이를 한자에 확대 적용하지 않는다.
- 현재 고정 base에는 이후 통합의 한자 보정이 없다. 사용자 지시대로 upstream을 fetch·동기화하거나
  rebase하지 않는다. 해당 PR 본문·댓글·원 diff를 읽기 전용으로 대조했으며, 표 이어받기 등
  무관한 통합 변경을 함께 적용하지 않는다. 이 단계는 #8153 두 문서의 실제 남은 결함 보정이다.

## 공통 메트릭 보정과 반례

설치된 KoPub바탕·돋움 Light/Medium/Bold 여섯 원본 프로그램의 `cmap/hmtx`를 읽었다.
한자 U+4E00–9FFF·F900–FAFF의 실제 수록4888자와 coverage는 여섯 파일에서 같았다.
바탕 Bold의 여섯 글자만898·899·901이고 나머지는900이다. 해당 차이도 보존한다.
Unicode 블록 전체를900으로 추정하지 않고 실제 cmap 수록 비트맵과 전진 메타데이터를 사용한다.
KoPubWorld는 별도 프로그램이므로 이 데이터로 이름만 바꾸어 포함하지 않는다.

공통 `kopub_symbol_metrics` 조회가 확인된 한자 메타데이터를 소비하게 한다. 측정의
`kopub_char_advance_em` → `EmbeddedTextMeasurer` 전진 → 런 원점·배치 위치와 최종 paint가
같은 값을 사용한다. 글꼴 파일·윤곽·이름을 복사·수정하지 않는다. 관련 기록은 ignored
`continuous-text-shadow/kopub-hanja-all6-complete-metrics-proof.json`과
`p166-independent-exact-char-origins.json`에 보존한다.

수정 전 실제166쪽의 뒤 원점 검사와 한자 전진을 실패로 확인한 뒤, 같은 원본·동작을 재검증한다.
한글·ASCII·기존 기호·서로 다른 굵기·cmap 결측을 정상 대조로 둔다. Native/fresh WASM의
166쪽과 핵심 경계를 먼저 직접 비교하고 전쪽을 검사한다. 이전136쪽 보정의 고정 producer로
진행 중인 전쪽 검증과 새 source의 결과를 혼합하지 않는다. 전체1532조건100%·0픽셀,
필수 전체 검사와 PR 준비는 아직 완료되지 않았다.
