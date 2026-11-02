# #8440 시각 증거 — 자리차지 표 문단 앞 문단 편집 뒤 vpos 보존

## 입력과 기준 PDF

| 파일 | 출처 | SHA-256 |
|---|---|---|
| 원본 `samples/issue6123/3112461_railway_emc_criteria.hwpx` | 저장소 표본 | `23d545bb87dd23ea755de79341a7b2fab57088493d84fbc691172dbcba1927de` |
| 편집본 `emc_edit.hwpx` (커밋하지 않음, 아래 명령으로 재생성) | 수정 후 rhwp | `8009a8f11dd1f53545ed457fdee47326310caf26c56f2eda9bde7d226d7ab9e9` |
| 수정 전 편집본 `emc_edit_base.hwpx` (커밋하지 않음) | `upstream/devel` `1a7ae8f5b` rhwp | `cd24cd38c78b6d76c3bafb4cc96144bb89a57ef4b130706aaed9806e6aa69d1c` |
| `pdf/issue-8440/emc_edit-2024.pdf` | 편집본을 한컴오피스 2024 Print(`frame_print_to_pdf_ex_one_up`)로 출력, 14쪽 | `054f6a2be2dd1b08cb6ac98c72ba0563391c41d1f7b312f91b132a3fbfa17968` |

```bash
rhwp edit delete-text samples/issue6123/3112461_railway_emc_criteria.hwpx \
  --section 0 --para 114 --offset 0 --count 1 -o emc_edit.hwpx
```

같은 명령을 두 번 실행해도 SHA-256 이 같다(결정적 출력).
편집은 구역 0 문단 114 `"  2) 신호 및 통신, 프로세스 측정 및 제어 포트"`의 첫 글자(공백) 1자 삭제다.

수정 전 편집본도 한컴 2024로 출력해 보았다. 한컴은 저장 줄 vpos 를 따르지 않고 다시 조판하므로
수정 전·후 편집본의 PDF 텍스트 배치가 같다(`pdftotext -layout` 동일, 둘 다 14쪽). 즉 수정 전 편집본에서
표가 겹치는 것은 rhwp 가 저장한 vpos 를 그대로 쓰는 경로에서만 생기며, 기준 PDF 는 편집본 하나로 충분하다.

## 측정

`scripts/visual_sweep.py --silhouette-only --compat 2024 --dpi 96` (pdftoppm), raw = `silhouette_raw_match_percent`,
실루엣 방법 `threshold_boundary_color_support_v1`.
rhwp 글꼴은 `RHWP_FONT_PATH=ttfs/opensource`(문서 글꼴 굴림·바탕·바탕체·한양신명조 미설치, 대체).
Native는 `--features native-skia` release 바이너리, fresh WASM은 같은 소스에서
`scripts/wasm-pack-locked.sh --target web --out-dir pkg`로 새로 만든 package다.

| 측정 | 소스 (base) | `pkg/rhwp_bg.wasm` SHA-256 |
|---|---|---|
| 최초 | `bef747b59` (`1a7ae8f5b`) | `bf0f9ad6edc318c518906931408cd151234a16da57db44cc3681458e7bc92f2a` |
| rebase 뒤 재측정 | `62de2294a` (`c88657d9c`) | `c542999ec3b80e028be67bb36f9dc5628e6accc81fd02f05667e626f0573cedf` |
| 점유 보존 한정 뒤 재측정 | `85dbf88c5` (`c88657d9c`) | `a998acb1b9115611a818f352b90ec319b90933bd15fae21584978c6935e33591` |

rebase 뒤와 점유 보존 한정(`85dbf88c5`) 뒤에 새로 빌드한 Native 바이너리로 편집본을 다시 만들면 SHA-256 이 같고(`8009a8f1…`),
전쪽 Native·fresh WASM TSV 는 최초 측정과 바이트 단위로 같으며, 7쪽 review·overlay PNG 도
바이트 단위로 같다. 세 측정 모두 같으며, 아래 수치와 `tsv/`·`png/` 는 세 측정에 공통이다.

수정 전 열은 수정 전 편집본(`emc_edit_base.hwpx`)을 같은 기준 PDF 에 대 측정했다. 이번 수정은 편집 경로의
vpos 재계산만 바꾸고 읽기·렌더 경로는 바꾸지 않으므로, 저장된 수정 전 편집본은 어느 빌드로 그려도 같다.

| 쪽 | 수정 전 Native | 수정 후 Native | 수정 후 fresh WASM |
|---|---|---|---|
| 1 | 99.95% | 99.95% | 99.96% |
| 2 | 99.93% | 99.93% | 99.87% |
| 3 | 99.98% | 99.98% | 99.99% |
| 4 | 99.96% | 99.96% | 99.97% |
| 5 | 99.99% | 99.99% | 99.85% |
| 6 | 100.00% | 100.00% | 98.79% |
| **7** | **60.76%** | **99.25%** | **99.08%** |
| 8 | 99.99% | 99.99% | 99.99% |
| 9 | 99.99% | 99.99% | 99.96% |
| 10 | 99.13% | 99.13% | 98.43% |
| 11 | 98.86% | 98.86% | 98.65% |
| 12 | 99.86% | 99.86% | 99.72% |
| 13 | 99.70% | 99.70% | 99.39% |
| 14 | 99.65% | 99.65% | 99.30% |

- 수정 후 전쪽 최저: Native 98.86%(11쪽), fresh WASM 98.43%(10쪽). 90% 미만 0쪽.
- 수정 전 편집본은 7쪽 60.76%로 `pr_review_gate` = `re_review_required`.
- 전체 쪽수: 한컴 14쪽 = rhwp 14쪽(수정 전·후 모두).
- 7쪽 전체 모드(`--file-target … --pages 7`) `pr_review_gate`: 수정 후 Native·fresh WASM 모두 `passed`.
- 7쪽을 뺀 13쪽은 수정 전·후 Native 값이 같다.
- TSV 원본: `tsv/native/`, `tsv/wasm/`, `tsv/base_native/`.

## 직접 판독 (7쪽, 96dpi)

| PNG | 판독 |
|---|---|
| `png/emc_edit-p007-base-native-review.png` | 수정 전: "2) 신호 및 통신 …" 아래 표(pi=115) 위에 "3) 함체 포트" 제목과 다음 표(pi=118)가 겹쳐 그려진다 |
| `png/emc_edit-p007-{native,wasm}-review.png` | 수정 후: 편집한 제목, 표 115 외곽선, 빈 문단, "3) 함체 포트", 표 118 외곽선이 한컴 PDF 와 같은 자리에 있다 |
| `png/emc_edit-p007-{native,wasm}-overlay.png` | 수정 후: 표 괘선과 제목 줄이 한컴과 겹친다. 남은 차이는 대체 글꼴의 글자 폭(Ω·µ 등 기호, 셀 안 글자)뿐이다 |

## 경로와 값 (구현 주장 ↔ 증거)

`apply_body_text_edit` → `reflow_paragraph(114)` → `recalculate_section_vpos(start_para=114, start_stored_end=Some(24162))`:

| 문단 | 저장 first | 수정 전 재계산 | 수정 후 재계산 |
|---|---|---|---|
| 115 (자리차지 표 호스트) | 39805 | 24162 (= 114 end + 스타일 gap 0) | 39805 (점유 15643 보존) |
| 116 | 41845 | 26202 | 41845 |
| 117 | 43885 | 28242 | 43885 |
| 118 (자리차지 표 호스트) | 45925 | 30282 | 45925 |

점유 = 다음 문단 저장 first − 편집 전 저장 end − 스타일 gap = 39805 − 24162 − 0 = 15643 HU.
줄 수가 늘어나는 편집에서는 뒤 문단이 늘어난 높이만큼만 움직이고 이 점유는 그대로다
(`growing_edit_moves_following_paragraphs_by_growth_and_keeps_table_gap`).
