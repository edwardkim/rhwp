# 레이아웃·겹침 디버그 순서

코드 무수정으로 결함을 좁힌다. 순서는 이슈 #5316 과 스킬 본문이 같은 단어를 쓴다.

1. `export-svg --debug-overlay`
2. `dump-pages`
3. `dump`
4. `ir-diff`
5. `export-render-tree`
6. `hwp5-inventory-diff`

`cli_commands.md` §6 은 5·6 이 뒤바뀐 참고 순서가 있다. **이 스킬의 계약 순서는 위 여섯**이다.
render-tree(좌표)를 inventory(저장 record)보다 먼저 본다. 화면 버그는 좌표가 먼저다.

## 단별 산출

### 1. `export-svg`

- 플래그: `--debug-overlay`, `-p`
- 이유: 문단/표 식별. 라벨은 s{섹션}:pi={인덱스} y={좌표}
- 산출: SVG overlay

### 2. `dump-pages`

- 플래그: `-p`
- 이유: 해당 페이지 문단/표 배치 목록과 높이(vpos/lh/ls)
- 산출: pagination dump

### 3. `dump`

- 플래그: `-s`, `-p`
- 이유: ParaShape / LINE_SEG / 표·도형 속성 상세
- 산출: control dump

### 4. `ir-diff`

- 플래그: `-s`, `-p`, `--json`
- 이유: HWPX↔HWP IR 불일치. --json 이면 차이는 exit 3 (판정 데이터)
- 산출: IR categories

### 5. `export-render-tree`

- 플래그: `-p`
- 이유: bbox JSON. SVG 문자열 비교보다 좌표 분석에 정확
- 산출: render_tree_NNN.json

### 6. `hwp5-inventory-diff`

- 플래그: `--report`, `--focus`
- 이유: HWPX→HWP 저장 계약. oracle=한컴 저장본, generated=rhwp 저장본
- 산출: inventory hints

## 겹침 여정 (실측 순서)

```bash
# 사용자가 "3쪽이 겹친다" — 한컴 3쪽 = -p 2
rhwp export-svg 보고서.hwp --debug-overlay -p 2 -o output/poc/overlap/
# SVG 라벨에서 s0:pi=14 y=... 를 읽는다
rhwp dump-pages 보고서.hwp -p 2
# 문단 14 의 vpos/lh 가 옆 표와 겹치면
rhwp dump 보고서.hwp -s 0 -p 14
# HWPX 원본이 있으면
rhwp ir-diff 보고서.hwpx 보고서.hwp -s 0 -p 14 --json
# 좌표를 숫자로
rhwp export-render-tree 보고서.hwp -p 2 -o output/poc/overlap/tree/
# 한컴 저장본이 있으면
rhwp hwp5-inventory-diff oracle.hwp generated.hwp --focus table
```

## 정지

| 단 | 멈추는 때 |
|---|---|
| 1 | overlay 라벨만으로 문단/표가 특정됨 |
| 2 | 높이 숫자가 겹침을 설명함 |
| 3 | ParaShape/LINE_SEG 가 원인 후보 |
| 4 | identical:true 이면 형식 쌍 문제는 아님 |
| 5 | bbox 가 두 파일에서 갈라짐 |
| 6 | record 힌트가 저장 축을 가리킴 |

구현 수정은 별도 이슈다. 이 스킬은 좁히기만 한다.

## 보정 전/후

레이아웃 변경의 회귀는 테스트 golden 통과로 안 잡힐 수 있다.
기준 브랜치 SVG 와 변경 SVG 를 `output/poc/before|after` 에 두고 페이지별로 본다.
좌표 분석은 render-tree JSON diff 가 SVG 문자열보다 정확하다.
셀 내부는 `translate(x,y)` 단위다.

시각 회귀의 **숫자 게이트**(render-diff max-disp)는 `rhwp-visual-regression` 스킬.
여기서는 그 명령을 발명하지 않고, 사람이 overlay 를 읽는 사다리만 닫는다.
