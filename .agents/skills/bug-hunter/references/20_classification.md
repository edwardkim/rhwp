# 20 — 비교 분류표

playbook 문자 멀티셋 분류를 표로 고정한다. 단어를 바꾸지 않는다.

`fixtures/classification.json`:

```
missing = loss     # 소실
extra   = excess   # 과잉
both    = substitution  # 치환
```

TSV: `fixtures/tsv/classification.tsv`

## 표

| ID | 관측 | 한국어 | 영어 | 최종? | 이슈화? |
| --- | --- | --- | --- | --- | --- |
| C01 | reference_only | 소실 | loss | 아니오 | 아니오 |
| C02 | svg_only | 과잉 | excess | 아니오 | 아니오 |
| C03 | both_delta_same_page | 치환 | substitution | 아니오 | 아니오 |
| C04 | reread_mismatch | 기록값 불일치 | reread | 예 | 예 |
| C05 | exit_or_json_contract | 계약 위반 | contract | 예 | 예 |
| C06 | pixel_diff_rank | 픽셀 후보 | pixel-candidate | 아니오 | 아니오 |
| C07 | zip_name_set_missing | 엔트리 소실 | zip-loss | 아니오 | 아니오 |
| C08 | constant_byte_shrink | 상수 블록 신호 | constant-shrink | 아니오 | 아니오 |
| C09 | self_render_diff_only | 자기 일관성 | self-consistency | 아니오 | 아니오 |
| C10 | console_mojibake | 콘솔 착시 | not-a-defect | 예(결함아님) | 아니오 |
| C11 | pdf_path_only_glyphs | 텍스트층 손상 | pdf-path-text | 아니오 | 아니오 |
| C12 | layout_ledger_square_wrap | 그림 침범 | square-wrap | 아니오 | 아니오 |

C07/C08 은 태그 개수·사람 확인을 붙이면 이슈화할 수 있다. 표의
`issueReady=false` 는 "관측만으로 확정하지 마라"는 뜻이다.

## 검출 ≠ 판정

samples/ HWPX 전수에서 `--verify` 100% 와 구조 무손실 5.4% 가
동시에 참이다. fwSpace/nbSpace/`hp:t` 다수는 정규화였다. 검출
비율을 손실 비율로 복사하지 않는다 (P16).
