# Task #514 Stage 1 완료 보고서 — 본질 진단

## 진단 결과 요약

**근본 원인 확정**: `samples/복학원서.hwp` 의 s0:pi=0 학교 로고 이미지 (`BIN0001.PCX`) 가 **PCX 포맷** 으로 저장되어 있고, rhwp 의 `detect_image_mime_type` 가 PCX 를 인식하지 못해 `application/octet-stream` 으로 emit → 브라우저가 이미지로 렌더링 불가.

**가설 점검 결과**: 수행 계획서의 가설 A (빈 문단 가드) / B (다른 controls 차단) / C (BehindText 처리 표 한정) / D (좌표 산출 결함) 모두 **본질 아님**. SVG 의 `<image>` 태그는 정상 emit 되어 있고, 좌표/크기도 정확하나 **이미지 데이터 자체가 비표준 포맷**.

## 진단 과정

### 1. SVG 출력 점검 (가설 A/B/C 검증)

```bash
rhwp export-svg samples/복학원서.hwp -o output/svg/task514/
```

생성 SVG 의 `<image>` 태그 인벤토리:

```
<image x="137.71" y="270.24"  width="495.04" height="495.73" ... 워터마크 직인
<image x="65.49"  y="49.01"   width="77.01"  height="87.89"  ... 학교 로고 (s0:pi=0)
```

학교 로고는 SVG 에 정상으로 emit 되어 있다 → **paint 단계 누락 아님** → 가설 A/B/C 모두 기각.

### 2. 좌표 검증 (가설 D 검증)

학교 로고의 SVG 좌표:
- x = 65.49 px ≈ 17.3 mm = 페이지 좌여백 15mm + 단 오프셋 2.3mm ✅
- y = 49.01 px ≈ 13.0 mm = 페이지 상여백 10mm + 문단 오프셋 3.0mm ✅
- width = 77.01 px ≈ 20.4 mm ✅ (IR 와 일치)
- height = 87.89 px ≈ 23.3 mm ✅ (IR 와 일치)

좌표 / 크기 모두 정확 → **좌표 산출 결함 아님** → 가설 D 기각.

### 3. 이미지 데이터 점검 (신규 가설 발견)

학교 로고의 `<image>` 태그 `href` 속성:

```
href="data:application/octet-stream;base64,CgUBAQAAAAB..."
```

**관찰**: 정상 이미지라면 `data:image/png` 또는 `data:image/jpeg` 가 와야 하는데 `application/octet-stream` (일반 바이너리) 으로 emit 됨 → 브라우저가 이미지 디코딩 불가.

base64 디코딩 시 첫 바이트:

```
0a 05 01 01 00 00 00 00 6d 03 e8 03 2c 01 2c 01
```

표준 magic number 점검:
- PNG: `89 50 4e 47` ❌
- JPEG: `ff d8 ff` ❌
- GIF: `47 49 46` ❌
- BMP: `42 4d` ❌
- WMF: `d7 cd c6 9a` 또는 `01 00 09 00` ❌
- TIFF: `49 49 2a 00` 또는 `4d 4d 00 2a` ❌

**일치 magic**: PCX (`0a 05`)
- 첫 바이트 `0a` = ZSoft manufacturer
- 두 번째 바이트 `05` = version 5 (PC Paintbrush v3.0+)

### 4. HWP 파일 직접 점검 (정체 확정)

`olefile` 로 HWP CFB 의 `/BinData/` 스트림 인벤토리:

```
BinData/BIN0001.PCX  size=12165   ← 학교 로고 (PCX 포맷)
BinData/BIN0002.jpg  size=81727   ← 워터마크 직인 (JPEG, 정상 출력)
```

→ HWP 가 학교 로고를 **PCX 포맷** 으로 임베드. JPEG (워터마크) 는 표준 포맷이라 정상 출력.

### 5. detect_image_mime_type 점검

`src/renderer/svg.rs:2350~2380` 의 `detect_image_mime_type` 함수:

```rust
pub(crate) fn detect_image_mime_type(data: &[u8]) -> &'static str {
    // PNG / JPEG / GIF / BMP / WMF / TIFF 만 분기
    // 알 수 없는 형식 → 기본값
    "application/octet-stream"
}
```

**PCX 시그니처 (`0a 05`) 처리 누락** → octet-stream 폴백.

## 결함의 본질 (확정)

`detect_image_mime_type` 가 **PCX 포맷 시그니처 (`0a 05`) 를 인식하지 못함** → octet-stream 으로 emit → 브라우저가 이미지로 렌더링 불가.

**정정 단계 두 층**:

### 층 1: MIME 타입 인식 (즉시 정정 가능)

`detect_image_mime_type` 에 PCX 시그니처 추가:

```rust
// PCX: 0A 05 (ZSoft Paintbrush v3.0+)
if data.starts_with(&[0x0A, 0x05]) {
    return "image/x-pcx";
}
```

→ MIME 타입은 정합. 그러나 **브라우저는 PCX 를 native 렌더링하지 못함** → 시각 결함 그대로.

### 층 2: PCX → PNG/JPEG 변환 (본질 정정)

브라우저 호환을 위해 PCX 를 web-safe 포맷 (PNG) 으로 변환 후 emit. 한컴도 내부적으로 PCX/WMF 등 비표준 포맷을 화면 그릴 시점에 변환해서 그린다.

Rust 생태계의 PCX 디코더:
- `image` crate (이미 dependency 일 가능성) 의 PCX 지원 여부 확인 필요
- 또는 `pcx` crate (lightweight)

## 다른 fixture 의 PCX 사용 점검

본 결함이 복학원서.hwp 한정인지 다른 fixture 에도 영향 있는지 sweep:

```bash
find /home/edward/mygithub/rhwp/samples -name "*.hwp" -exec sh -c '...' \;
```

→ 별도 점검 필요 (Stage 2 구현 계획서에서 진행).

## 위험 영역 (재정리)

수행 계획서의 위험 영역은 모두 잘못된 가설 기반. **새 위험 영역**:

| 위험 | 가능성 | 비고 |
|------|--------|------|
| PCX 디코더 dependency 추가로 binary 크기 증가 | 🟨 중간 | `image` crate 가 이미 있다면 feature flag 추가만으로 처리 가능. `pcx` crate 는 lightweight |
| PCX 디코더 결함으로 다른 PCX 이미지에서 panic 또는 깨진 출력 | 🟨 중간 | 본 fixture (BIN0001.PCX, 12 KB) 외 다른 PCX 샘플 점검 필요 |
| 한컴 자체 PCX 처리와 시각 차이 (안티앨리어싱 / 컬러팔레트 매핑 등) | 🟨 중간 | 한컴 정답지와 시각 비교 필요 |
| WMF / EMF 등 다른 비표준 포맷도 동일 결함일 가능성 | 🟢 작음 | WMF 는 detect 분기에 있음. 그러나 WMF→raster 변환이 빠져 있는지 별도 점검 |

## 다음 단계 (구현 계획서 작성)

Stage 2 에서 다음 결정:

1. **MIME 타입 인식 (층 1) 만 정정 vs PCX 디코딩 (층 2) 까지 정정**
   - 층 1 만: 즉시 정정 가능, 그러나 시각 결함 그대로
   - 층 2 까지: PCX 디코더 dependency + 변환 로직 + 회귀 테스트
   - **권장**: 층 2 까지 (시각 결함 정정이 본 task 목표)

2. **PCX 디코더 선택**: `image` crate 의 PCX feature vs 별도 `pcx` crate

3. **변환 시점**: parse 시 한 번 변환 vs render 시점마다 변환
   - parse 시 변환: HWP IR 의 BinData 가 web-safe 포맷으로 통일
   - render 시점: BinData 원본 보존 + 출력 시점에만 변환
   - 라운드트립 정합 (HWP→HWP 저장) 을 위해 **render 시점 변환** 권장

4. **회귀 테스트**: 복학원서.hwp 의 SVG 출력에서 학교 로고 image href 가 `data:image/png` 또는 `data:image/jpeg` 임을 검증

## Stage 1 산출물

- 본 보고서 (`mydocs/working/task_m100_514_stage1.md`)
- 진단용 SVG: `output/svg/task514/복학원서.svg` (380,804 bytes)

## 메모리 정합

- `feedback_search_troubleshootings_first` — 사전 검색 완료, 직접 관련 트러블슈팅 없음
- `feedback_hancom_compat_specific_over_general` — 본 task 는 PCX 포맷 추가 (case-specific) 라 일반화 알고리즘 회피와 정합
- `reference_authoritative_hancom` — 한컴이 PCX 를 어떻게 변환하는지 정답지 비교 (Stage 5 시각 판정)

## 작업지시자 승인 요청

본 진단 결과 (PCX 포맷 인식 누락) 를 본질 결함으로 확정하고, Stage 2 구현 계획서 작성으로 진행 가능한지 승인 요청.
