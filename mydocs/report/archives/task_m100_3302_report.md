# Task #3302 — export-png/pdf 외부 연결 그림 적재 누락 (최종 보고서)

`Closes #3302`. #3032 시각 판정 중 발견된 "SO-SUEOP 1쪽 그림 미표시(회색 placeholder)"의
CLI 축 정정 기록.

## 진단 여정 (오진 2회 → 확정)

1. **1차 가설(기각)**: skia가 1bpp BMP를 디코드 못 한다 — 합성 1bpp BMP를 skia가
   직접 디코드함을 단위 테스트로 반증. stage2 v1의 image_conv 폴백은 철회·원복.
2. **분류 가설(기각)**: 도형(Shape) 오인식 — IR상 도형 0개, 유일한 Picture 컨트롤로 정상.
3. **확정**: 이 그림은 **HWP3 pic_type=0 '외부 파일 연결'**(#824 계열) — 데이터가 문서
   밖 사이드카(`samples/00000000.OOO`, 실체는 정상 PNG 180×602)에 있다.
   `export-svg`/`export-text`는 #741 후속으로 `populate_external_images_from_dir()`
   (HML 제외 가드)를 호출해 적재하지만 **`export-png`/`export-pdf`는 누락** →
   렌더 트리 ImageNode가 비어 skia가 placeholder(회색 96,96,96 α48 → 합성 225,
   실측 99.1%와 일치)를 그렸다.

## 수정

`src/main.rs` 2곳 — export-svg와 동일한 가드 블록 이식:
- `export_png`: `DocumentCore::from_bytes` 직후 populate (DocumentCore 기구현 메서드,
  페이지 트리 캐시 무효화 포함)
- `export_pdf`: `HwpDocument::from_bytes` 직후 동일 (DerefMut→DocumentCore)

## 검증

- 실측: 1쪽 이미지 영역 placeholder(회색 99.1%) → 흑백 스캔 실 콘텐츠(백 75.7%/흑 18.2%),
  SVG 경로 복원본과 픽셀 상관 **0.932**
- 회귀: 42·43쪽(devel 대비) diff **0** — #3306 판정 결과 보존. 전체 release-test
  **4005/4005**, fmt clean, Docker wasm 빌드 무영향
- **시각 판정(작업지시자) 통과** — `output/pr3036_judge/page1_compare.png`

## 파생

- studio(canvas2d) 동일 증상은 별개 원인(주입 후 뷰 갱신 부재)으로 **#3313에서 분리 정정**
  — 자동 주입·서빙·canvas2d 백엔드는 정상임을 headless e2e로 판별.
- 진단 중 오진 교훈: "SVG는 되고 skia는 안 된다"는 대칭 증상에서 디코더 차이를 먼저
  의심했으나, 실제 분기는 **파이프라인 앞단(적재 단계)의 명령별 비대칭**이었다.
  백엔드 비교 전에 입력 데이터 동일성부터 검증할 것.
