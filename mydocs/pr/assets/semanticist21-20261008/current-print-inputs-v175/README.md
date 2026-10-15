# 현재 편집 저장본의 검증 입력

기존 `print_reference_provenance.json`과 누적 리뷰의 진행151–154에서 실제 편집 절차,
원본·독립 한컴2020 Print·Native/fresh 출처를 연결한다. 이 파일들은 편집 전 원본의
대체물이 아니라 해당 편집 동작이 만든 저장본이다.

## 각주 스타일이 없는 원본의 편집 저장본 보존 보완

`additional-edited-note-style-missing-named-style-v175.hwpx`는 기존 검증에서 사용한
10443바이트 입력 그대로이며 SHA-256은
`6b4f6a375029dcb0c9def5b4443683d1b55cf09acbdb9cb9ab87ac2d94888edb`다.
스타일 이름이 없는 `hwp3-sample19-hwp5.hwp`에 새 각주를 넣고 저장한 실제 편집 결과다.
검증용 output 사본만 존재하던 누락을 보완했으며 새로 재생성하거나 내용을 바꾸지 않았다.

기존 기준은 [같은 편집 저장본의 한컴2020 Print PDF](../../../../../pdf/semanticist21-20261008/new_note_paragraph_style-0462-6b4f6a375029-2020.pdf)이며
SHA-256은 `778594c4e099ec86ddaa825f1dd307c195c4ffe49b842a5082df0c15274d6828`다.
이 PDF는 두 쪽이며, 독립 출력에서 `location.replace("URL")` 사용 예는 첫 쪽에,
`navigator` 제목과 마지막 `screen` 속성 목록은 둘째 쪽에 있다.

현재 production source에 대응하는 Native v315와 최적화 fresh WASM v323의 실제 브라우저
전쪽 v322/v326은 두 쪽 모두 `95.47353% / 92.07644%`이고 decoded RGB도 같다(v338).
실행 명령·source/font/package/input/PDF 해시는 ignored
`output/pr-review/semanticist21-20261008/wrapper-cell-note-native-fresh-completion-v338.json` 및
연결한 각 실행 기록에 있다. v329 확대 판독과 v339의 새 fresh review는 본문 끝·다음 쪽
소속을 확인한다. 원래 잘못 나뉜 세 쪽이나 정상 한컴 재저장 대조군으로 이 입력을 바꾸지 않는다.

정식 검사는 `tests/cases/footnote_stored_width_body_ownership.rs`에서 실제 Native 인쇄 SVG의
쪽 소속·내용 순서·보존을 검사한다. 글꼴 raster와 기존 가는 선의 잔여 차이, 별도 차트와
직접 보정 세 문서·전체 회귀·backend의 미검증을 이 검사 통과로 해소했다고 보고하지 않는다.

## 2026-10-13 최종 검증 입력 누락 보완

최종 재검증 준비에서 아래 세 편집 저장본이 ignored output에만 있고 커밋된 입력에
없음을 확인했다. 기존 독립 Print를 생성한 입력 바이트 그대로 보존하며 새로 재생성하거나
기준 PDF를 바꾸지 않았다. 이 보존은 현재 소스의 시각 통과 판정이 아니며,
7749d의 Native/fresh 재검증을 이어서 수행한다.

| 입력 | SHA-256 | 같은 입력의 독립 Print |
| --- | --- | --- |
| `additional-edited-note-style-blank-endnote-v175.hwpx` | `31414e6363d510ce28bdbb933b24431ac1f2642f7d65b4ef219c74340a3e973d` | `pdf/semanticist21-20261008/new_note_paragraph_style-0460-31414e6363d5-2020.pdf` |
| `additional-edited-note-style-blank-footnote-v175.hwpx` | `56839e167e1177d8c29b636a93508e0d59e7874e5242fc8ae6cb6c320a361d3d` | `pdf/semanticist21-20261008/new_note_paragraph_style-0458-56839e167e11-2020.pdf` |
| `additional-edited-note-style-hancom-hwp-footnote-v175.hwpx` | `b3c9896ea898d3b3f9185b473a7328bc53ac886c0f607221a9df28b7d483f5ad` | `pdf/semanticist21-20261008/new_note_paragraph_style-0464-b3c9896ea898-2020.pdf` |
