---
title: "{{title}}"
labels: bug
draft: true
submit: never
---

# {{title}}

> **초안 — 제출하지 않음.** 이 파일은 디스크에만 남긴다. `gh issue create` 로
> 올리지 말고, 사람이 수치·재현 커맨드를 확인한 뒤 수동으로 등록한다.

## 현상

문서 `{{hwp}}` 가 한컴 기준 대비 임계 게이트를 넘었습니다.

- 지표: `{{metric}}` = **{{metric_value}}**
- 게이트: `{{metric}} {{threshold_op}} {{threshold_value}}` 이면 실패
- 쪽수: {{pages}}
- 최악 페이지: {{worst_pages}}

{{notes_block}}

## 재현 방법

작업 디렉터리: `{{repro_cwd}}`

```
{{repro_command}}
```

위 명령을 그대로 다시 돌리면 같은 수치가 나와야 합니다. `scripts/visual_sweep.py`
자체를 수정하지 않습니다.

## 기대 결과

한컴 기준 PDF `{{pdf}}` 와 같은 쪽수·레이아웃이어야 합니다.
`{{metric}}` 이 게이트(`{{threshold_op}} {{threshold_value}}`)에 걸리지 않아야 합니다.

## 실제 결과

{{metrics_table}}

## 환경

- 리포트 출처: `{{source}}`
- 리포트 시각: {{generated_at}}
- rhwp 바이너리: `{{rhwp_bin}}`
- DPI: {{dpi}}
- pixel_diff_threshold: {{pixel_diff_threshold}}
- 문서 id: `{{id}}`

## 제출

제출은 **수동**입니다. 이 생성기는 GitHub 이슈를 만들지 않습니다.
