# #8153: 가시 기호의 실제 glyph 공급 확인

## 실제 원본과 독립 출력

200쪽의 세 글머리표는 원본 U+26AB `⚫`다. `68dbcdc78f`의 실제 Native PNG에서
세 기호가 보이지 않으며 각각 미대응 37픽셀이 있다. 문서 전체 점수로 이 내용 누락을
면제하지 않는다. 현재 원본의 style은 `KoPub돋움체 Bold,한컴바탕`이며 IR의 문자와
family를 다른 것으로 바꾸어 숨기지 않는다.

독립 2024 Print의 실제 glyph는 HCR Batang이다. 공급된 `HANBatang.ttf`의
U+26AB는 advance 970/1000em, 윤곽 명령 6개이며 독립 PDF의 해당 glyph와
명령 지문의 SHA-256이 정확히 같다. 반면 공급된 `HBATANG.TTF`에는 cmap의
비영 gid 60425와 advance 1024/1024em이 있어도 해당 glyph의 윤곽 명령은 0개다.
두 프로그램의 이름과 매핑은 서로 다른 원본 자료이며 alias를 바꾸어 동일시하지 않는다.

ignored `remaining-raw100-diagnostic/p200-symbol-font-outline-proof.json`에 실제 파일·
PDF SHA와 메트릭·명령 수·지문만 보존했다. 프로그램이나 윤곽 데이터를 Git에 추가하지
않았으며 설치 파일도 변경하지 않았다. 실제 Native 선택 face는 renderer snapshot의
`get_font_decision_trace`로 추가 확인한다. 빈 윤곽 후보의 존재만으로 선택 원인을 확정하지 않는다.

## 사양과 공통 원인 계층

OpenType의 [cmap](https://learn.microsoft.com/en-us/typography/opentype/spec/cmap)은
문자에서 glyph ID로 가는 매핑이고, [glyf](https://learn.microsoft.com/en-us/typography/opentype/spec/glyf)는
실제 TrueType 윤곽을 별도로 담는다. 매핑과 advance만으로 잉크 공급을 입증하지 않는다.
공백·서식 제어·결합 문자에는 정상적인 빈 glyph가 있으므로 일괄 거절하지 않는다.
색·bitmap·SVG glyph, CFF와 가변 글꼴의 실제 출력 자료도 별도로 확인한다.
이는 HWP 사양 오류의 확인이나 특정 글꼴 자체가 손상됐다는 판정이 아니다.

현재 Native 선택 함수는 비영 gid만 확인한다. SVG의 실제 문자 포함 검사와 browser
FontFace 공급도 같은 의미를 소비하는지 추적한다. 측정이 선택한 실제 프로그램과
paint가 그리는 프로그램을 함께 확인하고, 같은 공급 결과를 Native/fresh WASM이
소비하게 한다. 기호 하나·폰트 이름·문서 ID를 분기 조건으로 추가하지 않는다.

## 검증 순서

1. 동일 producer·같은 공급 경로의 Native observer에서 실제 요청·후보·선택·출처를
   확인한다. 원본 언어 슬롯과 실제 CSS/Typeface를 대조한다. 프로그램 이름을
   실제 선택의 증거로 추정하지 않는다.
2. 빈 윤곽 때문에 후속 실제 기호 공급이 막힌 것으로 확인되면 해당 공통 공급 계약을
   보정한다. 저장 글꼴명·cmap 원본 파일·사양 수치와 레이아웃을 임의로 변경하지 않는다.
   browser의 cmap 선택과 Native의 glyph 선택이 다른 판단으로 남지 않게 한다.
3. 저장소의 MIT 테스트 글꼴을 메모리에서만 조합해 빈 기호 후보·정상 fallback·공백·
   실제 glyph·색/bitmap 대조를 검증한다. 기존 원 PUA 공급·미공급 계약도 유지한다.
4. 200쪽의 세 기호·뒤 본문 시작·순서와 앞뒤 정상 쪽을 실제 재출력한다. Native/fresh
   WASM에서 양쪽 원본의 전쪽 비교·직접 판독·새 #8153 회귀와 최종 필수 게이트를 완료한다.

현재는 원인 확인 계획이다. 신규 글꼴 정책 구현·100% 또는 필수 전체 테스트의 완료를
뜻하지 않는다. 브랜치는 `task/8153-handbook-raw100`이며, 동기화·원격 PR 처리는 하지 않는다.
