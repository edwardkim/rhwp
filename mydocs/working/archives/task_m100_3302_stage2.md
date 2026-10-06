# Task #3302 Stage 2 — 구현계획서 (정정판 v2)

> v1(skia image crate 폴백)은 **오진으로 철회**. 근거: ① `samples/00000000.OOO` 는 정상
> PNG(180×602), ② 합성 1bpp BMP 를 skia 가 직접 디코드함(전제 반증), ③ 실제 원인은
> 아래와 같이 CLI 파이프라인의 외부 이미지 적재 누락. image_conv.rs 변경은 원복 완료.

## 확정 진단 (v2)

- SO-SUEOP 1쪽 그림은 **HWP3 pic_type=0 "외부 파일 연결" 그림** (#824 처리 계열) —
  문서 안에 데이터가 없고 같은 디렉터리의 `00000000.OOO`(실체는 PNG)를 찾아야 한다.
- `export-svg`/`export-text` 는 #741 후속으로 `populate_external_images_from_dir()`
  (HML 제외 가드)를 호출하지만, **`export-png`/`export-pdf` 는 호출 누락** →
  렌더 트리 ImageNode 에 바이트가 없어 skia 가 회색 placeholder 를 그림.
- **wasm/studio(canvas2d)에서도 미표시** (작업지시자 실측): 브라우저는 임의 로컬 파일을
  읽을 수 없어 같은 증상 — 단 wasm 은 `inject_external_image` API 가 이미 있으므로
  studio 쪽 공급 배선 문제로 **별도 이슈 분리** (본 태스크 범위 밖).

## 변경 명세 (v2)

`src/main.rs` 2곳 — export-svg 와 동일한 가드 블록 이식:

1. `export_png`: `DocumentCore::from_bytes` 직후
   `allows_implicit_sibling_resources(detect_format)` 가드 + `core.populate_external_images_from_dir(parent)`
   (DocumentCore 에 기구현, 페이지 트리 캐시 무효화 포함)
2. `export_pdf`: `HwpDocument::from_bytes` 직후 동일 블록 (DerefMut→DocumentCore)

## 검증 (v2)

1. 실측: `export-png SO-SUEOP.hwp -p 0` — 이미지 영역이 placeholder(회색 225)가 아닌
   실 콘텐츠. SVG 경로 복원본과 영역 픽셀 상관 확인.
2. `export-pdf` 1쪽에서도 이미지 포함 확인.
3. 전체 release-test + 42·43쪽(#3306 판정 페이지) diff 0 유지.
4. 1쪽 before/after 대조 → 작업지시자 시각 판정.

## 후속 분리

- studio/wasm 외부 연결 그림 공급 배선(파일 동반 업로드 → `inject_external_image`) —
  신규 이슈 등록.
