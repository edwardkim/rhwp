---
name: rhwp-visual-regression
description: 편집/변환 전후의 HWP/HWPX 레이아웃 회귀를 숫자로 판정합니다. render-diff(자기 라운드트립·두 파일 비교·폴더 배치, px 변위+구조 불일치) → ir-diff(구조 차이) → thumbnail/export-png(눈 검증) 판단 트리를 태우고, STRUCT_MISMATCH 를 반사적으로 실패 처리하지 않고 노드 경로로 판독합니다. 트리거 — 사용자가 "편집 전후 화면 비교", "레이아웃 회귀/깨졌는지 확인", "라운드트립 시각 검증", "render-diff 돌려줘", "바뀐 게 의도한 것뿐인지" 등을 요청할 때.
---

# rhwp-visual-regression — 전후 시각 회귀 판정 Skill

## 목적

`edit`/`convert`/`export-hwpx` 를 돌린 뒤 "내용이 바뀌었다"가 아니라 "**의도한 것만**
바뀌고 나머지 레이아웃은 그대로다"를 사람 눈이 아니라 px 단위 수치로 판정한다.
IR 비교(`--verify`)로는 안 잡히지만 화면에서는 티가 나는 차이(표 병합·폰트 치환·
페이지 넘김)를 잡는다.

권위 출처: [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md)
(§render-diff · §ir-diff · §thumbnail · §export-png). 절차의 실측 원형은
[레시피 6](../../../mydocs/manual/recipes/06_visual_regression_before_after.md).

## 바이너리 실행

```bash
cargo build --release        # 최초 1회 또는 소스 변경 후
./target/release/rhwp <명령> [옵션]
```
`export-png` 은 `native-skia` feature 빌드 필요(release 바이너리에 포함됨).
(공통 규약은 [rhwp-cli skill](../rhwp-cli/SKILL.md) 참조)

## 판단 트리

```
1. render-diff <파일> [--via hwpx|hwp]        가장 싼 점검 — 자기 라운드트립(포맷 왕복이 레이아웃을 깨뜨리나)
   render-diff <A> <B>                        편집 전 vs 후 두 파일 직접 비교
   render-diff --batch <폴더> [-o 출력폴더]    폴더 전수 → geom_inventory.tsv (CI 게이트용)
        │
        ├─ PASS → 끝. (자기 비교 A==A 는 항상 PASS 여야 한다 — 회귀 도구의 결정성 기준선)
        │
        ├─ STRUCT_MISMATCH → 변위 노드 경로를 읽는다 (반사적 실패 처리 금지)
        │     경로가 편집한 위치와 일치 → 정상 (값이 바뀌면 그 자리 구조도 바뀐다)
        │     편집과 무관한 페이지/단     → 진짜 회귀 ↓
        │
        ├─ PAGE_MISMATCH → 페이지 수 자체가 다름 → dump-pages 로 갈라지는 쪽 좁힘
        └─ OVER/LOAD_FAIL → 임계·파싱 문제 ↓
2. ir-diff <A.hwpx> <B.hwp> [-s N] [-p M] [--summary] [--json]   구조(IR) 차이 국소화
3. thumbnail / export-png / export-svg --debug-overlay           눈 검증 — 산출물을 실제로 본다
4. export-render-tree <파일> -p N                                 정밀 bbox 좌표 diff
```

## 요청 → 명령 매핑

| 사용자 요청 | 명령 |
|------------|------|
| "이 파일 포맷 왕복이 안전한지" | `render-diff <파일> --via hwpx` (HWP 어댑터 경로는 `--via hwp`) |
| "편집 전후 비교해줘" | `render-diff <전.hwp> <후.hwp> [-p N] [--max-disp <px>]` |
| "폴더 전체 회귀 게이트" | `render-diff --batch <폴더> [-o 출력폴더]` → `geom_inventory.tsv` |
| "어느 구조가 달라졌는지" | `ir-diff <a.hwpx> <b.hwp> [-s N] [-p M] [--summary] [--json]` |
| "빨리 눈으로 확인" | `thumbnail <파일> [--data-uri]` / `export-png <파일> [-p N] [--vlm-target claude]` |
| "문단/표 경계 겹쳐서 보여줘" | `export-svg <파일> --debug-overlay -p N` |
| "정밀 좌표로 비교" | `export-render-tree <파일> -p N` → bbox JSON 을 전/후 diff |

## 출력 판독법 (레시피 6 실측)

PASS(자기 라운드트립, `samples/form-01.hwp` 실측):

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

편집 전 vs 후(빈 서식 vs `batch fill` 산출물, 실측):

```
페이지 수: A=1 B=1
최대 변위: 495.93 px (page 0)
임계 초과 페이지: 1 / 구조 불일치 페이지: 1 (임계 1.00px)
  page   0: max= 495.93 mean= 13.40 nodes=39/37  [STRUCT]
       495.93px  Page/Body2/Column0/TextLine10/TextRun0
         0.00px  Page
      Δ TextRun: 15→13 (-2)
status: STRUCT_MISMATCH
```

빈 누름틀("여기에 입력")을 실제 값("김철수 귀하")으로 채우면 그 줄의 텍스트런 구조가
달라져 `STRUCT_MISMATCH` + `Δ TextRun: 15→13 (-2)` 가 나온다 — **이건 버그가 아니다**.
핵심 판정은 **변위가 보고된 노드 경로**(`Page/Body2/Column0/TextLine10/TextRun0`)가
**편집한 필드 위치와 일치하는가**다. 상단 로고·다른 문단이 움직였다면 그것이 진짜
회귀다. 상위 구조 노드(`Page`·`PageBg0`)가 0.00px 이면 전체 틀은 안 건드린 것이다.

- 같은 편집을 받은 산출물끼리(메일머지 표본 몇 건) 비교하면 값이 달라도 글자 수가
  같으면 PASS — 특정 값에서만 레이아웃이 깨지는 행을 찾는 용도다.
- 구조 불일치 시 노드 타입별 순증감(`Line:-4;RawSvg:-1`)이 출력된다 — 음수=라운드트립
  손실, 양수=추가. 손실 노드 타입으로 직렬화 누락 원인을 즉시 좁힌다.
- `--batch` 의 `geom_inventory.tsv` 컬럼: sample/status/pages_a/pages_b/max_disp/
  worst_page/struct_pages/over_pages/elapsed_ms/error/struct_delta — CI 아티팩트·회귀
  추이 표로 그대로 쓴다.

## 봉투·종료 코드 규약 — 판정은 데이터다

- **`render-diff` 는 `--json` 을 지원하지 않는다**(v0.8.2 실측, 텍스트 전용) —
  자동화 게이트는 **종료 코드를 1차 판정**으로 쓴다: `PASS` 만 0,
  `OVER`/`STRUCT_MISMATCH`/`PAGE_MISMATCH`/`LOAD_FAIL` 은 1. 상세 분석은
  `--batch` 의 TSV(컬럼: sample/status/…/struct_delta)를 2차 자료로 읽는다.
- **`ir-diff --json` 은 차이 발견이 exit 3** 이다(0=동일 / 1=읽기·파싱 실패,
  stdout 0바이트 / 2=사용법 오류). 봉투는 한 줄:
  `{"schemaVersion":"1.0","a","b","identical","diffCount","categories":{…}}` —
  변환 파이프라인 게이트는 `rhwp ir-diff A B --json || 격리처리` 꼴로 닫는다.
  단, **기본(텍스트) 모드의 정상 비교는 차이가 있어도 0**(기존 소비자 무변경).
- `edit` 계열의 `--verify` 도 저장 직후 IR 차이 시 exit 3 — 종료 코드 3 은 "실패"가
  아니라 "회귀(차이) 검출"이라는 **데이터**다. 소비자가 의도한 차이인지 판정한다.
- `--max-disp` 기본 1.0px. 구조 불일치는 임계값과 **무관하게** 항상 플래그된다.

## 함정 (실측)

- **STRUCT_MISMATCH 를 반사적으로 실패 처리하지 않는다** — 편집한 자리의 구조 변화는
  당연한 결과다. 빨간불이 가리키는 노드 경로부터 읽는다.
- **자기 라운드트립 통과 ≠ 한컴 충실도** — render-diff 는 내부 회귀 방지용이다.
  최종 게이트는 한컴 수동 검증(rhwp-cli skill §검증·주의와 동일).
- **자기 비교(A==A)가 PASS 가 아니면** render-diff 나 렌더링 파이프라인에 비결정성이
  있다는 훨씬 심각한 신호다 — CI 상시 기준선으로 심어둘 가치가 있다.
- `render-diff` 의 render tree 노드 위치·구조 비교이지 **래스터 픽셀 diff 가 아니다** —
  색상·폰트 렌더링 품질까지 봐야 하면 `export-svg`/`export-png` 산출물을 별도 이미지
  diff 로 비교한다.
- `--batch` 폴더 경로가 잘못되면 exit 2(`오류: 폴더 읽기 실패`), 단건 파일 경로가
  잘못되면 exit 1 — 게이트 스크립트에서 두 실패를 구분한다.
- 페이지 번호는 0부터(`-p 0` 이 첫 쪽). PDF/한컴 표기(1부터)와 혼동 주의.
- `thumbnail` 은 HWP **내장** 썸네일(PrvImage) 추출이다 — 편집 후 재렌더가 아니라
  저장 시점의 미리보기라, 전후 눈 검증의 기준은 `export-png`/`export-svg` 쪽이다.

## 상세 레퍼런스

- `render-diff`·`ir-diff` 전체 옵션: [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md)
- 전후 비교 실측 절차: [`recipes/06_visual_regression_before_after.md`](../../../mydocs/manual/recipes/06_visual_regression_before_after.md)
- `ir-diff` 상세: [`mydocs/manual/ir_diff_command.md`](../../../mydocs/manual/ir_diff_command.md)
- `export-png` 상세: [`mydocs/manual/export_png_command.md`](../../../mydocs/manual/export_png_command.md)
- LOAD_FAIL/PAGE_MISMATCH 원인 좁히기: [`mydocs/manual/document_diagnostics_tool_manual.md`](../../../mydocs/manual/document_diagnostics_tool_manual.md)
