# M-fid: fidelity_compare 텍스트층·픽스처 고도화

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5467
브랜치: `feat/m-fid-fatten` (`upstream/devel` 기준 격리 worktree)
범위: `tools/fidelity_compare/` 만
비범위: `scripts/visual_sweep.py` · canvaskit_policy · serializer ·
layout-anomaly · render_backend · proptest · gym

## 무엇을

한컴 기준 PDF 텍스트층과 rhwp SVG `<text>` 를 쪽별로 대조하는
`text-report.tsv` 후보를 **소실 / 과잉 / 치환 / 일치** 로 분류하는
픽스처와 표를 닫는다. `--text-only` 경로의 산출 계약과 parse 오류도
같은 폴더에 고정한다.

- 텍스트층 케이스 244건 (loss 26, excess 92, substitution 36, match 90)
- `--text-only` 경로 32건
- SVG clip/PUA 픽스처 4건

## 왜

픽셀 diff% 는 자간 잡음에 민감하다. 문자 멀티셋은 폰트 대체와 무관한
소실·과잉·치환 후보를 먼저 고른다. 이 루프가 #3385 PUA 원문자 tofu 를
찾았다. 분류는 후보이지 판결이 아니다.

## 어떻게

1. `classify_text_layer_delta` / `text_layer_row` / `write_text_report` /
   `text_only_artifact_names` 를 하네스에 명시한다.
2. `fatten_text_layer.py` 가 등록 키·이슈 문장·NFC/전각/PUA/URL/각주
   변이를 라이브 함수로 재분류한다.
3. `tables/loss.tsv` `excess.tsv` `substitution.tsv` `match.tsv` 와
   owner-shift · glyph-risk · visible-excess 표를 방출한다.
4. `fixtures/text_only_paths/` 가 등록 키 4깃발 × 6키 + direct + 오류 경로를 고정한다.
5. Chrome 없이 `test_fatten_text_layer.py` 가 라이브 함수와 픽스처를 대조한다.

## 분류 규칙

| 분류 | 조건 | 원장 |
| --- | --- | --- |
| match | reference_only=0 이고 svg_only=0 | text-report |
| loss | reference_only>0 이고 svg_only=0 | text-report |
| excess | svg_only>0 이고 reference_only=0 | text-report |
| substitution | 둘 다 >0 | text-report |
| owner-shift | 인접 쪽 75% 상호 일치, 8자+ | text-owner-shift-candidates |
| sequence | 16자+ 순서 보존 이동 | text-owner-sequence-candidates |
| visible-excess | 가시 과잉 48자+, 소실 작음 | visible-text-excess-candidates |
| glyph-risk | raw PUA 또는 U+FFFD | svg-glyph-risk-report |

공백은 `str.isspace` 로 제거한다. 한글은 NFC. 순서는 Counter 에서 무시하고
sequence 원장에서만 본다.

## --text-only

- pypdf 필요, Chrome·pypdfium2 불필요
- `report.tsv` 의 diff% 는 `not-run`
- `--export-all-svg` 는 SVG cache 한 번
- `--layout-ledger` 는 render-tree 후보 원장
- 픽셀 시트 `cmp-pNNN.png` 를 만들지 않는다

## 하지 않은 것

- `scripts/visual_sweep.py` 미수정
- 렌더러·serializer·canvaskit_policy 미수정
- gym pack / 채점기 없음
- 암호화 PDF 우회 없음

## 검증

```bash
python tools/fidelity_compare/test_fidelity_compare.py
python tools/fidelity_compare/test_fatten_text_layer.py
python tools/fidelity_compare/fatten_text_layer.py
cargo fmt --all -- --check
```
