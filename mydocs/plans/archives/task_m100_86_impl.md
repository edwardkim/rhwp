# Task #86: HWP 썸네일 자동 추출 + data-hwp-thumbnail 연동 — 구현 계획서

## 1단계: WASM API + 라이브러리 함수

### 목표

전체 문서 파싱 없이 CFB의 PrvImage 스트림만 빠르게 추출하는 API를 제공한다.

### 구현

**`src/parser/mod.rs`** — 썸네일 전용 경량 추출 함수 추가:
```rust
/// CFB에서 PrvImage만 추출 (전체 파싱 없이)
pub fn extract_thumbnail_only(data: &[u8]) -> Option<PreviewImage>
```

**`src/wasm_api.rs`** — WASM API 노출:
```rust
#[wasm_bindgen(js_name = extractThumbnail)]
pub fn extract_thumbnail(data: &[u8]) -> Result<JsValue, JsValue>
// 반환: { format: "bmp"|"gif", base64: "...", width: N, height: N }
```

- BMP → PNG 변환 포함 (브라우저 호환성, `image` 크레이트 활용)
- GIF는 그대로 base64 반환
- PrvImage 없는 경우 `null` 반환

### 검증

- `cargo test` — 썸네일 추출 단위 테스트 (samples/ 내 HWP 파일)
- PrvImage 있는 파일 / 없는 파일 모두 테스트

## 2단계: CLI 명령 (`rhwp thumbnail`)

### 목표

CLI에서 썸네일을 PNG 파일 또는 base64로 추출한다.

### 구현

**`src/main.rs`** — `thumbnail` 서브커맨드 추가:

```bash
rhwp thumbnail sample.hwp                    # output/sample_thumb.png 저장
rhwp thumbnail sample.hwp -o my_thumb.png    # 지정 경로에 저장
rhwp thumbnail sample.hwp --base64           # base64 문자열 stdout 출력
rhwp thumbnail sample.hwp --data-uri         # data:image/png;base64,... 형식 출력
```

- 배치 처리 예시: `for f in *.hwp; do rhwp thumbnail "$f" -o "thumbs/${f%.hwp}.png"; done`
- PrvImage 없는 파일: 에러 메시지 출력 + exit code 1

### 검증

- 실제 HWP 파일로 PNG 추출 확인
- `--base64`, `--data-uri` 출력 확인
- PrvImage 없는 파일 에러 처리 확인

## 3단계: Chrome 확장 연동

### 목표

`data-hwp="true"` 링크 호버 시 썸네일을 자동 추출하여 미리보기를 표시한다.

### 구현

**`rhwp-chrome/content-script.js`** — 호버 이벤트에 썸네일 로직 추가:

1. `data-hwp="true"` 링크에 마우스 호버 감지
2. `data-hwp-thumbnail` 속성이 있으면 해당 이미지 사용 (사전 생성)
3. 없으면 HWP 파일을 Range 요청으로 부분 다운로드
4. WASM `extractThumbnail()`로 PrvImage 추출
5. 툴팁/팝업으로 썸네일 표시
6. 추출 결과 캐싱 (동일 URL 재요청 방지)

### 우선순위 로직

```
data-hwp-thumbnail 속성 있음? → 해당 이미지 표시
                               ↓ 없음
HWP 파일 부분 다운로드 → WASM extractThumbnail()
                               ↓ PrvImage 없음
첫 페이지 SVG 렌더링 (폴백, 향후 구현)
```

> 3단계의 Range 요청 + WASM 추출은 복잡도가 높으므로, 먼저 1~2단계를 완성하고 3단계는 Chrome 확장 첫 배포 후 별도 패치로 진행할 수 있다.

### 검증

- `data-hwp-thumbnail` 속성 있는 링크: 해당 이미지 표시 확인
- 속성 없는 링크: WASM 추출 + 캐싱 동작 확인
- PrvImage 없는 HWP: 폴백 동작 확인

## 검증 (전체)

- `cargo build` + `cargo test` 통과
- CLI `rhwp thumbnail` 정상 동작
- WASM 빌드 후 브라우저에서 `extractThumbnail()` 호출 확인

## 예상 단계: 3단계
