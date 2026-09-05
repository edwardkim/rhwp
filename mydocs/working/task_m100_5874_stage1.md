# #5874 Stage 1: 최신 devel 재현과 원인

- Issue: #5874
- 기준: `upstream/devel` = `2c144b180dd776aa450c499778510199ae6cdf89`.
- 브랜치: `fix/5874-pdf-synthetic-italic`.
- 결론: **아직 재현됨**. 1페이지 최소 재현 HWPX와 `<hh:italic/>`만 제거한 대조본의
  PDF SHA-256이 모두 `994d2664fef74a5a801d7075c24554512c3e865f48b6603aac976b70081d34e1`이다.
  144 DPI raster 차이 0 pixel, 페이지 수와 추출 텍스트도 동일하다.

## 명령과 증거

```bash
cargo build --locked --profile release-test --target-dir target/pr-review --bin rhwp
venv/bin/python mydocs/pr/assets/issue_5874/reproduce.py \
  --binary target/pr-review/release-test/rhwp --label before
```

- 빌드 성공: 2분 40초.
- 입력/출처: [샘플 설명](../../samples/issue5874/README.md).
- 실측: [before-comparison.json](../pr/assets/issue_5874/before-comparison.json).
- PDF: [원본](../../pdf/issue_5874/before.pdf), [기울임 제거 대조본](../../pdf/issue_5874/before-upright.pdf).
- 작성자 기준: [한컴 화면](../pr/assets/issue_5874/reporter-hancom.png).

## 원인과 범위

기본 `export-pdf`는 SVG/usvg/svg2pdf 경로다. SVG의 `font-style="italic"`은 유지되지만
usvg가 기울임 face 없는 한글 폰트의 정규 face를 선택하면 svg2pdf는 해당 글리프를 그대로 그린다.
Text IR v2의 `syntheticStyleAuthorityPending`을 제거하거나 flag를 일괄 활성화할 문제가 아니다.

다음 단계는 실제 선택된 glyph face가 정규일 때만 수평 baseline을 고정한 shear를 적용한다.
실제 italic/oblique face는 유지하고, 혼합 face/복잡한 배치는 경고로 구분한다.
기본 PDF 경계만 수정하며, 선택적 Skia direct PDF/공통 Text IR authority까지 해결했다고
주장하지 않는다. 레이아웃과 텍스트 선택 기능을 보존하는 계약 테스트를 추가한다.
