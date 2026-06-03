# Task M100-852 Stage 2.5 구현 계획서 — JavaScript 일관 직렬화

- 이슈: [#852](https://github.com/edwardkim/rhwp/issues/852)
- 브랜치: `local/task852`
- 기반 보고서: `mydocs/working/task_m100_852_stage24.md` (5/5 Form byte-perfect + 한컴 5/5 성공)
- 작업지시자 관찰 (2026-05-20): "form-01.hwp 에 자바스크립트가 포함되지 않은 것으로 보임"

## 1. 진단

### 1.1 정답지 분석

`samples/form-01.hwp` 의 Scripts/DefaultJScript (raw deflate, 1580 bytes → 789 chars):

```javascript
var Documents = XHwpDocuments;
var Document = Documents.Active_XHwpDocument;
var PushButton = Document.XHwpFormPushButtons.ItemFromName("PushButton");
var CheckBox = Document.XHwpFormCheckButtons.ItemFromName("CheckBox");
var ComboBox = Document.XHwpFormComboBoxs.ItemFromName("ComboBox");
var RadioButton = Document.XHwpFormRadioButtons.ItemFromName("RadioButton");
var Edit = Document.XHwpFormEdits.ItemFromName("Edit");

function OnDocument_Open() { /*todo*/ }
function OnDocument_New() { /*todo*/ }
function OnComboBox_Click() { /*todo*/ }

{
ComboBox.Enabled = 1;
ComboBox.ResetContent();
ComboBox.Text = "계절 선택"
ComboBox.InsertString("봄", 0);
ComboBox.InsertString("여름", 1);
ComboBox.InsertString("가을", 2);
ComboBox.InsertString("겨울", 3);
}
```

### 1.2 HWPX 컨테이너 두 파일로 분리

| HWPX 파일 | 크기 | 내용 |
|----------|------|------|
| `Scripts/headerScripts` | 860 B | UTF-16 LE 변수 선언 (var Documents... var Edit...) |
| `Scripts/sourceScripts` | 700 B | UTF-16 LE 함수/핸들러 (OnDocument_Open / OnComboBox_Click / ComboBox 초기화) |

### 1.3 Stage 2.1 의 한계

Stage 2.1 의 [src/parser/hwpx/contract_streams.rs:84-89](src/parser/hwpx/contract_streams.rs:84-89) :

```rust
let scripts_default_jscript = match reader.read_file_bytes("Scripts/sourceScripts") {
    Ok(bytes) => zlib_deflate(&bytes).unwrap_or_else(|| FALLBACK_SCRIPTS_DEFAULT_JSCRIPT.to_vec()),
    Err(_) => FALLBACK_SCRIPTS_DEFAULT_JSCRIPT.to_vec(),
};
```

- `Scripts/sourceScripts` 만 사용 → `headerScripts` 누락 → var 선언 부재 → JS 동작 불능
- `zlib_deflate` 사용 (zlib 헤더 포함) → 정답지는 **raw deflate** (zlib 헤더 없음)
- Length-prefix 4 bytes (u32 LE) 미포함 — 정답지의 첫 4 bytes 가 `ae 01 00 00` (= 0x1ae = 430). 텍스트 길이 또는 다른 메타

### 1.4 BodyText 의 %clk + 0x57 누락

`%clk` (CTRL_HEADER tag=0x47, size=151) 와 자식 0x57 (size=26 "myMsg01") 는 PushButton 의 onclick 핸들러 메시지 ID 를 BodyText 에 reference. Stage 2.4 에서 미구현.

#### %clk CTRL_HEADER 구조 (reverse engineering 확정, 151 bytes)

```
0..4    ctrl_id "%clk"
4..8    attr = 0x00000001
8       u8 = 0x09 (flag)
9..11   u16 LE wchar_count (예: 66)
11..143 UTF-16 LE 속성 문자열 (66 wchars):
        "Clickhere:set:48:Direction:wstring:6:여기에 입력 HelpState:wstring:0:  "
143..147 instance_id (u32, 0x7dcd59db — form 0xd6..0xda 다음)
147..151 zero (4 bytes)
```

#### 0x57 자식 레코드 구조 (26 bytes)

```
0..2    0x021b (?)
2..6    0x00000001 (?)
6..8    0x4001 (flag)
8..10   0x0001 (?)
10..12  u16 LE wchar_count = 7
12..26  UTF-16 LE 'myMsg01' (7 wchars)
```

## 2. 구현 전략

### 2.1 Scripts/DefaultJScript = headerScripts + sourceScripts 결합 + raw deflate

[src/parser/hwpx/contract_streams.rs](src/parser/hwpx/contract_streams.rs) 의 zlib_deflate 사용 → **raw deflate** + length-prefix 4 bytes 추가:

```rust
// HWPX 의 headerScripts (var 선언) + sourceScripts (함수) 결합
let header_scripts = reader.read_file_bytes("Scripts/headerScripts").unwrap_or_default();
let source_scripts = reader.read_file_bytes("Scripts/sourceScripts").unwrap_or_default();

if !header_scripts.is_empty() || !source_scripts.is_empty() {
    // 4 bytes length prefix (u32 LE) + headerScripts + sourceScripts
    let len_prefix = ((header_scripts.len() / 2) as u32).to_le_bytes(); // wchar count
    let mut combined = Vec::with_capacity(4 + header_scripts.len() + source_scripts.len());
    combined.extend_from_slice(&len_prefix);
    combined.extend_from_slice(&header_scripts);
    combined.extend_from_slice(&source_scripts);
    let deflated = raw_deflate(&combined).unwrap_or_else(|| FALLBACK_SCRIPTS_DEFAULT_JSCRIPT.to_vec());
    streams.push(("/Scripts/DefaultJScript".to_string(), deflated));
} else {
    streams.push(("/Scripts/DefaultJScript".to_string(), FALLBACK_SCRIPTS_DEFAULT_JSCRIPT.to_vec()));
}
```

(length prefix 의 정확한 의미 — 4 bytes = 0x1ae = 430, 헤더 바이트 카운트? Stage 2.5.2 에서 byte-level diff 확인 후 정확화)

### 2.2 raw deflate 헬퍼

`flate2::write::DeflateEncoder` 사용 (zlib 헤더 미포함):

```rust
fn raw_deflate(input: &[u8]) -> Option<Vec<u8>> {
    use flate2::write::DeflateEncoder;
    use flate2::Compression;
    use std::io::Write;
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(input).ok()?;
    encoder.finish().ok()
}
```

### 2.3 BodyText 의 %clk + 0x57 직렬화

작업지시자 관찰의 본질은 "JS 미포함" — 즉 Scripts/DefaultJScript 가 핵심. `%clk` 는 PushButton 의 onclick 메시지 ID reference (보조). Stage 2.5 에서 정공법 구현:

- HWPX form-01.hwpx 의 `command` 속성 또는 `<hp:script>` 자식 확인
- 발견 시 `Control::Form` 직렬화 직후 `%clk` 컨트롤 + 0x57 자식 추가
- 발견 안 되면 본 단계 생략 (정답지의 PushButton 만 %clk 보유 가능성)

검증 시 한컴이 %clk 미존재로 손상 판정한다면 Stage 2.5.4 에서 보강.

## 3. Stage 분해

### Stage 2.5.1 — Scripts/DefaultJScript 정정 (예상 30분)

1. [src/parser/hwpx/contract_streams.rs](src/parser/hwpx/contract_streams.rs) 의 zlib_deflate → raw_deflate
2. headerScripts + sourceScripts 결합 + 4-byte length prefix
3. byte-level diff (정답지 1580 bytes raw deflate vs 변환 결과)

### Stage 2.5.2 — %clk + 0x57 BodyText 직렬화 (예상 1시간)

1. HWPX form-01.hwpx 의 `command` / `<hp:script>` 자식 확인 (script handler 존재 여부)
2. FormObject 에 script handler 정보 보존 또는 별도 모델 (CommandHandler?)
3. PushButton (또는 모든 Form) 직렬화 직후 `%clk` 추가
4. byte-level diff

### Stage 2.5.3 — 전체 fixture 재검증 + CI 패턴

1. form-01/02/002.hwpx 변환 → CFB 스트림 보존 + Form byte-perfect 유지
2. hy-001 / tbox-v-flow-01 회귀 없음
3. CI 패턴 통과 (cargo test --release --tests + clippy + fmt)

### Stage 2.5.4 — 한컴 재판정 (작업지시자 게이트)

1. form-01.hwp 변환 결과 → 한컴 에디터에서 **JavaScript 포함 + 정상 동작 확인**
2. 기존 5/5 손상 없음 유지

## 4. 회귀 위험

| 영역 | 위험도 | 근거 |
|------|--------|------|
| HWPX→HWP 변환 (Scripts) | **의도된 변경** | Stage 2.1 의 단순 zlib → 정답지 호환 raw deflate + 결합 |
| HWPX 컨테이너 비-script 케이스 | **낮음** | headerScripts/sourceScripts 부재 시 fallback 유지 |
| Form 직렬화 (Stage 2.4) | **영향 없음** | %clk 는 Form CTRL_HEADER 와 별개 컨트롤 |
| HWP→HWP roundtrip | **영향 없음** | parser 의 Scripts/DefaultJScript 보존 무변경 |

## 5. 검증 기준

- 정답지 Scripts/DefaultJScript 1580 bytes 와 변환 결과 ±10% 이내
- 정답지 JS 코드 (var 선언 + 함수 + ComboBox 초기화) 모두 보존
- 한컴 에디터에서 form-01.hwp 의 PushButton/ComboBox 가 JS 핸들러 보유 (작업지시자 시각 판정)
- 기존 5/5 한컴 손상 없음 유지

## 6. 메모리 룰 정합

- ✅ `feedback_self_verification_not_hancom` — 작업지시자 한컴 게이트 필수
- ✅ `feedback_diagnosis_layer_attribution` — Scripts/DefaultJScript 의 단일 sourceScripts vs 결합 차이 정확 식별
- ✅ `feedback_hancom_compat_specific_over_general` — 정답지 raw deflate + length prefix 정공법
- ✅ `feedback_push_full_test_required` — CI 패턴 통과
