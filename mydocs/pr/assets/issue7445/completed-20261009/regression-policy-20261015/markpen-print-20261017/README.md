# 생물독소146쪽 RangeTag 형광펜의 화면·인쇄 분리

## 독립 근거

원본 `samples/issue5941/1480000-201900698-native-neartop-reset.hwp` SHA-256은
`18765ac22270a2cb4ece6a1e6539c2dea6ca6ccde2573b42fb9acce7c34fa83a`,
독립 한컴 Print `pdf/issue5941-1480000-201900698-2020.pdf`는
`fe01f1ddffc3153d015adab93178f0fa8a5e9624763ff529362d4410d50955dc`다.
Print·compat2022·96DPI·동일 공급 폴더와 실제 Library/Fonts 조건이다.

8구역 문단174 표44의 셀33 Wasp kinin과 셀39 Mastoparan, 셀57 Acid phosphatase는
문자 음영이 아니라 원본 RangeTag type2의 형광펜이다. 전자는#FFB689, 후자는#FFE766이며
원본 MarkpenMark와 영역 범위를 유지한다. 독립 PDF에는 이 마킹 색이 없다.

한컴 [인쇄 확장 도움말](https://help.hancom.com/hoffice/multi/ko_kr/hwp/file/print/print_expanded.htm)은
형광펜을 별도 인쇄 선택 사항으로 구분한다. [한컴 개발자 답변](https://forum.developer.hancom.com/t/topic/3010)도
기본 인쇄에서 형광펜을 제외함을 설명한다. 이 원본의 독립 Print와 일치하는 기본 출력 계약을 적용한다.
형광펜을 포함하도록 선택하는 새 인쇄 옵션/API 지원을 주장하지 않는다.

## 구현과 실제 소비

`layout/paragraph_layout.rs`가 RangeTag type2로 만든 사각형에 기존 `editor_only` 속성을 부여한다.
그림·셀 채움·글자 shade_color/BorderFill에는 이 속성을 붙이지 않는다.
원본 정보나 화면 표시를 삭제하지 않으며, 텍스트·줄 구성·셀 높이는 변경하지 않는다.

`원본 RangeTag → 실제 글줄 위치의 Rectangle → 기존 LayerBuilder profile 필터 →
Native Skia/WebCanvas/CanvasKit의 같은 확정 PaintOp` 경로로 연결된다.
Screen/FastPreview에는 표시하고 Print/HighQuality에서는 편집 표시를 제외한다.
backend별 색상/문서번호 분기, PDF 픽셀 복사, 비교 허용치·마스크 변경은 없다.

## 전쪽 비교와 직접 판독

| 범위 | Native raw | fresh WASM raw |
| --- | ---: | ---: |
| 수정 전146쪽 |97.00104%|96.99896%|
| 수정 후146쪽 |100.00000%|100.00000%|
| 수정 후205쪽 최저 |97.13499%|97.13158%|

각205/205쪽·90% 미달0쪽·누락0쪽·쪽수 차이0쪽이다. 직전C82와의205쪽 PNG hash를
대조하면 양쪽 모두144/145/146/147/148쪽만 바뀐다.0.01%p 이상 하락한 쪽은 없다.
최저는 다른137쪽이며 이 보정으로 문서 전쪽98% 이상을 달성했다고 주장하지 않는다.

146쪽 전후 Native/WASM review·overlay를 직접 열어 본문·표 경계·단어 위치를 확인했다.
100%는2px 관용 raw 실루엣 지표이며, 엄격 내용 픽셀은 Native18.76%/WASM18.81%다.
글자 주위 AA·미세 raster 차이가 남으므로 엄격 픽셀 동일이라고 쓰지 않는다.

실제 Native Screen146쪽의 원본 주황색502픽셀·노란색5023픽셀이 유지되고,
같은 쪽 Print에서는 두 색 모두0픽셀이다. 직접 CLI의 첫 화면 대조는0부터 시작하는
페이지 인자를 잘못 주어147쪽이었다. 그 원본/로그를 별도 보존하고145 인자로146쪽을
다시 실행했다. 첫147쪽 이미지는146쪽 쌍대 증거에서 제외했다.

## 정식 회귀와 출처

정식 native-skia 회귀는 수정 전8건 중7통과·1실패(1.129초), 수정 후8건 모두 통과(1.410초)했다.
새 검사는 원본 셀·내용 순서·RangeTag 보존, Screen/Print/HQ의 실제 PaintOp와
Skia PNG의 마킹 유무 및 독립 Print의 글자 시작 ±2CSSpx를 함께 검사한다.

정상 대조군은 #6956 형광펜 저장 왕복3건, #6788 혼합 글자모양17건,
#536 글자 효과1건으로 총21건 모두 통과했다. 이 결과는 리베이스 전 부모4dc0761369의
보정 후보에서 실행한 증거이며, 이후 리베이스된 head의 통과로 대신 인용하지 않는다.

Native binary SHA-256은 `7166ac8bf5c3ff8cbada0f66a1966bda6386e70819d2f759ebe380b6fe457a2e`, fresh WASM은 `6f7dabdf684000ff7dae2ecfe1a59764aad3163b5d13684ab90df7352b923867`다.
부모4dc0761369와 당시 후보 source inventory1150파일을 보존한다.
Mac WASM은 locked wrapper web/no-opt 로컬 대체 빌드이며 root pkg/Studio public 해시가 같다.
Docker 최적화 통과나 실제 Studio 소비 검증으로 확대하지 않는다.

전체 TSV·producer·명령·로그는 ignored
`output/pr-review/7445-completed-20261009/regression-policy-audit-20261015/rebase-552514f4-20261016/markpen-print-c84/`
아래에 보존한다. 기준 devel 전쪽 비교·남은raw 미달(sample16/로드맵153)·가는 선 보정의
다른 페이지 하락·실제 Studio/CanvasKit·최종 전체 회귀/lint는 PR 전체의 잔여 항목이다.

| 출력 | review | overlay |
| --- | --- | --- |
| 수정 전 Native |[review](before-native-review.png)|[overlay](before-native-overlay.png)|
| 수정 후 Native |[review](after-native-review.png)|[overlay](after-native-overlay.png)|
| 수정 전 WASM |[review](before-wasm-review.png)|[overlay](before-wasm-overlay.png)|
| 수정 후 WASM |[review](after-wasm-review.png)|[overlay](after-wasm-overlay.png)|

[화면 형광펜 보존 대조](screen-native-page146.png)
