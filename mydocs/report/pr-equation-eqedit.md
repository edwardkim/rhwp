# PR: 수식 EQEDIT attribute 파싱/직렬화 양쪽 소실 수정 (Issue #2727)

## 배경

HWP5 EQEDIT record의 첫 필드(UINT32 attr)는 `lineMode` (bit 0: 0=글자단위/CHAR, 1=줄단위/LINE)를
포함하지만, 파싱 시 `_attr`로 읽기만 하고 모델에 저장하지 않아 버려졌다. 직렬화 시에도
`write_u32(0)`으로 고정 출력되어 저장 시 항상 0이 기록되었다.

HWPX 경로에서도 `<hp:equation>`의 `lineMode` 속성이 파싱/직렬화 양쪽에서 누락되어 동일한 유실이
발생했다.

**4경로 전부 유실**: HWP5 parse, HWP5 serialize, HWPX parse, HWPX serialize

## 변경 사항

### 1. 모델: `eqedit` 필드 추가

**파일**: `src/model/control.rs`

`Equation` struct에 `pub eqedit: u32` 필드 추가 (bit 0 = lineMode)

### 2. HWP5 파서: EQEDIT attr 저장

**파일**: `src/parser/control.rs`

기존 `let _attr = r.read_u32().unwrap_or(0);` → `equation.eqedit = r.read_u32().unwrap_or(0);`

### 3. HWP5 직렬화: EQEDIT attr 출력

**파일**: `src/serializer/control.rs`

기존 `w.write_u32(0).unwrap()` → `w.write_u32(eq.eqedit).unwrap()`

### 4. HWPX 파서: `lineMode` 속성 파싱

**파일**: `src/parser/hwpx/section.rs`

`<hp:equation lineMode="LINE">` → `eqedit |= 0x01`
`<hp:equation lineMode="CHAR">` → `eqedit = 0x00`

### 5. HWPX 직렬화: `lineMode` 속성 출력

**파일**: `src/serializer/hwpx/section.rs`

EQEDIT attr bit 0에 따라 `lineMode="LINE"` 또는 `lineMode="CHAR"` 속성을 `<hp:equation>`에 추가

### 6. 테스트 픽스처 갱신

**파일**: `src/serializer/hwpx/mod.rs`

`equation_control_roundtrip_preserves_script` 테스트의 Equation 생성자에 `eqedit: 0` 추가

## 영향

- `Default` derive로 `eqedit: 0`이 기본값이므로 신규 생성 수식은 CHAR mode (글자 단위)
- 기존 HWP5 파일의 EQEDIT attr이 라운드트립에서 보존됨
- HWPX `<hp:equation>`에 `lineMode` 속성이 항상 출력됨 (기존에는 없었음)
