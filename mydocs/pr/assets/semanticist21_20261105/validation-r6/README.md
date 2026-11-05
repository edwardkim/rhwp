# R6 실제 검증 증거

제품 source는 `86709ac16711820e61baa3b87e2c4ddaac23573e`, 빌드 시 Git HEAD는 `1da15804ebb5d6fc8f63b7ccb8410018bd42c29d`다.
이후 변경한 테스트·주석·증적은 제품 소스와 구분한다. Native와 최적화 fresh WASM은 각각 새로
생성한 실제 출력이며, R5 점수를 R6로 바꿔 적지 않았다.

| 경로 | 입력 수 | 전체 쪽수 | 전쪽 최저 raw |
| --- | ---: | ---: | ---: |
| Native | 128 | 301 | 91.81401% |
| fresh WASM | 128 | 301 | 91.17272% |

전체 독립 PDF 쪽수와 RHWP 쪽수가 같고 미달·측정 누락이 없다.
각 경로 301쪽의 고정 base `170b886f167b705b6e630c5b34bb638d92fae0a9` 대조에서도
하락 자체가 0쪽이다. [Native 대조](native-new-base-comparison.json)와
[WASM 대조](wasm-new-base-comparison.json)에 반올림 전 수치를 보존했다.
입력별 전쪽 TSV는 [tsv](tsv), 변경 전 base TSV는 [base-tsv](base-tsv)다.
후보와 변경 전 결과를 혼동하지 않는다.

## 원본·측정·직접 판독

[measurements.json](measurements.json)에 원본·독립 한컴 Print PDF의 정식 경로와 SHA-256,
compat, DPI, 전체 쪽수, 정밀 raw, producer, 실제 공급 글꼴을 입력·backend별로 기록했다.
원본과 PDF는 [기존 입력 목록](../validation-r5/inputs.json)의 정식 파일과 같은 SHA임을 확인했다.
Mac 12.30 Print와 2020/2024 MCP Print는 출처를 구분하며 임의로 출력 버전을 바꾸지 않는다.
[panels.json](panels.json)은 원 PR 주장 → 같은 원본·쪽 → 실제 review/overlay → TSV를 연결한다.
[새 직접 판독](direct-readings-new.json)과 [실제 PNG 쌍 동일성에 따른 판독 재사용](direct-readings-reused.json)을 구분했다.
90% 또는 100% raw가 픽셀 완전 동일이나 모든 필드 값의 동일성을 뜻하지 않는다.
특히 PR #8564는 Print 과정에서 재평가된 파일명 필드 값 차이를 판독 기록에 명시했다.

## 재현 명령과 환경

공통 조건은 96 DPI, Print 프로필, 색상 경계 보정 전 2px 내용 실루엣 raw다.
Native 실제 명령은 다음 형식이다. `--font-environment`와 외부 이미지 공급은 measurements의 해당 입력 조건을 유지한다.

```bash
venv/bin/python scripts/visual_sweep.py --silhouette-only --resume \
  --dpi 96 --page-workers 1 --rhwp-bin <동일 SHA의 Native 실행 파일> \
  --hwp <정식 HWP/HWPX> --pdf <독립 Print PDF> --key <키> --compat <원본 compat> \
  --out output/pr-review/<review-id>/<키>
```

WASM은 정본 exporter로 전체 쪽을 출력한 뒤 정본 PDF raster·PNG pair 점수를 산출했다.
사전 생성 catalogue는 정본 helper가 실제 시스템 전체 글꼴에서 만든 1,906 face 목록이다.
대형 문서는 protocolTimeoutMs=3600000, 전체 export 제한 14400초를 명시했으며 점수 계산은 바꾸지 않았다.

```bash
node scripts/export-wasm-png-for-sweep.mjs --pkg <fresh 최적화 패키지> \
  --input <정식 원본> --out <출력 경로> --fonts <실제 시스템 catalogue> --options <canvas-options.json>
venv/bin/python scripts/visual_sweep_pdf.py --pdf <독립 Print PDF> --dpi 96 --prefix <출력 경로>/pdf_png/pdf
venv/bin/python scripts/visual_sweep.py --silhouette-only --resume \
  --png-pair <출력 경로>/rhwp_png <출력 경로>/pdf_png --key <키> --out <출력 경로>/scores
```

PNG pair 모드에는 정본 `run_manifest.json`이 생기지 않는다. 실제 exporter 영수증의 모든 페이지·PNG SHA와
독립 PDF 전체 쪽수를 별도로 대조했다. 없는 manifest를 위조하지 않았다.
Native [producer](native-producer.json), WASM [producer](wasm-producer.json),
[시스템 catalogue 동일성](system-font-catalog-comparison.json), [동일 WOFF 테이블 대조](font-table-comparison.json)를 보존했다.
비공개 글꼴 파일은 포함하지 않는다. 시각 검증은 설치 글꼴 전체를 사용하며,
회귀 테스트는 저장소 공개 글꼴 5개만 노출한 별도 최소 환경에서 실행한다.

## 동작과 회귀

- [빈 번호 머리 공통 간격 보정](empty-numbering/README.md): 수정 전 2 FAIL, 수정 후 원본·0 거리 대조·독립 원점 등 8 PASS.
- [Mac 네 원본의 전체 1쪽·내용 보존 검사](mac-page-contracts-minimal.txt): 관련 6 PASS.
- 27개 테스트 파일 첫머리에 실제 양쪽 전쪽 수치와 이전 Git 시각을 기록했다.
  [17개 고정 페이지 승인](pin-reading-approval.json)은 양쪽 raw 98% 이상·독립 확대 판독 근거다.
- [현재 WASM 실제 API](api/actual-results.json) 12그룹 PASS.
  [저장 바이트 동일성](api/saved-byte-comparison.json)으로 기존 독립 CFB 검사의 재사용 범위를 제한했다.
- 실제 브라우저 스타일 편집·실패 원자성·undo, base PR #8630 그룹 편집·undo/redo/Esc,
  TypeScript·Vite production build 및 실제 생성 타입 export·인수 검사와 소비 테스트를 [ui](ui)에 보존했다.

최종 커밋의 공식 Native 383쪽 편람 게이트와 R6 전체 Rust·lint·Skia·release·doctest는 이 단계 뒤 실행한다.
이 파일의 후보 수치 완료를 해당 필수 게이트 완료나 최종 머지 승인으로 읽지 않는다.

보존한 실행 로그는 원래 출력의 마지막 빈 줄만 정리했다.
