# 분석·디버깅 판단 트리

권위는 `mydocs/manual/cli_commands.md` 와 `src/main.rs` 디스패치다.
이 장은 기존 CLI 를 설명하고 소비 규칙을 고정한다. 새 명령·새 플래그를 발명하지 않는다.

이 스킬은 **gym 이 아니다.** 실사용 에이전트가 HWP/HWPX 를 내보내고 레이아웃을 좁히는 경로다.

## 한 줄

요청을 명령으로 매핑하고, 겹침·간격은 overlay → dump-pages → dump → ir-diff → render-tree → hwp5-inventory-diff 순으로 좁힌다.

페이지는 0부터. 자기 라운드트립 통과는 한컴 호환이 아니다.

## 요청 트리

```
사용자가 파일을 준다
  ├─ 없다 / 깨졌다 ──▶ 예외 봉투 (missing-file · load-fail). 명령을 발명하지 않음
  ├─ 규모만 ──▶ info --json
  ├─ 본문 ──▶ export-text [--json --max-chars]
  ├─ 그림/인쇄 ──▶ export-svg / export-png / export-pdf
  ├─ 레이아웃·겹침 ──▶ 디버그 6단 (아래)
  ├─ 두 파일 내용 ──▶ ir-diff --json
  └─ 한컴 저장과 다름 ──▶ hwp5-inventory-diff oracle generated
```

## 레이아웃 디버그 6단 (강제 순회가 아님, 기본 순서)

```
1 export-svg --debug-overlay -p N
    └─ 라벨 s{섹션}:pi={인덱스} y={좌표}
2 dump-pages -p N
    └─ 배치 목록 + vpos/lh/ls
3 dump -s N -p M
    └─ ParaShape / LINE_SEG / 표 속성
4 ir-diff a.hwpx b.hwp [-s N -p M] [--json]
    └─ 형식 쌍이 있을 때만. 차이는 데이터
5 export-render-tree -p N
    └─ bbox JSON. translate 단위
6 hwp5-inventory-diff oracle.hwp generated.hwp
    └─ 저장 계약. oracle=한컴, generated=rhwp
```

답이 나오면 다음 단으로 내려가지 않는다. overlay 라벨만으로 문단이 보이면 dump 로 점프해도 된다.
순서를 건너뛰어 render-tree 부터 여는 것은 금지 기본값이 아니다 — 다만 인덱스 없이 bbox 를 읽으면 좌표만 떠다닌다.

## 분기 필드

| 단계 | 보는 것 | 다음 |
|---|---|---|
| info | pageCount, format, version | 규모만이면 정지 |
| export-svg --json | overflowCellLines | >0 이면 셀 소실 |
| dump-pages | vpos/lh/ls | 높이 이상이면 dump |
| ir-diff --json | identical, diffCount, categories | exit 3 = 데이터 |
| convert --verify | exit 3, 산출물 잔류 | 한컴 검증은 별도 |
| hwp5-* | oracle vs generated | 순서를 뒤집지 않음 |

## 페이지와 단위

- `-p` / export-text `pages[].page` / search matches 는 **0부터**.
- 사용자가 "4쪽" 이라고 하면 한컴·PDF 표기이므로 `-p 3`.
- `extract-pages --from/--to` 만 **1부터**. 이 스킬의 기본 축이 아니다.
- 1인치=7200 HWPUNIT=96px, 1px=75 HWPUNIT, 1mm≈283.46 HWPUNIT.

## 금지 진입

- 새 CLI 하위명령·플래그 발명
- gym/ 팩으로 이 트리를 대체
- 자기 hwp5-roundtrip 통과를 한컴 호환으로 보고
- oracle/generated 순서를 추측으로 뒤집기
- 페이지 1부터를 `-p` 에 그대로 넣기
- 없는 파일·깨진 파일을 빈 성공으로 삼키기
- DocumentCore 편집 로직을 이 스킬에서 고치기

## 관련

[01_request_command_map.md](01_request_command_map.md) · [17_layout_debug_order.md](17_layout_debug_order.md) · [21_exception_envelopes.md](21_exception_envelopes.md)

## 질문 카드

| 질문 | 첫 명령 | 정지 |
|---|---|---|
| 이 파일 뭐야 | info --json | 규모만이면 정지 |
| 1쪽 그림으로 | export-svg -p 0 | 파일 생성 |
| 겹친다 | export-svg --debug-overlay | 라벨 |
| 몇 문단이 이 쪽 | dump-pages -p N | 목록 |
| 줄간격 숫자 | dump -s -p | LINE_SEG |
| 두 파일이 같나 | ir-diff --json | identical |
| 한컴이 안 연다 | hwp5-inventory-diff | 축 힌트 |
| PNG 가 거절 | native-skia 봉투 | 재빌드 안내 |
