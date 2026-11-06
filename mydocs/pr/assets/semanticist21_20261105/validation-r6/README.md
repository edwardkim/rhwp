# R6 최종 시각 증적

제품 `86709ac16711820e61baa3b87e2c4ddaac23573e`의 실제 새 출력이다.
Native·최적화 fresh WASM 각각 128입력·301쪽이고 독립 Print PDF 전체 쪽수와 같다.
Native 전쪽 최저 91.81401%, fresh WASM 전쪽 최저 91.17272%, 미달·누락 0쪽.
새 base `170b886f167b705b6e630c5b34bb638d92fae0a9`의 각 301쪽 대비 하락 자체가 0쪽이다.

- [원본·PDF·producer SHA와 입력별 전쪽 결과](measurements.md)
- [실제 MCP Print job·한컴 제품·Print 경로](print-sources.md)
- [현재 전쪽 TSV 원문](tsv) · [같은 base의 전쪽 TSV 원문](base-tsv)
- [반올림 전 전쪽 base 대조](base-comparison.md)
- [주장 페이지·직접 판독·최종 review/overlay](panels.md)
- [실제 공급 글꼴·동일 WOFF 변환의 조건 대조](fonts.md)
- [17개 고정 페이지와 회귀 첫머리 근거](qualification.md)
- [실제 API·독립 CFB·브라우저·집중 회귀](runtime-checks.md)

## 재현 조건과 명령

2026-11-06 KST, 96 DPI, Print 프로필, PDFium 5.13.0/153, 색상 경계 보정 전 2px raw.
사용한 원본과 독립 Print는 measurements.md의 정식 파일과 같은 SHA다.
Mac 12.30 Print·2020/2024 MCP Print의 출처를 구분하고, PDF를 재출력하거나 이름만 바꿔 기준을 바꾸지 않는다.
시각 검증에는 실제 설치 글꼴 전체 1,906 face와 기존 승인 대체 설정을 공급한다.
회귀는 저장소 공개 글꼴 5개만 노출한 별도 최소 환경이며, 비공개 글꼴 프로그램을 포함하지 않는다.

```bash
venv/bin/python scripts/visual_sweep.py --silhouette-only --resume \
  --hwp <정식 원본> --pdf <독립 Print PDF> --key <키> --compat <입력 compat> \
  --dpi 96 --page-workers 1 --rhwp-bin <동일 SHA Native> --out output/pr-review/<검토>/<키>
```

WASM은 현재 최적화 패키지를 정본 exporter에 공급해 전체 쪽을 새로 출력했다.
options에는 pages=null, workers=1, scale=1, compat·fontEnvironment·linkedImages를 지정했다.
대형 입력은 protocolTimeoutMs=3600000·명령 timeout 14400초를 명시하며 점수는 바꾸지 않았다.

```bash
node scripts/export-wasm-png-for-sweep.mjs --pkg <fresh 최적화 패키지> \
  --input <정식 원본> --out <출력> --fonts <정본 helper의 실제 시스템 catalogue> --options <options 파일>
venv/bin/python scripts/visual_sweep_pdf.py --pdf <독립 Print PDF> --dpi 96 --prefix <출력>/pdf_png/pdf
venv/bin/python scripts/visual_sweep.py --silhouette-only --resume \
  --png-pair <출력>/rhwp_png <출력>/pdf_png --key <키> --out <출력>/scores
```

PNG pair에는 정본 run_manifest가 없으므로 전체 exporter metadata의 페이지·PNG SHA와
독립 PDF 쪽수를 별도로 검증했다. 없는 manifest를 위조하지 않았다.
90% 또는 100% raw를 RGB 완전 동일·내용 전수 동일로 표현하지 않는다.
PR #8564의 동적 파일명 재평가 차이는 직접 판독과 API 기록에 명시한다.

공식 Native 편람 383쪽 gate는 실제 exit 0·최저 99.96%·공개 기준 하락 0으로 완료했다.
최소 공개 글꼴 5개에서 전체 Rust 12,740건 PASS·기존 ignored 45건, 실제 목록 누락·중복 0을 확인했다.
후속 Studio 보정 `5feccb130e`는 Rust 생산 코드·Native/WASM bytes와 직접 exporter를 바꾸지 않는다.
전체 Node 2,744건·production build·실제 Studio 12캡처는 새 소스에서 다시 통과했다.
[실제 실행과 재사용 경계](runtime-checks.md)를 연결하며, [최종 Cargo·전체 Rust·Studio 검증 기록](final-validation.md)에 실제 종료 결과를 보존했다.

## 보호 PR #8389·#8397

[현재 Native/fresh WASM 전쪽 TSV 34개·전체 대조](protected/README.md)를 함께 제출한다.
각 backend 17개 고유 문서 1770쪽, 공개 기준의 중복 보호 행을 포함한 2153행을 대조해
표시값 하락·누락·오류 0이다. 상승은 허용하고 소수 셋째 자리에서 반올림한 표시값을 비교했다.

## 실제 Studio의 추가 확대 판독

[PR #8616 같은 두 원본 1쪽의 최종 review·overlay·TSV와 실제 backend 화면](../studio-symbol/README.md)을 연결한다.
직접 exporter의 결과를 Studio 화면으로 재표기하지 않았다. 자동 컬러 이모지와 반각 크기 차이를
기존 전각 공급·공통 후보 병합에서 보정했고 실제 Canvas2D도 두 원본 모두 raw 100.00%를 확인했다.
원시 실행 로그·metric/run JSON은 output의 로컬 검증 자료이며 제출하지 않는다.

[최종 로컬 검증의 모든 명령·종료 코드·재사용 경계](final-validation.md)를 연결한다.
