---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 이슈 초안: BMP 임베딩 HWP 문서가 SVG에서 렌더링되지 않음

> 작성일: 2026-04-22
> 용도: GitHub Issue 등록용 본문 초안 (edwardkim/rhwp)

---

## 제목

BMP 임베딩 HWP 문서가 SVG에서 렌더링되지 않음

## 현상

BMP 이미지(또는 OLE 객체의 BMP 미리보기)를 포함한 HWP 문서를 `export-svg`로 변환하면, SVG 안에 `<image xlink:href="data:image/bmp;base64,...">` 형태로 BMP 원본이 그대로 임베딩된다. 주요 브라우저(Chrome/Firefox)는 SVG `<image>` 요소 내부의 `data:image/bmp` URI를 표준 지원하지 않아 이미지 영역이 빈 공백으로 나타난다.

## 재현 방법

1. 샘플 파일
   - `bitmap.hwp` — 순수 비트맵 이미지가 본문에 삽입된 문서
   - `한셀OLE.hwp` — 한셀 OLE 객체(BMP 미리보기 포함) 문서
2. 환경: `rhwp export-svg <파일>` 로 SVG 생성 후 브라우저(Chrome/Firefox)에서 열기
3. 발생 페이지: 1페이지 (해당 이미지가 나타나는 모든 페이지)

재현 커맨드:

```bash
cargo run --bin rhwp -- export-svg bitmap.hwp
cargo run --bin rhwp -- export-svg 한셀OLE.hwp
# 생성된 output/bitmap.svg, output/한셀OLE.svg 를 브라우저에서 열면 이미지가 비어 보임
```

## 기대 결과

한/글에서 보이는 것과 동일하게, 비트맵 이미지와 한셀 OLE 미리보기가 SVG에서도 정상적으로 표시된다.

## 실제 결과

생성된 SVG를 브라우저에서 열면 해당 이미지 자리가 빈 영역으로 보인다. SVG 자체는 정상 생성된다.

- `output/bitmap.svg` 약 7.6MB
- `output/한셀OLE.svg` 약 174KB
- 내부 `<image>` 태그의 `xlink:href`가 `data:image/bmp;base64,Qk0...`로 시작

관련 코드 위치: `src/renderer/svg.rs:2110` — `detect_image_mime_type`에서 BMP 시그니처(`0x42 0x4D`)를 만나면 `image/bmp`를 그대로 반환하며, 상위 임베딩 로직에서 별도 변환 없이 data URI로 삽입됨.

## 환경

- rhwp 버전: `devel` 브랜치 HEAD (커밋 `118b08b`)
- 브라우저: Chrome / Firefox (SVG `<image>` 내 `data:image/bmp` 미렌더)
- OS: Linux

## 해결 방향 (제안)

SVG 내보내기 시 BMP로 감지된 이미지 데이터는 `image` crate로 디코드하여 PNG로 재인코딩한 뒤 `data:image/png;base64,...`로 임베딩한다. 디코드 실패 시 원본 BMP 유지(폴백).

- 적용 지점: `src/renderer/svg.rs` 이미지 임베딩 경로 한 곳
- 영향 범위: SVG 내보내기만 해당 (HWPX 직렬화 등 다른 경로 무영향)
- 파일 크기: PNG 재인코딩으로 일반적으로 감소 (32-bit BMP 기준)
