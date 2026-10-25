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

## 2026-10-26 실제 관측과 보정 범위

동일 producer `6e0114e077dd2354266dbe18e7935341a6fbd7a4`의 Native observer에서
200쪽 세 기호 모두 `paint.native.resolved=Haansoft Batang`, `certainty=observed`를
확인했다. 측정은 기존 KoPub advance 872HU에 원본 장평·자간을 적용하고 있다.
이번 공급 보정은 그 저장/조판 좌표를 변경하지 않고 빈 paint 기호만 후속 실제 프로그램으로 넘긴다.
보정 후 실제 글꼴·윤곽·뒤 본문과 위치를 다시 대조한다. 측정과 paint의 동일성 전체를
확인했다고 주장하지 않는다.

브라우저는 원본 HBATANG의 기존 cmap 종료 형식 때문에 원본 바이트를 직접 거부했다.
이 실패를 fallback 결과로 해석하지 않았다. 표준 Sweep에서 이미 사용하는 메타데이터
정규화 프로그램(원본 SHA `35f84328…`, 공급 SHA `e391734b…`)으로 재현한 결과,
첫 face와 HCR Batang을 함께 등록해도 U+26AB의 잉크는 0픽셀이었다. FontFace의
unicodeRange에서 그 코드포인트만 제외한 진단 대조에서는 잉크 36픽셀, 정상 A는
양쪽 모두 39픽셀이었다. 이는 원인 실험이며 실제 새 WASM 출력 검증을 대체하지 않는다.

공통 `font_symbol_coverage`는 Unicode 기호 범주 중 원래 빈 칸인 U+2800을 제외하고,
TrueType loca/glyf가 명시적으로 빈 glyph라고 선언한 경우에만 공급에서 제외한다.
공백·문자·결합/서식 문자와 미해석·깨진 glyph는 빈 것으로 추정하지 않는다.
색·bitmap·SVG 자료가 있는 glyph와 CFF·가변 face도 이 빈 glyf 정책으로 거절하지 않는다.
기호 집합은 Unicode scalar의 유한한 범위를 한 번 순회해 만들고, 외부 cmap가 주장하는
무제한 범위를 순회하지 않는다. 같은 제외 집합을 Native selector와 WASM FontFace
unicodeRange가 소비한다. 원본 face 이름·cmap·glyf·advance는 바꾸지 않는다.
Studio의 승인된 로컬 프로그램·호스트 FontFace 공급에도 같은 Rust API를 사용한다.

생산 경로는 `공급 bytes → 공통 빈 기호 집합 → Native 후보 선택/FontFace unicodeRange →
기존 positioned paint`다. 문서의 기호나 family, 저장 좌표, PNG 점수 판정은 수정하지 않는다.
표준화되지 않은 WOFF/WOFF2·URL 전용 공급의 빈 glyph를 추정하지 않는다.

정식 검사는 저장소의 MIT 합성 글꼴만 메모리에서 조합한다. 기존 E 라이브러리로 실행한
`mapped_empty_symbols_use_real_fallback_without_changing_source_or_positions`는
첫 빈 기호가 실제 fallback의 PNG를 가리는 결함으로 FAIL했다. 해당 모듈 원문·SHA와
빌드/실행 명령을 ignored `symbol-fallback-before-*`에 보존했다. 원본 private 글꼴은
새 fixture나 Git 파일로 복사하지 않는다. 보정 후 합성 계약·실물 200쪽과 정상 대조군,
Native/fresh WASM 전체 비교 결과를 이어서 기록한다.


### 내장 bitmap의 빈 잉크 추가 확인

실제 HBATANG에는 U+26AB를 포함한 EBLC/EBDT 12·14·16ppem strike가 있다.
그러나 해당 glyph의 bit-aligned 1-bit 데이터 18·25·32바이트는 모두 0이다.
원본 SHA와 각 strike의 metric·길이·0이 아닌 바이트 수·데이터 SHA만 ignored
`p200-empty-symbol-bitmap-proof.json`에 보존했다. 이는 실제 표시 가능한 bitmap의
부재를 뜻하며, bitmap 테이블의 존재 자체를 정상 잉크의 증거로 삼으면 안 된다.

[공식 EBDT 사양](https://learn.microsoft.com/en-us/typography/opentype/spec/ebdt)의
1-bit 0=white 계약과 [EBLC 사양](https://learn.microsoft.com/en-us/typography/opentype/spec/eblc)의
모든 strike 범위를 확인한다. 최대 ppem 하나만 확인해서 다른 크기의 정상 glyph를
버리지 않는다. PNG·색 표현·미해석/손상·파싱 예산 초과는 빈 것으로 추정하지 않고 보존한다.
실제 잉크가 있는 중간 strike와 전부 빈 strike를 MIT 메모리 fixture로 대조할 계획이다.
현재 Native 빌드 소스를 동결한 상태이므로 이 추가 보정은 해당 빌드 종료 후 적용한다.


## 보정 구현과 실제 실행 결과 (2026-10-26)

- `aab94696861685a4f952150668cdcba61bf0216e`: 공통 빈 glyf 판정과 Native/FontFace
  소비 경로를 연결했다. MIT 실제 Native paint 회귀는 수정 전 E에서 FAIL, 이 후보에서
  PASS했다. 기존 핵심 사례와 합쳐 41건 통과했다. 실제 양쪽 51쪽씩, 102개 조건은
  전부 출력 불변이었다. 실제 원본의 EBDT strike가 모두 빈 사실까지 확인한 뒤
  이 중간 후보의 불필요한 전쪽 실행은 중단했다. 전쪽·WASM 통과로 기록하지 않는다.
- `5bcb654516968c64d5c3910aeb556fe5e3eebe72`: 공통 판정이 모든 해당 EBDT/CBLC/bloc
  strike의 잉크도 확인한다. 1-bit/회색조 원자료가 모든 크기에서 0인 경우에만 빈 것으로
  인정한다. 크기 하나라도 실제 잉크가 있으면 보존하며, PNG·색·sbix와 해석 실패·예산
  초과도 보존한다. EBLC 크기 수 검사 예산 256은 사양의 최대치라고 주장하지 않는다.
- 새 정식 `entirely_blank_ebdt_does_not_mask_fallback_but_any_ink_strike_is_preserved`는
  앞 후보에서 2 PASS / 1 FAIL로 결함을 재현했고 새 후보에서 PASS했다. 최대 크기는
  비었지만 중간 크기에 잉크가 있는 독립 합성 자료도 정상 보존한다. 이 합성 자료는
  기존 MIT glyph에 메모리에서 명시적인 EBLC/EBDT 구조를 더한 것이며 한컴 일치의 증거와 구별한다.
- 새 Native 후보는 고정 `target/pr-review`, jobs 8로 release-test 빌드했고 exit 0이다.
  frozen code SHA를 쓰는 집중 회귀는 **42 PASS / 0 FAIL**이다. 전체 nextest 완료를 뜻하지 않는다.
- 실제 HWPX 200쪽은 미대응 **125→14개**, raw **99.84905→99.98309%**다.
  누락됐던 세 원 U+26AB가 표시됐고 각 37픽셀이 해소됐다. 같은 96 DPI의 독립 Print,
  E의 수정 전 Native, 새 Native를 확대해 직접 판독했다. 같은 51쪽 대조에서 나머지 쪽은
  미대응 증가가 없었다. 남은 14픽셀은 본문 공백/글자 간격 차이이며 이 페이지의 100%
  달성으로 보고하지 않는다. HWP·양쪽 전쪽 Native·optimized fresh WASM은 이어서 검증 중이다.
- 실제 Chrome을 쓰는 기존 공급 검사 2건, Sweep 환경 검사 12건, Studio host font 검사
  4건과 Studio TypeScript 검사는 통과했다. 새 최적화 WASM의 실제 공통 API 및 문서
  출력은 해당 producer의 빌드/PNG/TSV로 별도 확인한다. 과거 브라우저 단독 실험을
  새 WASM 검증으로 승격하지 않는다.

실행 로그·명령·beforeFAIL 지문과 확대 PNG는 ignored `output/pr-review/
issue8153-handbook-raw100-20261022/`에 있다. 코어와 기존 설치 글꼴의 source 프로그램은
변경하지 않았으며 private font binary/윤곽을 Git에 추가하지 않았다. 외부 글꼴의
저작권과 원문 family 선언을 이 테스트 목적의 별칭으로 덮어쓰지 않았다.
전체 목표와 필수 전체 게이트는 계속 진행하며, upstream 동기화·원격 PR 조치는 하지 않는다.
