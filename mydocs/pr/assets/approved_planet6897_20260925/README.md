# 승인 후보 8건 통합 head의 시각 증적

- 비교 base: `b3e3d4e2170a43ca449e3d832440a9274e4e8ee4` (`upstream/devel`)
- 검증 code head: `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88`
- Native 실행 파일: 같은 head에서 빌드한 `release-test/rhwp`
- WASM: 같은 head에서 `scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt`로 새로 빌드한 Mac 로컬 대체 산출물
- DPI 96, Chrome webfont rasterizer, `--embed-fonts=full`; Mac `~/Library/Fonts` 자동 공급
- 입력·기준 PDF·CLI·WASM·검증 스크립트·글꼴 공급 기록의 SHA-256과 쪽별 지표는 [evidence.json](evidence.json)에 있다. 라이선스 글꼴과 대형 진단 SVG는 커밋하지 않았다.

| 문서 | 확인한 쪽 | Native 최악 | WASM 최악 | gate |
| --- | --- | ---: | ---: | --- |
| `hwpctl_API_v2.4` | 14, 15, 21, 49, 60, 88 | 97.19% | 97.19% | 양쪽 passed |
| `1341000_research_report_footnotes` | 72, 76 | 96.19% | 96.19% | 양쪽 passed |

각 수치는 2px 이웃 관용 내용 실루엣 일치율이다. API 49·60쪽의 표 외곽·뒤 문단과 라틴 72·76쪽의 한글·영문 글리프를 Native/WASM review·overlay PNG에서 직접 확인했다. 대표 이미지 외의 검사 쪽도 `evidence.json`에 지표를 기록했다.

라틴 문서는 설치 글꼴을 공급하지 않은 첫 진단에서 한글이 깨졌으나 자동 게이트는 통과했다. `~/Library/Fonts` 자동 공급 보정 뒤 한글이 정상 표시됐고 Native 최악 96.19%가 됐다. 여기에 실린 증적은 보정 뒤 새로 캡처한 결과다.

`#7393`의 [한컴 2024 원본 PDF](../../../../pdf/issue6907/pr7393-source-2024.pdf)와 [왕복 PDF](../../../../pdf/issue6907/pr7393-roundtrip-2024.pdf)는 3~8쪽을 96 DPI로 래스터화해 여섯 쪽 모두 픽셀 차이가 없음을 확인했다. 이 둘은 증적 자료로 커밋돼 있다.

검증 명령의 공통 형식:

```bash
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt
python3 scripts/visual_sweep.py --file-target <key> <입력> <기준 PDF> \
  --rhwp-bin target/pr-review/release-test/rhwp --pages <영향 쪽> \
  --dpi 96 --embed-fonts=full --out output/<검증명>
# WASM 검증은 같은 명령에 --wasm-pkg pkg를 더하고 별도 output 경로를 쓴다.
```
