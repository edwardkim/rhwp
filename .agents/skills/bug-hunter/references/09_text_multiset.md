# 09 — 쪽별 문자 멀티셋

playbook: 기준 PDF 텍스트층 ↔ SVG `<text>` 의 쪽별 문자 멀티셋.
공백·순서·NFC 를 정규화한다.

| 관측 | 분류 | 영어 픽스처 |
| --- | --- | --- |
| 기준본 전용 문자 | 소실 | `reference_only` → loss |
| SVG 전용 문자 | 과잉 | `svg_only` → excess |
| 같은 쪽의 양쪽 차이 | 치환 후보 | both → substitution |

폰트 대체가 픽셀을 흔들어도 쪽번호·채움점 소실, 숨김 대상의 과잉
출력, PUA 치환은 문자 수 차이로 드러날 수 있다.

## 단독 최종 판정이 아니다

PDF 가 글자를 path 로 그렸거나 텍스트층 매핑이 손상되면 거짓
소실이 잡힌다. F06·F07·F08. 사람 감사와 픽셀 시트를 붙이기 전에
이슈를 확정하지 않는다.

순서와 공백을 무시하므로 배치·줄바꿈은 이 축으로 못 본다. 그 축은
픽셀/시각이다.

## 읽는 법

```bash
sort -t $'\t' -k2,2nr -k3,3nr /tmp/rhwp-fidelity-plan/text-report.tsv | head
```

- `reference_only` 큰 쪽 → 소실 후보 큐
- `svg_only` 큰 쪽 → 과잉 후보 큐
- 같은 쪽 둘 다 크면 → 치환 후보 (PUA↔U+FFFD 등)

픽스처 봉투:

- `fixtures/envelopes/text_report_loss.json`
- `fixtures/envelopes/text_report_excess.json`
- `fixtures/envelopes/text_report_substitution.json`
- `fixtures/tsv/text_report_sample.tsv`

## 보조 원장 (fidelity_compare)

도구가 추가로 남기는 TSV 도 후보다. 결함 확정이 아니다.

- `svg-glyph-risk-report.tsv` — raw PUA / U+FFFD
- `text-owner-shift-candidates.tsv` — 쪽 owner 이동
- `visible-text-excess-candidates.tsv` — clip 안 과잉
- `page-boundary-fidelity-candidates.tsv` — 경계 큐
- `--layout-ledger` 의 `square_wrap_text_overlap`

## 분류 계약

`fixtures/classification.json`:

```
missing = loss
extra   = excess
both    = substitution
```

이 세 단어를 스킬 안에서 다른 뜻으로 쓰지 않는다.
