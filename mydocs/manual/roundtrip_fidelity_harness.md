# HWPX→HWP roundtrip 페이지네이션 fidelity 하니스

`tools/roundtrip_fidelity_harness.py` + `tools/roundtrip_fidelity_diff.py`.

## 무엇을 측정하나

기존 `hwpx-roundtrip` / `cargo test --test hwpx_roundtrip_baseline` 은 **HWPX→HWPX**
직렬화의 IR 뼈대·패키지 구조 보존을 검사한다. 본 하니스는 이와 **상보적**으로
**HWPX→HWP 변환 후 페이지네이션이 원본 HWPX 와 일치하는지**를 측정한다.

각 HWPX 를 `rhwp convert` 로 HWP 로 변환한 뒤, orig-HWPX 와 변환-HWP 의 `dump-pages`
`(section, pi) → 첫 등장 페이지` 매핑을 대조한다. 불일치는 HWPX↔HWP5 파서/typeset
경로의 divergence 를 의미한다(예: 빈-앵커 host_line_spacing 소스-의존 억제 계열).

`.hwpx` 옆에 native `.hwp` 가 함께 있으면 실제갭 vs phantom갭 두 인코딩을 같은
하니스에서 비교할 수 있다(예: `samples/rowbreak-problem-pages.{hwp,hwpx}`).

## 판정

| verdict | 의미 |
|---|---|
| SAME | orig-HWPX 와 변환-HWP 의 페이지 수·(sec,pi)→page 완전 일치 |
| PI_MOVED | 페이지 수 동일하나 일부 (sec,pi) 가 다른 페이지 |
| PAGE_DELTA | 총 페이지 수가 다름 |
| ERR | convert/dump 실패 |

종료 코드: `PI_MOVED + PAGE_DELTA > 0` 이면 1.

## 사용

전제: `cargo build --release` (`target/release/rhwp[.exe]` 사용).

```bash
# 폴더 전수(재귀)
python tools/roundtrip_fidelity_harness.py --corpus samples/hwpx --workdir /tmp/wd -o out.tsv

# 파일 목록(재현용 코퍼스; 줄당 HWPX 경로, # 주석 허용)
python tools/roundtrip_fidelity_harness.py --file-list corpus.txt --workdir /tmp/wd -o out.tsv

# 개별 파일
python tools/roundtrip_fidelity_harness.py --files a.hwpx b.hwpx --workdir /tmp/wd -o out.tsv
```

산출 TSV 컬럼: `sample / verdict / hwpx_pages / hwp_pages / n_moved / detail`.

## 두 바이너리 비교 (회귀/개선 정량화)

기준 바이너리와 후보 수정 바이너리로 **동일 코퍼스**를 각각 측정한 뒤 전이를 분류한다:

```bash
# 기준 바이너리로 측정
python tools/roundtrip_fidelity_harness.py --file-list corpus.txt --workdir /tmp/base -o base.tsv
# (수정 후 재빌드) 후보 바이너리로 측정
python tools/roundtrip_fidelity_harness.py --file-list corpus.txt --workdir /tmp/new  -o new.tsv
# 전이 분류
python tools/roundtrip_fidelity_diff.py --base base.tsv --new new.tsv -o diff.tsv
```

`diff.tsv` 분류: `IMPROVED`(회귀 해소) / `REGRESSED`(신규 회귀) / `STILL_MOVED` /
`STILL_SAME`. `roundtrip_fidelity_diff.py` 는 `REGRESSED` 가 있으면 종료 코드 1.

> 주의: 본 하니스는 **HWPX↔변환HWP 자기정합**(proxy)을 측정하며, 한컴 편집기/PDF 대비
> 시각 정합(oracle)이 아니다. 시각 판정은 `pdf/`·한컴 편집기 정답지를 사용한다.
