# hwpx2hwp 이미지 리소스 보존/렌더링 설계 메모

## 1. 문제 정의

HWPX/HWP 문서의 이미지 리소스에는 웹 렌더러가 직접 표시하기 어려운 포맷이 포함될 수 있다.

예:

```text
WMF
EMF
BMP
PCX
OLE 내부 native image / preview image
```

rhwp-studio의 웹 canvas/SVG 렌더링에서는 이런 포맷을 브라우저 친화 포맷으로 변환해야 한다.

예:

```text
WMF -> SVG
BMP -> PNG
PCX -> PNG
EMF -> SVG 또는 PNG 계열
```

그러나 `hwpx -> IR -> hwp save`에서는 이 렌더링용 변환 결과를 HWP 저장 리소스로 사용하면 안 된다.
HWP 저장은 원본 BinData 바이트와 원래 확장자/저장 ID/압축 정책을 보존해야 한다.

## 2. 현재 코드 관찰

### BinDataContent 모델

현재 IR의 실제 바이너리 payload는 다음 구조다.

```rust
pub struct BinDataContent {
    pub id: u16,
    pub data: Vec<u8>,
    pub extension: String,
}
```

즉 현재 모델은 다음 둘을 구분하지 않는다.

```text
원본 저장용 바이트
렌더링용 변환 바이트
```

### HWP 저장 경로

`serializer/cfb_writer.rs`는 `BinDataContent.data`를 그대로 `/BinData/BINxxxx.ext`에 쓴다.

```text
content.data -> optional compress -> /BinData/BIN{storage_id}.{ext}
```

따라서 렌더러가 `BinDataContent.data`를 PNG/SVG 변환 결과로 덮어쓰면,
HWP 저장 결과도 원본 WMF/BMP가 아니라 변환 산출물로 바뀐다.

### 렌더러 경로

`renderer/svg.rs`는 WMF/BMP/PCX를 표시용으로 변환한다.

```text
WMF -> SVG
BMP -> PNG
PCX -> PNG
```

이 변환은 `Cow::Owned`로 렌더링 함수 내부에서만 쓰는 형태라 현재는 원본 `BinDataContent.data`를 직접 변경하지 않는다.

`renderer/web_canvas.rs`도 WMF/PCX 등에 대해 표시용 변환을 수행한다.

## 3. 설계 원칙

### 원칙 1: 저장용 원본과 렌더용 파생물을 분리한다

IR의 `BinDataContent.data`는 기본적으로 저장용 원본 바이트여야 한다.

```text
BinDataContent.data = canonical/original storage payload
renderer-derived bytes = transient render resource
```

렌더러는 필요할 때만 파생 바이트를 만든다. 파생 바이트는 문서 저장에 사용하지 않는다.

### 원칙 2: HWP 저장은 원본 포맷을 보존한다

HWP는 WMF/BMP 등 legacy image payload를 담을 수 있다. 따라서 HWP 저장 시에는 다음을 유지한다.

```text
storage_id
extension
compression
attr/type/status
raw payload bytes
```

예:

```text
원본 BinData: BIN0003.wmf
렌더링: WMF -> SVG data URL
HWP 저장: BIN0003.wmf 그대로 저장
```

### 원칙 3: HWPX 저장도 원본 포맷을 우선 보존한다

HWPX ZIP 내부 `BinData/` 엔트리 역시 원본 확장자와 MIME을 가능한 보존한다.

단, 사용자가 새 이미지를 삽입하거나 실제 이미지 편집으로 rasterize가 필요한 경우에만 새 포맷을 문서 리소스로 채택한다.

### 원칙 4: 렌더 캐시는 DocumentCore/Renderer 계층에 둔다

렌더 성능을 위해 변환 캐시가 필요할 수 있다. 하지만 캐시는 저장 IR과 분리해야 한다.

후보:

```text
RenderImageCache
  key: (bin_data_id, original_hash, target_backend)
  value: RenderImageBytes { mime, data }
```

위치는 다음 중 하나가 적절하다.

```text
DocumentCore runtime cache
Renderer runtime cache
rhwp-studio frontend cache
```

금지:

```text
BinDataContent.data를 렌더링 변환 결과로 덮어쓰기
BinDataContent.extension을 렌더링 MIME 기준으로 바꾸기
```

### 원칙 5: 실제 편집은 명시적 리소스 교체로 처리한다

사용자가 이미지를 새로 삽입하거나 기존 이미지를 실제로 변경하면 그때는 원본 리소스가 바뀐다.

이 경우는 렌더 변환이 아니라 편집 명령이다.

예:

```text
replace_picture(bin_id, png_bytes, "png")
insert_picture(...)
crop/brightness/effect changes: payload 보존, Picture attr 변경
```

## 4. #903과의 관계

#903 Stage 2의 수정은 embedded BinData의 HWP record attr/type/status를 복구한다.

하지만 이미지 포맷 변환 설계는 별도 축이다.

Stage 2가 보장한 것:

```text
HWP 저장 후 재로드 시 embedded BinData 5개가 사라지지 않음
```

아직 별도 검증이 필요한 것:

```text
WMF/BMP/PCX 등 비웹 포맷이 renderer 변환 후에도 HWP 저장에서 원본 바이트로 보존되는지
grouped picture / OLE preview / native image가 저장 경로에서 섞이지 않는지
```

## 5. 필요한 테스트

### 테스트 A: BMP payload 보존

입력:

```text
HWPX 또는 HWP 샘플: embedded BMP BinData 포함
```

검증:

```text
1. 로드 후 renderer가 PNG로 표시 가능
2. HWP 저장 후 재로드
3. BinData extension이 bmp로 유지
4. BinDataContent bytes sha256이 저장 전후 동일
```

### 테스트 B: WMF payload 보존

검증:

```text
1. 렌더링에서는 WMF -> SVG 변환 사용
2. HWP 저장 결과에는 BINxxxx.wmf가 유지
3. 저장 후 재로드한 BinDataContent sha256이 원본과 동일
```

### 테스트 C: PCX payload 보존

검증:

```text
1. 렌더링에서는 PCX -> PNG 변환 사용
2. HWP 저장 결과에는 원본 PCX payload가 유지
```

### 테스트 D: 새 이미지 삽입은 예외적으로 새 payload 사용

검증:

```text
insertPicture(... png ...)
저장 후 BinData extension=png
payload sha256 = 삽입한 png
```

## 6. CLI/진단 도구 확장

`hwpx2hwp-probe` 또는 `ir-diff --focus bindata`에는 다음 비교가 필요하다.

```text
bin_data_id
extension
mime by magic
record attr
record data_type/status/compression
content byte length
content sha256
render mime if transformed
render byte length
render sha256
```

중요한 출력 구분:

```text
storage: original payload
render: backend-compatible derivative
```

예:

```text
BinData[3]
  storage: id=3 ext=wmf mime=image/x-wmf sha256=...
  render: mime=image/svg+xml generated=true sha256=...
  save_policy: preserve-storage
```

## 7. 권장 구현 방향

### 단기

#903에서는 현재 수정 범위를 유지한다.

```text
BinData attr/type/status 정규화
```

이미지 변환 설계 변경은 섞지 않는다.

### 중기

별도 이슈로 다음을 진행한다.

```text
[hwpx2hwp][image-resource] 렌더용 이미지 변환과 저장용 BinData 보존 정책 정리
```

작업 순서:

1. BinData sha256 비교 유틸 추가
2. `ir-diff --focus bindata` 또는 `hwpx2hwp-probe`에 storage/render 구분 출력 추가
3. BMP/WMF fixture 확보
4. 저장 전후 payload 보존 테스트 추가

### 장기

IR 모델 확장 검토:

```rust
pub struct BinDataContent {
    pub id: u16,
    pub data: Vec<u8>,
    pub extension: String,
    pub source_kind: BinDataSourceKind,
}

pub struct RenderImageDerivative {
    pub bin_data_id: u16,
    pub backend: RenderBackend,
    pub mime: String,
    pub data: Vec<u8>,
}
```

단, 렌더 파생물은 문서 모델에 영구 저장하기보다 runtime cache에 두는 편이 안전하다.

## 8. 결론

정책은 다음 한 줄로 고정한다.

```text
렌더러는 표시 가능한 포맷으로 변환할 수 있지만, HWP/HWPX 저장은 원본 BinData payload를 보존한다.
```

이 원칙이 깨지면 한컴 호환성과 왕복 저장 신뢰성이 동시에 무너진다.
