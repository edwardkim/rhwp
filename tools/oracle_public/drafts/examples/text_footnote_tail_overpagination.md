---
title: "[오라클] text_footnote_tail_overpagination: pdf_pages 242 (>= 12)"
labels: bug
draft: true
submit: never
---

# [오라클] text_footnote_tail_overpagination: pdf_pages 242 (>= 12)

> **초안 — 제출하지 않음.** 이 파일은 디스크에만 남긴다. `gh issue create` 로
> 올리지 말고, 사람이 수치·재현 커맨드를 확인한 뒤 수동으로 등록한다.

## 현상

문서 `samples/task1725/text_footnote_tail_overpagination.hwpx` 가 한컴 기준 대비 임계 게이트를 넘었습니다.

- 지표: `pdf_pages` = **242**
- 게이트: `pdf_pages >= 12` 이면 실패
- 쪽수: 242
- 최악 페이지: 1

cheap 스윕 판정 `CHEAP_OK` · 한컴 2020 · 군 `footnote` · 쪽수 엔진/visual_sweep 미사용.


## 재현 방법

작업 디렉터리: `.`

```
python tools/oracle_public/page_smoke.py --pair samples/task1725/text_footnote_tail_overpagination.hwpx pdf/task1725/text_footnote_tail_overpagination-hwpx-2020.pdf
```

위 명령을 그대로 다시 돌리면 같은 수치가 나와야 합니다. `scripts/visual_sweep.py`
자체를 수정하지 않습니다.

## 기대 결과

한컴 기준 PDF `pdf/task1725/text_footnote_tail_overpagination-hwpx-2020.pdf` 와 같은 쪽수·레이아웃이어야 합니다.
`pdf_pages` 이 게이트(`>= 12`)에 걸리지 않아야 합니다.

## 실제 결과

| 지표 | 값 |
| --- | --- |
| `pdf_pages` | 242 |
| `pdf_bytes` | 1388624 |
| `sample_bytes` | 784704 |
| `worst_pages` | 1 |

## 환경

- 리포트 출처: `oracle_public.fatten_catalog.cheap`
- 리포트 시각: 2026-09-03T13:33:12Z
- rhwp 바이너리: `(unused)`
- DPI: 96
- pixel_diff_threshold: 12
- 문서 id: `text_footnote_tail_overpagination`

## 제출

제출은 **수동**입니다. 이 생성기는 GitHub 이슈를 만들지 않습니다.
