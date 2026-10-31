# PR #8144 시각 증거

2026-10-30, 코드 `21ba44e3ea75966321d7184e2d94a8620e378013`, base `168aaaf2bf5a4ef5ecc70a61087aeabf2d9d2433`.
이후 커밋은 이 코드의 테스트·문서·증적을 보존한다. [전체 결과](results-table.md)와 각 하위 폴더의 `silhouette.tsv`는 공식 Visual Sweep 원문이다. 모든 값은 96 DPI, 색상 경계 보정 전 2px `silhouette_raw_match_percent`다. `nine-empty-break`는 2쪽, 나머지 원 사례·경계·편집 사례는 1쪽이며 독립 Print와 일치한다.

- `base/`, `candidate/`는 동일 입력·기준·글꼴 환경의 변경 전후다.
- `native-verified/`, `wasm-verified/`는 아래 명시적 글꼴 공급이다. `*-fresh`는 같은 원본을 현재 한컴 2024에서 새로 Print한 PDF다. 원 PR의 PDF 비교도 별도로 유지했다.
- `*-controls/`는 기존 환경(KoPub, Nanum, Windows Fonts, HFT 변환 글꼴)에서 #7699, #7548, #8145, #8165의 base/후보를 대조한다. cellzone은 각주 색 없음 보정으로 Native 99.54886→99.73113%, WASM 99.55637→99.73834%로 개선됐고 나머지는 각 backend의 base와 동일하다. #7699 세 문서는 각 3쪽, 나머지는 각 1쪽이다.
- `*-edit-session/`는 실제 `insert_text_native`/WASM `insertText` 직후 화면이다. `*-edited-reopen`은 그 세션에서 저장한 동일 HWPX를 다시 연 결과다. 양쪽 backend에서 세션 PNG와 재열기 PNG가 바이트까지 같다. Native/WASM 저장 HWPX도 바이트가 같고, undo PNG는 각 편집 전과 같다.
- `candidate/native/`, `candidate/wasm/`에는 각 경계의 공식 compare·standalone overlay·review PNG가 있다. base의 원본·정상 대조군 및 후보의 모든 TSV를 함께 보존한다.
- 코드·Native 실행 파일·최적화 WASM·JS 해시, 한컴 작업 ID·버전·Print 계약·PDF 해시·글꼴 해시는 [provenance.json](provenance.json)에 있다. 입력 생성 방식은 [fixture README](../../../../tests/fixtures/pr8144/README.md)에 기록했다.

## 글꼴 공급

현재 한컴 PDF에서 실제 포함된 Haansoft Batang·KoPubBatangLight·KoPubDotumLight를 확인했다. [font-environment.json](font-environment.json)은 나눔스퀘어 계열을 이 환경의 Haansoft Batang으로 명시한다. `HBATANG.TTF` 파일을 먼저, KoPub 디렉터리를 다음으로 공급했다. 폰트 프로그램은 로컬 검증 전용이며 커밋하지 않았다.

전체 HWP 글꼴 디렉터리를 먼저 열면 Native의 기존 총 64MiB 등록 예산 때문에 필요한 HBATANG이 뒤에서 제외될 수 있다. 최종 검증은 약 31MiB HBATANG 파일 + 약 28MiB KoPub로 제한해 두 backend에 실제 같은 프로그램이 들어가게 했다. 이 작업에서 폰트 예산이나 fallback 코드를 변경하지 않았다. 빈 contour인 나눔스퀘어 U+2022를 공급한 초기 진단은 최종 증거로 쓰지 않았다.

원 PR 제목 PDF는 고딕 제목 글꼴, 현재 Print는 바탕 대체 글꼴이다. 원 PDF에서도 전쪽 94% 이상이며 같은 원본의 현재 Print는 98% 이상이다. 표 외곽·본문 시작·각주 위치는 일치한다. 원 PDF를 지우거나 새 PDF로 점수를 대체하지 않았다. 글꼴 예외로 gate를 면제하지 않았다.

## 재현

```bash
export CARGO_TARGET_DIR="$PWD/target/pr-review"
CARGO_BUILD_JOBS=6 CARGO_PROFILE_RELEASE_TEST_OPT_LEVEL=1 \
CARGO_PROFILE_RELEASE_TEST_CODEGEN_UNITS=64 \
cargo build --locked --profile release-test --features native-skia --bin rhwp
scripts/wasm-pack-locked.sh --target web --out-dir pkg

venv/bin/python scripts/visual_sweep.py --silhouette-only --compat 2024 --dpi 96 \
  --rhwp-bin target/pr-review/release-test/rhwp \
  --font-path /home/edward/mygithub/ttfs/hwp/HBATANG.TTF \
  --font-path /home/edward/mygithub/ttfs/kopub \
  --font-environment mydocs/pr/assets/pr_8144/font-environment.json \
  --file-target nine-empty tests/fixtures/pr8144/nine-leading-empty.hwp \
  pdf/title_page_start/maintainer/nine-leading-empty-2024.pdf \
  --out output/pr-review/pr8144-reproduce/native
```

WASM은 같은 명령에 `--wasm-pkg pkg`를 더하고 별도 `--out`을 쓴다. 실제 Chrome Canvas PNG 경로로 실행하며 이 검증은 Chrome 155.0.8059.79를 사용했다. `--silhouette-only`를 빼면 compare·overlay·review를 만든다. PDF는 PDFium으로 rasterize했다.

편집 동작은 `title-table-after-empty-paragraph.hwpx`의 후속 표 host pi4 또는 선행 빈 문단 pi0, `paper-square-table-tac-host.hwpx`의 pi0에 offset 0으로 `검증 `을 삽입한다. Print profile, 2024 compatibility, 같은 글꼴 환경을 공급한다. 스냅샷 → 삽입 → PNG·HWPX 저장 → 스냅샷 복원 PNG를 대조했다. 독립 PDF는 정확히 그 저장 바이트를 한컴 2024 `PrintMethod=0`, `frame_print_to_pdf_ex_one_up`으로 인쇄했다. HWPX export 결과는 정식 fixture에 보존했다.
