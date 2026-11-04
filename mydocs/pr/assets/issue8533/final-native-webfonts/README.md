# Native 공개 글꼴 공급·편람 품질 검증 증적

Issue: #8533. 검증 제품 소스는 `c031a38a41670f5e4f6ab0268ef46c62ab1445f5`다. 이후 Native·WASM 모듈과 Studio 제품 소스는 같지만, 검증 호스트는 명시 프로그램 선행 준비와 PostScript 별칭·legacy family 선택을 보정했다. 이 묶음의 기존 WASM 출력은 그 보정 전 증거이며, [3단계 기록](../../../../working/task_m100_8533_stage3.md)의 같은 현재 호스트 base/후보 재검증 결과로 갱신해야 한다. 이 묶음은 완료한 출력의 원본을 보존하며, 갱신과 남은 검사 완료 전에는 PR 준비 완료를 주장하지 않는다.

- [원본·독립 Print PDF](input-index.md)
- [전쪽 TSV 원본과 정밀 raw 값](tsv-index.md)
- [현재 호스트의 원 PR 36문서 base/후보 대조](wasm-corpus-base-candidate.tsv) · [생산 신원·범위](wasm-corpus-base-candidate.json)
- [대표 raw·review·overlay](png-index.md)
- [실제 소비 경로·프로그램 공급·응답 해시](consumer-evidence.md)
- [검증 실행·Rust 검사 소스 동일성](gate-results.md)
- [파일 SHA-256](sha256.md)

## 생산 신원

| 대상 | SHA-256 |
| --- | --- |
| Native Skia 실행 파일 | `c2a5651cf29dc68f7b60d20395ca06f6b6d4eccf020db802d3188a3f1d4a280c` |
| fresh WASM | `41f27298cf2b025bf5cebba6ea4cd96f539ceb3a97846368ca37df47d3ab9424` |
| root pkg·Studio public JS | `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393` |

Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 빌드다. Docker 최적화 통과로 기록하지 않는다. 비교 base는 `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4`이고, 제품 코드가 같은 #8517 병합 커밋 `c97709b59742372a817b3886f442144728fca646`의 고정 producer를 사용한다.

## 완료한 Native 전쪽 대조

| 글꼴 환경 | 쪽수 | base 최저 raw % | 후보 최저 raw % | raw 개선 쪽 | 0.1%p 초과 하락 |
| --- | ---: | ---: | ---: | ---: | ---: |
| normal | 383 | 94.26497 | 99.96463 | 35 | 0 |
| isolated | 383 | 93.13408 | 99.96465 | 128 | 0 |

두 환경 모두 후보 382쪽의 표시값은 100%, 130쪽은 99.96%다. raw 정밀값이100%인 쪽과 소수 둘째 자리 표시값 100%를 구분한다. 개인 글꼴 접근 차단은 macOS의 CoreText 이름 캐시까지 제거한 VM이 아니며, 일반 Native의 실제 공개 프로그램 공급을 검증한 조건이다.

원 PR #8438 10문서·78쪽과 #8517 26문서·1,123쪽의 Native PNG는 각각 같은 환경의 원 PR 재출력과 전쪽 해시가 동일하다. #8438의 10문서는 최신 base 동등 producer로도 재출력해 78쪽의 동일성을 확인했다.

## 재현 명령

저장소 루트에서 빌드·캡처는 각각 순차 실행한다. Cargo jobs8, 테스트 threads8, 전체 캡처 workers8을 유지한다.

```bash
CARGO_TARGET_DIR=target/pr-review CARGO_BUILD_JOBS=8 cargo build --locked --profile release-test --features native-skia --bin rhwp
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt
venv/bin/python scripts/visual_sweep.py --handbook-gate --rhwp-bin target/pr-review/release-test/rhwp --page-workers 8 --out output/handbook-required
venv/bin/python scripts/visual_sweep.py --silhouette-only --rasterizer png --wasm-pkg pkg --rhwp-bin target/pr-review/release-test/rhwp --dpi 96 --compat 2024 --page-workers 8 --file-target handbook-hwpx "samples/2025 행정업무운영 편람(최종).hwpx" "pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf" --out output/handbook-wasm
```

보존 코퍼스는 input-index의 같은 호환 세대별로 `--file-target <키> <입력> <PDF>`를 반복 지정한다. 암호 원본은 fixture의 123456을 stdin으로 전달해 열며, 복호화한 다른 파일로 치환하지 않는다. 측정 조건은 PDFium·96 DPI·Print·해당 2022/2024 호환·2px·원본 색상 문턱 32를 유지했다. 평균값으로 미달 쪽을 상쇄하지 않는다.
