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

2022 원본의 기호 슬롯 한양신명조 FACE_NAME 원시 attr은 `0x81`, 2024는 `0x62`다.
HWPX 저장 API는 앞쪽 값을 TTF로 내보내지만, 이것을 원본의 유효한 TTF 프로그램 선언으로
단정하면 안 된다. 현재 시스템의 해당 family 후보 7개는 모두 CFF이며 원 TTF 별칭을
공급하지 못한다. 실제 CFF 등록은 두 문서 모두 성공한다. 기존 `f70a8c937d3dd56ac942d019dfca40c851e0a6a3`은 HWP5 사양 표 20에 유형으로
정의되지 않은 하위 비트로 TTF를 확정하지 않도록 `source_font_is_ttf`도 해제한다.
따라서 이 정상 정책에서 HFT/TTF 플래그가 모두 false일 수 있다. 실제 등록 프로그램 대신
이 두 플래그만으로 CFF 소비를 제한하면 같은 윤곽을 사용하지 못하고 Canvas 래스터로 되돌아간다. TTF 선언을 HFT로 바꾸거나 원본을 수정해서 해결하지 않는다.

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


## R35 실물 실패를 통한 범위 보완

R35 제품 `71ce4b7ae5`의 실제 최적화 WASM은 HFT·TTF 합성 대조와 실제 TTF 별칭 우선권,
Screen·옛한글 집중 검사 3건을 통과했다. 그러나 실물 2022 HWP 51쪽은 여전히 99.90%였다.
HWPX 저장 타입만 보고 TTF 선언으로 단정한 분석을 원본 바이트·공개 `getFontDecisionTrace`·
위 선행 Git 커밋으로 바로잡았다. 문서의 FACE_NAME attr은 0x81이고 공개 trace의
layoutMetric은 원 HFT 메트릭을 유지한다. 이 미확정 상태는 잘못된 리베이스가 아니라 기존 정책이다.

미확정 source 타입을 사용하는 저장소 소유 CFF 대조군도 R35에서 실제 FAIL했다.
정상 HFT/명시 TTF와 구분하여 이 반례를 추가한다. 후속 보정은 이미 실제 FontFace와
연결된 CFF 프로그램을 조회하며 원본 유형을 다시 추정하지 않는다. 실제 원 TTF 별칭이
해당 glyph를 소유하는 경우의 우선권은 유지한다. 원본 바이트·글꼴 출처·조판 폭은 바꾸지 않는다.
`native_pua_host_font_protocol::unassigned_hwp5_font_bits_allow_source_probe_without_truetype_declaration`
정책 검사도 그대로 유지한다. 합성 반례의 통과를 실물 문서 통과로 표현하지 않으며,
새 제품의 실제 문제 쪽·2024 대조군·전쪽 TSV가 완료된 뒤 최종 수치를 기록한다.


## R36 실제 문제 쪽 복구 확인

`3f2a7c96ee1ec86128b6ca1c3a6b700aa2eaeb3b`의 새 최적화 WASM에서 원본 그대로의
2022 HWP 51쪽은 **99.90% → 100.00%**로 복구됐다. 같은 독립 Print·글꼴 환경을
유지했다. 원 기호 두 개의 윤곽·기준선과 제목·밑줄, 뒤 표·본문을 실제 같은 쪽의
review/overlay에서 직접 판독했다. [실제 51쪽 TSV·PNG·재현 조건](protected-r36/sample16-hwp2022/README.md).

실제 브라우저의 3개 집중 검사도 모두 통과했다. HFT·명시 TTF·미확정 타입 3조건에서
등록된 CFF 소비를 확인하고, 실제 원 TTF 별칭이 있을 때의 출력 보존과 Screen·옛한글
대조군도 유지한다. 조건별 반복을 별개의 테스트 함수 수로 합산하지 않는다.
이는 아직 진행 중인 2024 대조군·전쪽 Native/fresh WASM·기본 전체 Rust·lint의
최종 통과를 의미하지 않는다.

## R36 세 원본의 전쪽 확인

실제 R36 fresh WASM에서 2022 HWP·2024 HWP·HWPX의 각 64쪽, 합계 192쪽을 완료했다.
최저값은 각각 100.00%·99.99%·99.99%이며 기존 두 보호 PR의 같은 192행 표시값 하락은 0행이다.
[Native/fresh WASM의 실제 전쪽 TSV와 해시](protected-r36/sample16-full-provenance.tsv)를
입력별 README에 연결했다. 이 전쪽 결과와 별도로 2022 51쪽·2024 40/51쪽의 직접 판독,
실제 원 TTF 별칭 우선권·CFF 소비 브라우저 대조도 확인했다.
27개 PR 전체의 나머지 문서·회귀·lint 완료는 아직 진행 중이다.
