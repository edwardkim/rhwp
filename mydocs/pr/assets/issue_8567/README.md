# #8567 검증 증거

Square 어울림 표가 뒤 문단들을 표 옆 띠로 통째로 흡수했을 때 host 뒤 사다리 일치(#6900)를
흐름상 다음 문단(마지막 흡수 문단 뒤)으로 판정해 host 줄간격 이중 가산을 없앤 수정의 증거다.

## 측정 조건

- 측정: 2026-11-04 14:30+09:00(rebase 후 재측정), `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`,
  열 `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣, `threshold_boundary_color_support_v1`,
  color tolerance 32).
- head: `task/8567-square-wrap-successor-ladder`(base stream/devel 1924d371d6 + 이 수정의 `src/`, 0a63dba9bb).
- base: stream/devel 1924d371d6.
- Native: `cargo build --release --features native-skia --bin rhwp`
  - head 바이너리 sha256 `ed1761354bfc456d…`, base `2d5d84f87c0695a7…`
- fresh WASM: `scripts/wasm-pack-locked.sh --target web --out-dir pkg`, `--rasterizer svg --embed-fonts=full`
  - head `rhwp_bg.wasm` sha256 `8f31107ae47a29fc…`, base `63466acb3d7a2987…`
- 글꼴: `RHWP_FONT_PATH=<문서별 face 디렉터리>:ttfs/hwp:ttfs/windows`.

## 문서

| 키 | 원본 | 기준 PDF | 한컴 쪽수 |
|---|---|---|---|
| i447 | `samples/issue8567/20572683-busan-ordinance-reconsideration-form.hwp`(10k 코퍼스 `ordin_downloads/부산광역시교육청/20572683_[별지  제11호서식]부산광역시 조례안 재의요구안(…).hwp`, docs#447, sha256 `cdb457d4f439973a…`) | `pdf/issue8567/20572683-busan-ordinance-reconsideration-form-2022.pdf`(Creator `Hwp 2022 12.0.0.4547`, Producer `Hancom PDF 1.3.0.547`, sha256 `124e55c14eb0e4e3…`) | 1 |
| i129 | 10k 코퍼스 `korea_downloads/금융위원회/156607622_231226(보도참고)신임 금융감독원 부원장 임명.hwp`(sha256 `b1da87204b1dcbbf…`, 저장소 미포함 — 881/10k A/B 에서 바뀐 유일한 다른 문서) | 공유 오라클 `oracle-samples/i129/hancom_print_2020.pdf`(Creator `Hwp 2022 12.0.0.4547`, sha256 `28c9c73fec3744bb…`) | 2 |

## 결과(쪽별 raw)

| 문서 | 쪽 | head Native | head WASM | base Native | base WASM |
|---|---|---|---|---|---|
| i447 | 1 | 99.99 | 99.29 | 54.06 | 54.54 |
| i129 | 1 | 100.00 | 100.00 | 100.00 | 100.00 |
| i129 | 2 | 97.17 | 97.42 | 28.12 | 26.66 |

- i447 1쪽: base 는 표 옆 띠(문단 0.5~0.7) 뒤 문단 0.8 이하 전체가 8.7 px(host 줄간격 652 HU) 아래.
  head 는 문단 0.10 「이 유」·0.14/0.17 목록·0.25 「※ 조례안 첨부」의 기준선이 한컴 Print 와 겹친다.
- i129 2쪽: 사진(어울림 그림) 옆 「성 명」 문단 뒤 「생년월일」 이하가 base 에서 같은 원인으로 내려갔다.
  head 는 줄 기준선이 한컴과 겹친다. 남은 차이는 한자·괄호 글리프 획과 사진 윗변 약 2~3 px.
- A/B: 881건 공유 코퍼스와 10k 코퍼스 전수 SVG 비교에서 바뀐 문서는 위 두 건뿐이다.
- 쪽수: 두 문서 모두 한컴과 같다.

## 파일

- `{head,base}/{native,wasm}/{i447,i129}_silhouette.tsv`: sweep TSV 원본(바이트 그대로).
- `png/i447_p1_head_native_review.png`, `png/i447_p1_head_native_overlay.png`, `png/i447_p1_head_wasm_review.png`: head 1쪽.
- `png/i447_p1_base_native_review.png`: base 1쪽(문단 0.8 이하 8.7 px 아래).
