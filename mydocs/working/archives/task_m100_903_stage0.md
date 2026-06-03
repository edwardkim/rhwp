# Task m100 #903 Stage 0 - hwpx-h-01 저장 손상 진단

## 1. 대상

이슈:

```text
#903 [hwpx2hwp][hwpx-h-01] HWP 저장 손상 및 rhwp-studio 렌더링 깨짐 진단
```

샘플:

```text
samples/hwpx/hwpx-h-01.hwpx
```

## 2. 파일 포맷 확인

파일 크기:

```text
470K
```

`file` 결과:

```text
Hancom HWP (Hangul Word Processor) file, HWPX
```

매직 바이트:

```text
50 4B 03 04
```

즉 ZIP 기반 HWPX가 맞다.

`unzip -l` 결과 주요 항목:

```text
mimetype
version.xml
Contents/header.xml
Contents/section0.xml
Contents/section1.xml
BinData/image1.jpg
BinData/image2.png
BinData/image3.jpg
BinData/image4.jpg
BinData/image5.jpg
Preview/PrvImage.png
```

`mimetype`:

```text
application/hwp+zip
```

## 3. 포맷 감지 경로

코어 파서:

- `src/parser/mod.rs::detect_format`
- `PK\x03\x04`이면 `FileFormat::Hwpx`

rhwp-studio:

- `rhwp-studio/src/main.ts::detectDocumentByteKind`
- ZIP signature이면 `hwpx`

판정:

```text
확장자/매직 포맷 감지 문제는 아니다.
```

## 4. 원본 파싱 결과

명령:

```text
cargo run --bin rhwp -- info samples/hwpx/hwpx-h-01.hwpx
```

결과 요약:

```text
구역 수: 2
페이지 수: 9
총 문단 수: 121
ParaShape: 85
CharShape: 171
표: 26개
BinData: 5개
```

원본 BinData:

```text
[0] Embedding (ID: 1, ext: jpg, loaded: 3072 bytes)
[1] Embedding (ID: 2, ext: png, loaded: 17181 bytes)
[2] Embedding (ID: 3, ext: jpg, loaded: 26127 bytes)
[3] Embedding (ID: 4, ext: jpg, loaded: 295915 bytes)
[4] Embedding (ID: 5, ext: jpg, loaded: 80421 bytes)
```

## 5. 한컴 정답 HWP 확인

작업지시자가 한컴 에디터 기준 정답 HWP를 준비했다.

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

파일 정보:

```text
size: 469K (480256 bytes)
file: Hancom HWP (Hangul Word Processor) file, version 5.0
rhwp info version: 5.1.0.1
```

정답 HWP 구조 요약:

```text
구역 수: 2
페이지 수: 9
총 문단 수: 121
ParaShape: 85
CharShape: 171
표: 26개
BinData: 5개
```

정답 HWP BinData:

```text
[0] Embedding (ID: 1, ext: jpg, loaded: 3072 bytes)
[1] Embedding (ID: 2, ext: png, loaded: 17181 bytes)
[2] Embedding (ID: 3, ext: jpg, loaded: 26127 bytes)
[3] Embedding (ID: 4, ext: jpg, loaded: 295915 bytes)
[4] Embedding (ID: 5, ext: jpg, loaded: 80421 bytes)
```

판정:

```text
원본 HWPX와 한컴 정답 HWP는 페이지/문단/BinData 수와 embedded 이미지 바이트가 일치한다.
따라서 Stage 1에서는 정답 HWP처럼 HWP 저장 후 BinData 5개가 embedded로 생존하는지를 우선 RED 기준으로 삼는다.
```

## 6. Stage 0 산출물

저장 경로:

```text
output/poc/hwpx2hwp/task903/stage0/
```

### adapter 저장본

rhwp-studio 저장 경로와 같은 `export_hwp_with_adapter()` 사용.

```text
output/poc/hwpx2hwp/task903/stage0/hwpx-h-01_adapter.hwp
```

정보:

```text
size: 665K
sha256: d8b5f402e7fc59cba2768e0652388276547137c61ab6de8734e7383d75b7fd97
self reload: ok
구역 수: 2
페이지 수: 9
BorderFill: 82
```

재로드 후 BinData:

```text
[0] Link (ID: 0, ext: ?, loaded: 0 bytes)
[1] Link (ID: 0, ext: ?, loaded: 0 bytes)
[2] Link (ID: 0, ext: ?, loaded: 0 bytes)
[3] Link (ID: 0, ext: ?, loaded: 0 bytes)
[4] Link (ID: 0, ext: ?, loaded: 0 bytes)
```

### native 저장본

어댑터 없이 `export_hwp_native()` 사용. 비교용이다.

```text
output/poc/hwpx2hwp/task903/stage0/hwpx-h-01_native.hwp
```

정보:

```text
size: 663K
sha256: 6ed3eea71408febceebd5f324b6c1fa42ae58b33aa23a9d54158cdfcae724682
self reload: ok
구역 수: 2
페이지 수: 121
```

native 저장본은 PAGE_DEF가 보존되지 않아 121쪽으로 폭주한다.

## 7. SVG 렌더링 산출물

명령:

```text
cargo run --bin rhwp -- export-svg samples/hwpx/hwpx-h-01.hwpx -o output/poc/hwpx2hwp/task903/stage0/source_svg --debug-overlay
```

결과:

```text
9개 SVG 생성
```

경로:

```text
output/poc/hwpx2hwp/task903/stage0/source_svg/
```

로그:

```text
LAYOUT_OVERFLOW: page=0, col=0, para=21, type=Table, overflow=4.0px
LAYOUT_OVERFLOW: page=2, col=0, para=44, type=Table, overflow=4.0px
```

4px overflow는 기록한다. 다만 현재 저장 손상 후보와 직접 연결됐다고 단정하지 않는다.

## 8. 1차 원인 후보

가장 강한 단서:

```text
HWPX 원본: BinData 5개가 Embedding + loaded bytes
한컴 정답 HWP: BinData 5개가 Embedding + loaded bytes
HWP 저장 후 재로드: BinData 5개가 Link + ID 0 + loaded 0 bytes
```

HWPX 파서에서 `doc_info.bin_data_list`를 만들 때 `data_type=Embedding`, `storage_id`, `extension`은 채우지만 `attr`은 기본값 `0`으로 남긴다.

HWP `HWPTAG_BIN_DATA` 파서는 `attr & 0x000F`로 타입을 판정한다.

```text
0 -> Link
1 -> Embedding
2 -> Storage
```

따라서 HWPX 출처 BinData를 HWP로 직렬화하면 `attr=0`이 기록되고, 재로드 시 Link로 해석된다.

정상 이미지 삽입 경로는 다음 attr를 사용한다.

```text
0x0101
```

의미:

- low nibble 1: Embedding
- status bit 8: Success
- compression: Default

## 9. Stage 1 제안

RED 테스트를 먼저 작성한다.

후보 테스트:

```text
task903_hwpx_h_01_embedded_bindata_survives_hwp_save_reload
```

검증:

1. `samples/hwpx/hwpx-h-01.hwpx` 로드
2. 원본 `bin_data_content.len() == 5`
3. `export_hwp_with_adapter()`로 HWP 저장
4. 저장본 재로드
5. 재로드 후 `bin_data_content.len() == 5`
6. 재로드 후 각 BinData가 `Embedding`이며 loaded bytes가 0보다 큼

예상 RED:

```text
재로드 후 BinData가 Link/0 bytes가 되어 실패
```

GREEN 후보:

```text
HWPX 파서 또는 HWPX->HWP 어댑터에서 embedded BinData의 attr를 0x0101로 정규화
```

Stage 1 승인 후 RED 테스트부터 작성한다.
