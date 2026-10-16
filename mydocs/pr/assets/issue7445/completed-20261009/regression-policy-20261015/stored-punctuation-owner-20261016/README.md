# 저장 줄 구두점의 측정·paint 글꼴 소유 (2026-10-16)

Issue: #7445. 기준 `0953598d8e08af5e9149b3e4cdbbbf2f5f2bae64`,
부모 `af8c17eeee96683e7a986f7094e2fdab4e331068`의 C74 보정이다.
이 기록은 개인정보 문서의 구두점 보정이며 별도 HWP3 제목의 누적 자간 결함을 해결했다고 쓰지 않는다.

## 독립 근거·공통 소비

`samples/task2097/75544_pii_bunseok.hwpx`의10쪽 문단95에서 원본 두 번째 scalar인
하이픈은 독립 한컴2020 Print의 PalatinoLinotype-Roman15pt다. 기존 폭 결정은 Latin 슬롯을
선택하지만 저장 줄의 compose guard가 그 분할을 막아 실제 paint는 휴먼명조/Haansoft 체인을 썼다.
`char_lang_slot`/`char_width_decision`의 ASCII 구두점 슬롯을
`compose_paragraph_in_context`도 소비하도록 통일했다. 일반 공백은 원래 언어를 유지하고,
글자겹침 개체는 분할하지 않으며 각 조각의 표시 문자열도 원문 scalar 범위로 나눈다.
`ComposedTextRun` → `resolved_to_text_style` → 공통 `TextRunNode` → Native/SVG/Canvas paint로 전달한다.
문서 ID·글꼴 이름·고정 좌표로 실행 조건을 좁히지 않았다.

## 전쪽 확인과 px 선정

새 Native/fresh WASM 모두 쪽수66/66·전66쪽 최저93.77937%·90% 미만0쪽이다.
두 backend 모두10/29/53쪽100%이므로 동률 중 첫10쪽을 중요한 하이픈 경계로 골랐다.
독립 PDF 하이픈 원점(64.0800,106.3200)pt를96CSSpx로 환산한
X85.4400/기준선141.7600을 ±2CSSpx로 확인한다. 캐논 캡처는192DPI다.
원문 문단·글자모양·charStart1의 단일 소유, 원문 누락/중복/순서 및 한국어 본문 글꼴도 함께 검사한다.
원본 자간을 바꾸지 않는다.

|10쪽|보정 전 이력|보정 후|
|---|---:|---:|
|Native|99.50117%|100.00000%|
|fresh WASM|99.50302%|100.00000%|

전후 PNG의 before는 리베이스 전 C73 `25279395172d9e659b3feab5305108e1e3818bd3` 이력이며
새 base의 검증 결과로 재인용하지 않는다. C73→현재 부모 리베이스에서 renderer 소스는 동일했고,
Rust 차이는 PDF capability API 추가다. after는 새 base 위의 frozen C74를 새로 빌드·출력했다.
양쪽 after review/overlay를 직접 열어 하이픈·뒤 본문·정렬과 쪽번호를 판독했다.
100%는2px 관용 실루엣 값이며 엄격 픽셀 일치를 뜻하지 않는다.

## 정식 회귀·실제 소비

`stored_hyphen_uses_the_measured_latin_face_and_preserves_source_membership`는 정확한
부모 구현에서 실제1FAIL(하이픈의 단일 paint 소유0 vs1,0.164초) → 보정1PASS(0.153초)다.
첫 시도의 테스트 타입 오류는 컴파일 실패로 별도 보존하고 FAIL 증거에서 제외했다.
수정한 동일 테스트로 전후를 다시 실행한 `c74-formal-corrected/before.log`·`after.log`를 정본으로 쓴다.
동일 C74 생산 소스의 기존 composer 대조군91건도91PASS/0FAIL(0.09초)이며,
필드/표시 문자열·글자겹침·문단 프레임 대조를 포함한다. 재실행 중인 같은 대조군과 전체 회귀의
완료를 앞당겨 주장하지 않는다. 실행 정본은 `c74-formal/composer-counters.log`다.

실제 Studio 제품 파일 열기/Canvas2D의 layer에서도 문단95/char1이 Palatino Linotype,
X85.56/기준선141.667CSSpx임을 확인했다. 처음 로드한 JS/WASM 응답과 글꼴 선택 TS 응답을
frozen package/작업장 source와 대조했다. 이는 layer 소비 확인이며 실제 스크린샷·CanvasKit
전체 수용을 대신하지 않는다. 첫 진단의 기본 host CDP 접속 실패는 제외하고 Mac headless로 재실행했다.
새 PDF capability API도 실제 fresh package에서 존재하며 기본 build에서는 false다.

추가5문서315쪽은 Native/fresh WASM 모두 쪽수 일치·전쪽90% 이상이다.
고정 이전 cdef 대비0.01%p 감소와 최신0953 기준48조건 전쪽 비교는 별도 추적한다.
개인정보50쪽 및 다른 문서 감소·새 전체 회귀/actual 검증 완료를 주장하지 않는다.
지침·사용자 초안·원본·PDF·공유 target은 보존했다.

Native SHA256 `babff85404ce9d9377eab28ae9f806ea3c030e07bb2f8b6e59836c8fdb56df18`.
WASM SHA256 `48957313dba63e8175e3f92b07e8b31be682b58baeb17aec10a00c5f8e2aec67`.
기준 PDF SHA256 `0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770`.
TSV·생산/글꼴/입력 hash·명령·실제 로그는 ignored output의
`regression-policy-audit-20261015/rebase-0953598d-20261016/c74-*`에 보존한다.
Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.

## 비교 이미지

|출력|수정 전 이력|수정 후|
|---|---|---|
|Native review|[review](before-native-review.png)|[review](after-native-review.png)|
|Native overlay|[overlay](before-native-overlay.png)|[overlay](after-native-overlay.png)|
|WASM review|[review](before-wasm-review.png)|[review](after-wasm-review.png)|
|WASM overlay|[overlay](before-wasm-overlay.png)|[overlay](after-wasm-overlay.png)|
