# OSS-Fuzz 온보딩 계획 (M10-1)

본 문서는 rhwp 의 기존 `cargo-fuzz` 타깃을 Google [OSS-Fuzz](https://google.github.io/oss-fuzz/) 에
어떻게 매핑할지 적는다. **이 단계는 계획과 메인테이너 승인 요청만** 한다.

- `google/oss-fuzz` 에 프로젝트를 올리는 PR 은 **M10-2** (승인 후).
- 이 저장소에 required CI · nightly 스모크 · gym 을 추가하지 않는다.
- 로컬 하네스 정본은 [`README.md`](README.md) 이다. 본 문서는 등재 설계만 다룬다.

관련: RFC [#3141](https://github.com/edwardkim/rhwp/issues/3141), 계획 이슈 #5421.

## 1. 목적과 비범위

| 한다 | 하지 않는다 |
|---|---|
| 기존 `fuzz/fuzz_targets/*.rs` 를 OSS-Fuzz 바이너리로 1:1 매핑 | 새 하네스 작성 (M03-13 등 후속 PR 의 몫) |
| `projects/rhwp/{project.yaml,Dockerfile,build.sh}` 초안 | `google/oss-fuzz` 에 지금 PR |
| 시드 zip · 트리아지 · 공개 정책 정리 | PR required check / `pull_request` 퍼즈 잡 |
| 메인테이너가 승인할 연락처·범위 항목 | gym · M08 · 왕복 정합성(M04) · DocumentCore 신설 |

검출 대상은 로컬과 같다. 파서 `Err` 는 정상이다.

- 패닉 / abort
- 자원 고갈 (OOM)
- 무한루프 (타임아웃)

## 2. 현재 cargo-fuzz 상태 (`devel`)

`fuzz/` 는 본 크레이트와 분리된 워크스페이스(`fuzz/Cargo.toml` 의 `[workspace]`) 이고
`libfuzzer-sys` + `rhwp` path 의존만 쓴다. 기본 피처만 켜며 `native-skia` 는 끈다.

하네스는 모두 `fuzz_target!(|data: &[u8]| { let _ = …; })` 형태다.

### 2.1 devel 에 있는 타깃 (초기 등재)

| cargo-fuzz 타깃 | 진입점 | 컨테이너 | 시드 (`fuzz/corpus/<타깃>/`) |
|---|---|---|---|
| `parse_hwp` | `rhwp::parser::parse_hwp(&[u8])` | HWP 5.x / CFB | `english.hwp`, `shortcut.hwp`, `Textmail.hwp` |
| `parse_hwp3` | `rhwp::parser::hwp3::parse_hwp3(&[u8])` | HWP 3.x | `hwp3-pagedef-1915.hwp`, `hwp3-sample.hwp` |
| `parse_hwpx` | `rhwp::parser::hwpx::parse_hwpx(&[u8])` | HWPX / ZIP | `neartop_reset_sb2500.hwpx`, `saved_single_line_spacing_after.hwpx`, `tac-host-spacing.hwpx` |
| `parse_hml` | `rhwp::parser::hml::parse_hml(&[u8])` | HML / XML | `exambank_math_equations_min.hml`, `formatting_table.hml` |
| `parse_wmf` | `WMFConverter::new(data, SVGPlayer::new()).run()` | WMF | `minimal_placeable.wmf` |
| `parse_ooxml_chart` | `rhwp::ooxml_chart::parser::parse_chart_xml(&[u8])` (`rhwp_ooxml_chart` 재수출) | OOXML 차트 XML | `bar_chart.xml` |

### 2.2 devel 에 아직 없는 타깃 (M03-13)

`parse_equation` · `export_svg` 는 M03-13 ([#5423](https://github.com/edwardkim/rhwp/pull/5423))
에서 제안 중이며 **현재 `devel` 에는 없다.** 병합되면 아래 규칙으로 자동 포함되게 설계한다.

| cargo-fuzz 타깃 | 진입점 | 비고 |
|---|---|---|
| `parse_equation` | `renderer::equation::parser::parse` + `doclang::eqedit::convert` | 비 UTF-8 은 하네스가 즉시 return |
| `export_svg` | `DocumentCore::from_bytes` → `render_page_svg_native(0)` | 파싱+1페이지 렌더. 메모리·시간 비용이 크다 |

`build.sh` 는 `fuzz/fuzz_targets/*.rs` 를 훑어 복사하므로, 타깃을 이 저장소에만 추가하면
OSS-Fuzz 쪽 파일을 다시 고치지 않아도 된다.

1순위에서 빼는 후보: `export_svg` (렌더 경로, 시드·시간 예산이 큼). 메인테이너가
1차 등재에서 제외할 수 있다. `parse_equation` 은 파서만 치므로 1차에 넣어도 된다.

## 3. OSS-Fuzz 매핑

OSS-Fuzz Rust 는 [cargo-fuzz](https://google.github.io/oss-fuzz/getting-started/new-project-guide/rust-lang/)
가 정본이다. 엔진·새니타이저는 **libFuzzer + AddressSanitizer 만** 지원한다.

| rhwp 쪽 | OSS-Fuzz 쪽 |
|---|---|
| `fuzz/fuzz_targets/<name>.rs` | `$OUT/<name>` 실행 파일 (이름 동일) |
| `fuzz/corpus/<name>/` | `$OUT/<name>_seed_corpus.zip` |
| `cargo +nightly fuzz run <name> -- -rss_limit_mb=2048 -timeout=30` | ClusterFuzz 가 libFuzzer 플래그·RSS·타임아웃을 관리 |
| `fuzz/artifacts/<name>/` + `fuzz/regressions/<name>/` | OSS-Fuzz 트래커 재현 입력 → 로컬 `cargo fuzz run` / `tmin` |
| nightly 스모크(M03-1, 미병합) | 대체하지 않음. 상시 퍼징은 ClusterFuzz |

프로젝트 디렉터리 이름: `projects/rhwp` (`[A-Za-z0-9_-]+` 만 허용).

| 필드 | 제안 값 | 승인 필요 |
|---|---|---|
| `homepage` | `https://github.com/edwardkim/rhwp` | 아니오 |
| `main_repo` | `https://github.com/edwardkim/rhwp` | 아니오 |
| `language` | `rust` | 아니오 |
| `sanitizers` | `[address]` | 아니오 (Rust 제약) |
| `fuzzing_engines` | `[libfuzzer]` | 아니오 (Rust 제약) |
| `primary_contact` | **메인테이너 Google 계정 이메일** | **예** |
| `auto_ccs` | 트리아지 담당자 목록 | **예** |
| `file_github_issue` | 기본 false (SECURITY.md 의 Advisory 경로와 맞춤) | **예** |
| `help_url` | `https://github.com/edwardkim/rhwp/blob/devel/SECURITY.md` (선택) | **예** |

`primary_contact` 은 VCS 에 남은 established committer 의 Google 계정이어야 ClusterFuzz
대시보드에 접근할 수 있다. 후보는 [SECURITY.md](../SECURITY.md) 의
`tangokorea@gmail.com` 이지만, **이 문서에 이메일을 확정하지 않는다.** 승인 이슈에서
메인테이너가 적는다.

수락 요건 ([Accepting new projects](https://google.github.io/oss-fuzz/getting-started/accepting-new-projects/)):

- 오픈소스 라이선스 — MIT. 충족.
- 상당 사용자 또는 인프라 중요도 — HWP/HWPX 파서는 한국 문서 유통의 핵심 경로이고,
  RFC #3141 이 적대적 첨부 위협 모델을 이미 적었다. OSS-Fuzz 리뷰어가 물을 수 있으므로
  승인 이슈에서 “등재 신청을 한다”는 한 줄을 남긴다.

## 4. 제안 프로젝트 파일 (초안, M10-2 제출물)

아래는 `google/oss-fuzz` 의 `projects/rhwp/` 에 넣을 초안이다. **이 저장소에 복사하지 않는다.**

### 4.1 `project.yaml`

```yaml
homepage: "https://github.com/edwardkim/rhwp"
main_repo: "https://github.com/edwardkim/rhwp"
language: rust
primary_contact: "REPLACE_WITH_MAINTAINER_GOOGLE_ACCOUNT"
sanitizers:
  - address
fuzzing_engines:
  - libfuzzer
# auto_ccs:
#   - "..."
# help_url: "https://github.com/edwardkim/rhwp/blob/devel/SECURITY.md"
```

### 4.2 `Dockerfile`

```dockerfile
FROM gcr.io/oss-fuzz-base/base-builder-rust
RUN git clone --depth 1 https://github.com/edwardkim/rhwp.git rhwp
WORKDIR $SRC
COPY build.sh $SRC/
```

기본 피처만 쓰므로 `apt` 시스템 라이브러리(skia, 폰트)는 필요 없다.
`base-builder-rust` 에 nightly 와 `cargo-fuzz` 가 이미 있다.

### 4.3 `build.sh`

```bash
#!/bin/bash -eu
# Copyright 2026 Google LLC
# (OSS-Fuzz 관례 Apache-2.0 헤더. M10-2 제출 시 연도를 맞춘다.)

cd "$SRC/rhwp"

# rust-toolchain.toml 이 1.93.1 을 고정한다. cargo-fuzz 의 -Z 플래그는 nightly 가 필요하다.
export RUSTUP_TOOLCHAIN=nightly

# 기본 피처만. native-skia / subsecond-dev 금지.
cargo fuzz build -O --debug-assertions

FUZZ_TARGET_OUTPUT_DIR=fuzz/target/x86_64-unknown-linux-gnu/release
for f in fuzz/fuzz_targets/*.rs
do
    FUZZ_TARGET_NAME=$(basename "${f%.*}")
    cp "${FUZZ_TARGET_OUTPUT_DIR}/${FUZZ_TARGET_NAME}" "$OUT/"
    CORPUS_DIR="fuzz/corpus/${FUZZ_TARGET_NAME}"
    if [[ -d "${CORPUS_DIR}" ]]; then
        zip -jr "$OUT/${FUZZ_TARGET_NAME}_seed_corpus.zip" "${CORPUS_DIR}"
    fi
done
```

산출 바이너리 이름은 영숫자·`_`·`-` 만 쓴다. 현재 타깃 이름은 이 규칙을 지킨다.

coverage 새니타이저에서 rustc 가 깨지면 rustls 처럼 `RUSTFLAGS` 의 debug-assertions 를
끄거나 무거운 타깃만 빼는 분기를 M10-2 에서 추가한다.

## 5. 빌드 제약

1. **툴체인 핀.** 루트 `rust-toolchain.toml` 은 `channel = "1.93.1"` 이다. 오버라이드
   없이 `cargo fuzz build` 하면 OSS-Fuzz 이미지가 stable 을 받아 `-Z` 가 실패한다.
   `RUSTUP_TOOLCHAIN=nightly` 를 필수로 둔다.
2. **피처.** `native-skia` 는 네이티브 시스템 라이브러리가 필요하다. 등재 빌드는
   default 만 쓴다 (`console_error_panic_hook` 은 native 에서도 컴파일된다).
3. **독립 워크스페이스.** `cargo fuzz` 는 루트에서 실행한다. `fuzz/` 를 루트
   workspace members 에 넣지 않는다 (지금 구조 유지).
4. **디스크.** OSS-Fuzz 빌더 250GB, `$OUT` 는 비압축 10GB 미만. 바이너리 6개 + 소형
   시드 zip 은 여유 있다. 전체 `samples/` 를 시드로 넣지 않는다.
5. **의존성.** 파서 타깃은 순수 Rust(cfb, zip, quick-xml, image 의 bmp/jpeg/png/tiff).
   런타임에 공유 라이브러리를 가정하지 않는다.

로컬 사전 검증 (M10-2, Docker 필요):

```sh
python3 infra/helper.py build_image rhwp
python3 infra/helper.py build_fuzzers --sanitizer address rhwp
python3 infra/helper.py check_build rhwp
python3 infra/helper.py run_fuzzer rhwp parse_hml
```

이 명령을 이 PR 에서 실행하지 않는다.

## 6. 시드 코퍼스

CFB/ZIP 은 시드 없이 변이가 헤더에서 멈춘다. 기존 `fuzz/corpus/<타깃>/` 을
`<타깃>_seed_corpus.zip` 으로 그대로 올린다.

- 파일은 이 저장소 MIT 라이선스 아래의 공개 샘플·합성 최소 시드다.
- `samples/` 전체(수백 MB) 는 올리지 않는다. OSS-Fuzz 가 요구하는 것은 시작 입력이고,
  ClusterFuzz 가 이후 코퍼스를 키운다.
- 시드에 개인정보·비공개 문서·저작권 불명 파일을 넣지 않는다. 지금 6개 코퍼스는
  이 기준을 통과한다.
- M03-13 이 병합되면 `parse_equation` / `export_svg` 코퍼스도 같은 zip 규칙으로 따라간다.

사전 사전은 쓰지 않는다. 컨테이너 포맷은 시드가 우선이고, 필요하면 M10-2 이후
`.dict` 를 별도 제안한다.

## 7. 트리아지와 공개 정책

OSS-Fuzz 는 재현 가능한 크래시를 비공개 트래커에 넣고, 수정이 안 되면 약 90일 후
공개한다. rhwp [SECURITY.md](../SECURITY.md) 는 GitHub Advisory 를 1순위, 이메일을
백업으로 둔다.

권장 흐름:

1. ClusterFuzz 가 이슈를 연다. `primary_contact` / `auto_ccs` 가 CC 된다.
2. 담당자가 입력을 받아 로컬에서
   `cargo +nightly fuzz run <타깃> <재현파일> -- -rss_limit_mb=2048 -timeout=30`
   으로 재현한다.
3. `cargo +nightly fuzz tmin` 으로 줄인 뒤 `fuzz/regressions/<타깃>/` 에 커밋하고
   단위 테스트로 옮긴다 (`README.md` 트리아지 절, #2743 방식).
4. 수정 PR 은 `devel` 기준, 기존 이슈 → PR 규약을 따른다.
5. 같은 결함 클래스가 반복되면 전수 스윕 이슈를 따로 연다 (#3004 → #3012).
6. OSS-Fuzz 이슈를 닫아 공개 타이머를 멈춘다. GitHub Advisory / CVE 가 필요하면
   SECURITY.md 경로를 추가로 탄다.

`file_github_issue: true` 는 공개 이슈로 미러한다. 보안 제보를 Advisory 로 모으려면
기본값(false)을 유지하는 편이 SECURITY.md 와 맞다. 메인테이너가 뒤집으면 따른다.

이 단계(M10-1) 에서 발견된 DoS 를 고치지 않는다.

## 8. 메인테이너 승인 항목

등재 PR(M10-2) 전에 아래를 승인 이슈(`[제안] OSS-Fuzz 등록`)에서 닫는다.

- [ ] OSS-Fuzz 등재 자체를 진행한다 (트리아지·90일 공개를 수용한다).
- [ ] `primary_contact` Google 계정 이메일.
- [ ] `auto_ccs` 목록 (없으면 primary 만).
- [ ] 1차 타깃: devel 6개 전부. M03-13 병합 시 `parse_equation` 포함 여부,
      `export_svg` 1차 제외 여부.
- [ ] `file_github_issue` / `help_url`.
- [ ] 공개 시드 zip 이 현재 `fuzz/corpus/` 인 것.

승인 전에는 `google/oss-fuzz` PR 을 열지 않는다.

## 9. M10-2 체크리스트 (이 PR 의 범위가 아님)

1. 승인 이슈의 연락처를 `project.yaml` 에 기입한다.
2. `google/oss-fuzz` 를 포크해 `projects/rhwp/` 세 파일을 추가한다.
3. `infra/helper.py build_image` / `build_fuzzers` / `check_build` 를 address 로 통과한다.
4. 가능하면 coverage 빌드도 한 번 본다.
5. OSS-Fuzz 쪽 PR 본문에 이 문서와 승인 이슈를 링크한다.
6. 병합 후 배지·ClusterFuzz 접근은 후속 문서로 남긴다.

## 10. 하지 않는 것 (M10-1)

- `google/oss-fuzz` PR
- `.github/workflows/` 퍼즈 잡 · required check
- gym / `scripts/visual_sweep.py`
- M08, M03-13 하네스 본문, 파서 버그 수정
- `src/` · `fuzz/fuzz_targets/` · `fuzz/corpus/` 변경
