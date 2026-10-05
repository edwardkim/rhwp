# 실제 등록 글꼴의 Print 대조

원본3개는 실제 public API의 production HWP adapter 저장본이며 PDF3개는 한컴2020 MCP의 Print(method0,one-up,11.0.0.9136) 출력이다. 입력/PDF SHA와 job은 [개별 review](../../../archives/pr_7504_review.md)에 기록한다. HCR 기본과 use_font_space 모두97.96159%, Arial98.11853%를 Native/fresh WASM에서 확인했다. 글꼴 전용 규칙은 미검증으로 닫으며 일반 SFNT/Arial 커닝은 유지한다.

`registered-font-replay.rs`는 실제 DocCore에 exact source를 등록하고 Print SVG/tree를 출력한다. `registered-font-replay.mjs`는 실제 브라우저 HwpDocument에서 source를 등록해 Print SVG/tree와 Canvas 원점·scale을 산출한다. source 글꼴은 시스템 설치본을 지정하며 Git에 포함하지 않는다. 별도 SVG 래스터 브라우저에도 같은 bytes를 공급한다.

Linux 저장소 루트에서 공유 캐시의 해당 source Native lib와 fresh pkg를 먼저 빌드한다. 모든 진단 출력은 ignored `output/pr-review/semanticist21-20261005` 아래에 둔다. 원본을 이 경로의 `registered-hcr`, `registered-arial`, `registered-hcr-fontspace` 하위로 복사하고 SHA를 대조한 뒤 실행한다.

```bash
rustc --edition=2021 mydocs/pr/assets/semanticist21-20261005/pr7504/registered-font-replay.rs \
  -L dependency=target/pr-review/release-test/deps \
  --extern rhwp=target/pr-review/release-test/deps/librhwp.rlib \
  -o output/pr-review/semanticist21-20261005/registered-font-replay
output/pr-review/semanticist21-20261005/registered-font-replay \
  output/pr-review/semanticist21-20261005/registered-hcr/pr7504-registered-hcr.hwp \
  /usr/local/share/fonts/hwp-convert-mcp-survey/8664669bd2d6-HANBatang.ttf \
  output/pr-review/semanticist21-20261005/registered-hcr
node mydocs/pr/assets/semanticist21-20261005/pr7504/registered-font-replay.mjs \
  --pkg pkg \
  --input output/pr-review/semanticist21-20261005/registered-hcr/pr7504-registered-hcr.hwp \
  --font-file /usr/local/share/fonts/hwp-convert-mcp-survey/8664669bd2d6-HANBatang.ttf \
  --out output/pr-review/semanticist21-20261005/font-contract-wasm-export/registered-hcr
```

Arial에는 실제 source `525979822591-Arial.ttf`를 쓴다. `registered-hcr-export.py`와 `registered-wasm-export.py`는 위 실제 API 산출물을 `visual_sweep.py --rhwp-bin`의 export-svg/export-render-tree 계약으로 연결한다. Native는 등록 source로 SVG font-face의 공급원만 고정하고 좌표를 유지한다. WASM은 실제 브라우저 exporter가 공급원을 고정하며 adapter가 입력 SHA를 검사한다. metadata 명령만 원 CLI로 전달한다. 따라서 등록 없는 일반 CLI/WASM export를 등록 경로의 증거로 바꾸지 않는다. 각 adapter를 `--rhwp-bin`으로 지정하고 원본/2020 PDF/key/`--embed-fonts=full`/ignored `--out`을 전달한다.

대표 review/overlay는 Native/WASM별 PNG12개로 보존한다. Canvas 관측 JSON·패키지/source/글꼴 해시·TSV·직접 테스트 원 로그는 Git에 넣지 않는다. 합성 회귀의 전용 marker는 지원 경계 검사이며 실제 한컴 글꼴의 전용 테이블을 해독한 fixture가 아니다.
