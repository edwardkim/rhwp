---
kind: guide
status: active
canonical: mydocs/report/archive_register_demo/README.md
last_verified: 2026-07-27
---

# 실제 CLI 작동 사례 — 문서 아카이브 대장화 (파일 더미 → 대장)

> 여정: [버그 헌팅 playbook](../../manual/bug_hunting_playbook.md) "대량 아카이브 대장화·조문 DB화".
> 실행: `rhwp` CLI만으로 실문서 271건을 대장화. 정답지 = 개별 `info` 호출.

## 실제 사람 작업

공무원이 쌓인 HWP 더미를 인계받으면, 파일명만으론 무엇인지 알 수 없어 하나씩 열어봐야 한다. `rhwp batch info`로 **포맷·쪽수·용량**을 모으고, 각 문서의 `export-text --json` 첫 비어 있지 않은 줄을 **내용 제목**으로 결합해 대장을 만든다. 현재 CLI에서는 이 과정이 2-pass다.

## 원본 문서 대비 흐름

**왼쪽(원본)**: 불투명한 파일 더미 — `143E433F503322BD33.hwp` 만 봐선 무슨 문서인지 모른다. **오른쪽(대장)**: `batch info` 결과에 `export-text`의 내용 제목을 결합하면 `143E433F503322BD33.hwp` = "상공신문"으로 식별된다.

![파일 더미 → batch → 대장](daejang-before-after.png)

## 산출 대장

![batch info와 export-text를 결합한 문서 대장 대표 16건](register-table.png)

제목 열까지 포함한 전체 271건은 [document_register_271.tsv](document_register_271.tsv) (합계 5,706쪽 · 184MB · hwp5 255 · hwp3 15 · hwpx 1)이다. 위 이미지 아래쪽의 사람용 표기 `문서대장_271.tsv`는 이 저장소 파일을 뜻한다.

## 재현

```bash
# 1) 아카이브 전체 메타데이터 (1회 배치)
ls samples/*.hwp | rhwp batch info --json > register.jsonl
#  → 271건 중 271 성공, 0 실패 (9.9s, threads=12). 손상 CFB 는 lenient 로 복구.
#  → 봉투는 JSONL(줄당 1레코드): {source, format, pageCount, paraCount, sections, sizeBytes, fonts, version}

# 2) 제목은 각 문서의 전체 쪽에서 첫 비어 있지 않은 텍스트 줄로 추출 (2-pass)
rhwp export-text --json DOC \
  | jq -r '[.pages[]?.text // ""] | join("\n") | split("\n")
           | map(gsub("^[[:space:]]+|[[:space:]]+$"; ""))
           | map(select(length > 0)) | .[0] // "(제목 없음)"'
#  → 이 값을 register.jsonl의 source에 파일명으로 결합한 결과가 document_register_271.tsv
```

## 검증 (정답지 대조)

- `batch info` 결과를 개별 `rhwp info` 와 대조: 6개 표본에서 `format·pageCount·paraCount·sections·sizeBytes·version` **전 필드 일치**. batch 는 개별 호출과 동일한 값을 병렬로 낸다.
- `batch` 는 총량을 **정직하게 보고**한다("271건 중 271 성공, 0 실패") — 절단·은폐 없음.

## 발견한 격차

`info`/`batch info` 봉투에 문서를 식별할 **제목**이 없어, 대장화에 `export-text --json` 2-pass와 위의 소비자측 파싱 규칙이 필요하다. 1-pass 대장화를 위한 `title` 필드 추가를 [#3407](https://github.com/edwardkim/rhwp/issues/3407) 로 제안했다. 배치 처리 자체(속도·정확·에러 복구·총량 보고)는 이 규모에서 견고했다.

## 관련

- 방법론: [버그 헌팅 playbook](../../manual/bug_hunting_playbook.md)
- 명령 계약: [CLI 명령어 매뉴얼](../../manual/cli_commands.md)
- 개선 제안: [#3407](https://github.com/edwardkim/rhwp/issues/3407)
