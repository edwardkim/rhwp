# 원 PR의 글꼴 프로그램 선택과 TTF 선언 대조

2026-11-07, `1262a65fe9`의 실제 최적화 WASM을 기준으로 조사했다. 독립 PDF와 글꼴
프로그램을 다시 생성하거나 저장소에 추가하지 않는다.

## 선행 해결과 현재 소비 경계

- [PR #8397](https://github.com/edwardkim/rhwp/pull/8397)의 내부 커밋
  `730759eb2c3820f061cb2cf4f86215c96cedc548`은 Native Skia의 분리 OTF 선택을
  보정했다. 통합 커밋 `99a176b7cb34fde54454c03433baa1382b2de9fa`와 현재
  `src/renderer/skia/font_lookup.rs`에 해당 로직이 남아 있다.
- [PR #8583](https://github.com/edwardkim/rhwp/pull/8583)의 `96827b3acc`는 WASM
  Print TTF 보관소를 추가했다. [PR #8605](https://github.com/edwardkim/rhwp/pull/8605)의
  원 커밋 `4f8570309e925468552036f4c20d099011d55a35`, 현재 적용 `14ae66657f`는
  이 보관소에서 실제 보통 face의 윤곽을 얻어 합성 굵기에 fill+stroke를 적용한다.
- R34 `0fc490410d`는 이미 등록된 원 HFT CFF를 이 경로에 연결했다. 실제 2024 HWP와
  HWPX 40·51쪽은 100.00%로 복구됐으나 2022 HWP 51쪽은 99.90%가 남았다.

## 남은 하락의 독립 근거와 원인

원본 `samples/hwp3-sample16-hwp5-2022.hwp`와 독립
`pdf/hwp3-sample16-hwp5-2022-hwp-2020.pdf`를 사용했다. PDF 51쪽 T12와 2024 PDF의
T9는 모두 U+25CB를 HFT328에 연결하며, 글리프의 같은 cubic 윤곽과 `20 w B`를 보존한다.
현재 같은 영역의 Native는 기준을 충족하고 WASM의 두 원 기호만 크게 그려진다.

공개 저장 API로 확인한 2022 파일은 기호 슬롯의 한양신명조를 TTF로 선언하고, 2024는
HFT로 선언한다. 현재 시스템의 해당 family 후보 7개는 모두 CFF이며 원 TTF 별칭을
공급하지 못한다. 실제 CFF 등록은 두 문서 모두 성공한다. 따라서 선언만 검사하는
`source_font_is_hft` 조건 때문에 2022는 같은 공급 윤곽을 소비하지 못하고 Canvas
글꼴 래스터로 되돌아간다. TTF 선언을 HFT로 바꾸거나 원본을 수정해서 해결하지 않는다.

소비 경로는 실제 FontFace 공급 → 등록된 CFF/원 TTF 별칭과 cmap →
`HftOutlineFonts::faux_bold_glyph` → `draw_faux_bold_outline_with_color`의 동일한
fill+stroke다. 실제 TTF 별칭이 해당 글자를 소유하면 우선권을 유지한다. 그 별칭을
공급하지 못하고 같은 family의 CFF가 실제 등록된 경우만 기존 CFF 경로에 연결한다.
Screen·실제 기울임·옛한글 우선 face·기존 전진폭은 유지한다. 문서 번호나 글자별 좌표를
분기에 넣지 않는다.

저장소 소유 CFF fixture를 HFT/TTF 언어 슬롯에 각각 연결한 실제 브라우저 반례는
보정 전 R34에서 HFT는 통과하고 TTF는 Print `fillText('가')`가 남아 실패했다.
반대 대조군은 OFL Noto Sans KR을 원 TTF 별칭으로 직접 공급해 CFF 등록 전후의 실제
출력을 비교한다. 이 합성 검사는 실제 문서의 전쪽 TSV와 직접 판독을 대체하지 않는다.
최종 통과 수치와 원 PR별 판정은 실제 보정 후 재검증을 완료한 뒤 추가한다.
