---
kind: guide
status: active
canonical: mydocs/manual/recipes/06_visual_regression_before_after.md
last_verified: 2026-08-03
---

# 레시피 6 — 편집 전후를 눈이 아니라 숫자로 비교하기

**목표 한 줄**: `edit fill-fields`·`edit set-cell`·`export-hwpx` 같은 편집/변환
명령을 돌린 뒤, "내용이 바뀌었다"가 아니라 "**의도한 것만** 바뀌고 나머지 레이아웃은
그대로다"를 사람 눈이 아니라 픽셀 단위 수치로 판정한다.

[레시피 1](01_fill_form_and_submit.md)의 `--verify`는 "요청한 값이 IR에 들어갔는가"를
확인한다. 이 레시피의 `render-diff`는 한 걸음 더 나아가 "그 결과가 **실제로
렌더링됐을 때** 예상과 얼마나 다른가"를 mm/px 단위로 잰다. 표 병합·폰트
치환·페이지 넘김처럼 IR 비교로는 안 잡히지만 화면에서는 티가 나는 차이를 잡을 때
쓴다.

절차 요약:

```
render-diff <파일> --via hwpx   (자기 자신의 HWP↔HWPX 왕복 일관성, 단일 파일)
  또는
render-diff <파일A> <파일B>     (편집 전 vs 편집 후, 두 파일 비교)
  또는
render-diff --batch <폴더>      (여러 파일을 한 번에, CI 게이트용)
```

모든 명령·출력은 이 저장소의 `samples/form-01.hwp`와, [레시피
5](05_mail_merge_batch_fill.md)에서 `batch fill`로 실제 생성한 산출물
`batch_out/0001.hwp`(같은 서식에 `myMsg01` 필드를 채운 결과)로 실행해서
얻었다. 지어낸 값은 없다.

## 0단계 — 이 명령이 재는 것은 정확히 무엇인가

`render-diff`는 두 대상(또는 한 파일을 서로 다른 경로로 두 번 렌더링한 결과)을
`export-render-tree`와 같은 방식으로 렌더링한 뒤, 페이지별 render tree의 각
노드(`Page/Body2/Column0/TextLine.../TextRun...` 같은 경로)를 짝지어 위치 변위를
비교한다. 두 가지 판정축이 있다:

- **변위(displacement)**: 짝지어진 같은 경로의 노드가 px 단위로 얼마나
  움직였는가. `--max-disp`로 허용 임계값을 정한다(기본 1.00px).
- **구조 불일치(STRUCT)**: 노드 개수 자체가 다르거나(`Δ TextRun: 15→13`처럼),
  경로가 아예 짝지어지지 않는 경우. 변위 임계값과 무관하게 그 자체로 `STRUCT`
  플래그가 붙는다.

## 1단계 — 가장 싼 점검: 파일 하나로 HWP5↔HWPX 왕복 일관성 확인

편집 없이도 쓸 수 있는 가장 기본적인 회귀 점검이다. `--via hwpx`는 HWP5 원본을
HWPX로 변환했다가 다시 렌더링해서, 원본 HWP5 렌더링과 비교한다 — 포맷 변환
경로 자체가 레이아웃을 깨뜨리지 않는지 보는 것이다.

```bash
rhwp render-diff samples/form-01.hwp --via hwpx
```

실측 출력:

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

`status: PASS`, 종료 코드 0. 페이지 수(`A=1 B=1`)가 같고 최대 변위가 0px이니
포맷 왕복이 레이아웃에 아무 영향도 주지 않았다는 뜻이다. 실제로 종료 코드를
확인해보면:

```bash
rhwp render-diff samples/form-01.hwp --via hwpx > /dev/null; echo $?
```

```
0
```

CI 게이트에서는 이 종료 코드 하나로 판정할 수 있다 — `render-diff`는 `--json`을
지원하지 않는 텍스트 전용 출력이므로(`export-svg`·`fields` 등과 달리), 파이프라인
게이트는 stdout을 파싱하기보다 **종료 코드**를 1차 판정으로 쓰고, 실패 원인
분석에만 텍스트 출력을 읽는다.

특정 페이지만 좁혀서 볼 때는 `-p`를 쓴다:

```bash
rhwp render-diff samples/form-01.hwp -p 0 --via hwpx
```

실측 출력(1페이지짜리 문서라 결과는 동일):

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

## 2단계 — 진짜 회귀 시나리오: 편집 전 vs 편집 후

이제 실제로 값이 바뀐 두 파일을 비교한다. [레시피 5](05_mail_merge_batch_fill.md)에서
`batch fill`로 만든 `batch_out/0001.hwp`(누름틀 `myMsg01`에 "김철수 귀하"를 채운
결과)를 원본 빈 서식과 비교한다.

```bash
rhwp render-diff samples/form-01.hwp batch_out/0001.hwp
```

실측 출력:

```
페이지 수: A=1 B=1
최대 변위: 495.93 px (page 0)
임계 초과 페이지: 1 / 구조 불일치 페이지: 1 (임계 1.00px)
  page   0: max= 495.93 mean= 13.40 nodes=39/37  [STRUCT]
       495.93px  Page/Body2/Column0/TextLine10/TextRun0
         0.00px  Page
         0.00px  Page/PageBg0
      Δ TextRun: 15→13 (-2)
status: STRUCT_MISMATCH
```

이걸 어떻게 읽나:

- `status: STRUCT_MISMATCH` — 종료 코드 1로 실패 취급된다. 하지만 **이건
  버그가 아니다** — 빈 누름틀("여기에 입력")이 실제 값("김철수 귀하")으로
  바뀌면서 그 자리의 텍스트런 구조 자체가 달라졌기 때문에 당연히 나오는
  결과다. `Δ TextRun: 15→13 (-2)`가 그 변화의 크기를 정량적으로 보여준다.
- `Page/Body2/Column0/TextLine10/TextRun0`에서 `495.93px`의 변위가 잡혔다 —
  바로 그 누름틀이 있던 줄(`TextLine10`)의 첫 텍스트런이다. **바뀐 값이 있는
  위치와 렌더링 변위가 보고된 위치가 일치하는지**를 대조하는 게 이 단계의
  핵심 판정이다. 만약 전혀 관계없는 페이지나 다른 단(段)에서 변위가 잡혔다면
  — 예를 들어 상단 로고나 다른 문단이 움직였다면 — 그건 의도치 않은 부작용
  이고 진짜 회귀다.
- `Page`, `Page/PageBg0`처럼 상위 구조 노드는 `0.00px`로 그대로다 — 페이지
  배경·전체 틀은 안 건드렸다는 뜻이다.

`--max-disp`로 임계값을 조여서 더 엄격하게 판정할 수도 있다(구조 불일치는
임계값과 무관하게 항상 플래그된다는 점은 동일):

```bash
rhwp render-diff samples/form-01.hwp batch_out/0001.hwp --max-disp 0.05
```

실측 출력(임계값 표시만 바뀜, 나머지는 위와 동일한 실측 판정):

```
페이지 수: A=1 B=1
최대 변위: 495.93 px (page 0)
임계 초과 페이지: 1 / 구조 불일치 페이지: 1 (임계 0.05px)
  page   0: max= 495.93 mean= 13.40 nodes=39/37  [STRUCT]
       495.93px  Page/Body2/Column0/TextLine10/TextRun0
         0.00px  Page
         0.00px  Page/PageBg0
      Δ TextRun: 15→13 (-2)
status: STRUCT_MISMATCH
```

## 2-1단계 — 같은 편집을 받은 두 산출물끼리 비교해서 "값 종류"가 아니라 "편집 종류"를 검증한다

`batch fill`([레시피 5](05_mail_merge_batch_fill.md))로 같은 서식에 다른 값을
채운 산출물이 여러 개 있다면, 원본과의 비교뿐 아니라 산출물끼리도 비교할 수
있다. `batch_out/0001.hwp`("김철수 귀하")와 `batch_out/0002.hwp`("이영희
귀하")는 둘 다 같은 필드에 같은 글자 수의 값을 채운 결과다.

```bash
rhwp render-diff batch_out/0001.hwp batch_out/0002.hwp
```

실측 출력:

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

`status: PASS` — 값은 다르지만("김철수 귀하" vs "이영희 귀하") 글자 수가
같아서 텍스트런 구조가 동일하게 유지됐다. 이건 "같은 종류의 편집(같은 필드,
같은 길이대의 값)을 여러 행에 적용했을 때 산출물들이 서로 구조적으로
일관된가"를 확인하는 용도다 — 메일머지처럼 같은 서식을 대량으로 채울 때, 몇
건만 표본으로 뽑아 서로 비교해 보면 특정 값에서만 예외적으로 레이아웃이
깨지는 행을 찾아낼 수 있다.

동일한 파일을 자기 자신과 비교하면(회귀 테스트의 기준선 역할) 항상 `PASS`가
나와야 한다는 것도 같이 확인해 둔다:

```bash
rhwp render-diff batch_out/0001.hwp batch_out/0001.hwp
```

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

이 자기 비교가 `PASS`가 아니라 무언가 다른 값을 낸다면, `render-diff` 자체나
렌더링 파이프라인에 비결정성(non-determinism)이 있다는 훨씬 심각한 신호다 —
매 실행마다 같은 입력에 대해 같은 출력이 나오는지는 회귀 도구의 기본 전제이므로,
이 자기 비교를 CI에 상시 기준선으로 하나 심어두는 것도 방법이다.

## 3단계 — CI 게이트: 여러 파일을 한 번에 훑는다

문서 여러 개를 한 폴더에 모아두고 왕복 일관성을 배치로 확인한다. TSV로도
저장되므로, CI 아티팩트로 남기거나 스프레드시트에서 추이를 볼 수 있다.

```bash
mkdir -p rd_batch
cp samples/form-01.hwp samples/form-02.hwp rd_batch/
rhwp render-diff --batch rd_batch --via hwpx -o rd_out
```

실측 출력:

```
[           PASS] max_disp=   0.00 struct=0 over=0      5ms  form-01.hwp
[           PASS] max_disp=   0.00 struct=0 over=0      4ms  form-02.hwp

TSV 저장: rd_out\geom_inventory.tsv

=== render-diff 요약 ===
  총 파일         : 2
  PASS            : 2
  WARN_TEXTRUN    : 0
  OVER            : 0
  STRUCT_MISMATCH : 0
  PAGE_MISMATCH   : 0
  LOAD_FAIL       : 0
  전체 최대 변위  : 0.00 px
```

`rd_out/geom_inventory.tsv`를 열어보면 파일별 판정이 표로 저장돼 있다:

```
sample	status	pages_a	pages_b	max_disp	worst_page	struct_pages	over_pages	elapsed_ms	error	struct_delta
form-01.hwp	PASS	1	1	0.000	-	0	0	5		
form-02.hwp	PASS	1	1	0.000	-	0	0	4		
```

컬럼 그대로 CI 대시보드나 회귀 추이 스프레드시트에 붙여넣을 수 있다. 배치 요약에
나오는 상태값은 5가지다: `PASS`(문제 없음), `WARN_TEXTRUN`(텍스트런 경고 수준
차이), `OVER`(변위가 임계 초과), `STRUCT_MISMATCH`(구조 자체가 다름),
`PAGE_MISMATCH`(페이지 수 자체가 다름), `LOAD_FAIL`(파일을 못 열었음).

## 실패했을 때 무엇을 보고 어떻게 고칠지

| 신호 | 원인 | 처방 |
|---|---|---|
| `오류: 파일 읽기 실패 ...` (종료 코드 1) | 경로가 잘못됨 | 이 저장소 표본으로 재현: `rhwp render-diff samples/no-such.hwp` → `지정된 파일을 찾을 수 없습니다.` |
| `오류: 폴더 읽기 실패 ...` (`--batch`, 종료 코드 2) | `--batch` 폴더 경로가 잘못됨 | 폴더가 실제로 존재하는지, 상대 경로 기준이 맞는지 확인 |
| `status: STRUCT_MISMATCH`인데 변위 위치가 편집 의도와 일치 | 정상 — 값이 바뀐 자리는 구조도 바뀐다 | 실패로 취급하지 않는다. 변위 노드 경로(`Page/Body2/...`)가 편집한 필드 위치와 맞는지 사람이 한 번 대조 |
| `status: STRUCT_MISMATCH`인데 변위 위치가 편집과 무관한 페이지/단 | 진짜 회귀 — 의도치 않은 레이아웃 변화 | 편집 명령의 `--verify`(레시피 1) 결과와 대조하고, `export-svg`로 두 산출물을 나란히 눈으로도 확인 |
| `status: PAGE_MISMATCH` | 페이지 수 자체가 달라짐(내용 증가로 새 페이지가 생겼거나 줄어듦) | 의도한 변경이면 정상, 아니면 `dump-pages --json`으로 어느 페이지에서 갈라지는지 좁힌다 |
| `status: LOAD_FAIL` (배치 모드) | 그 파일만 파싱 자체가 실패 | `info --json`으로 그 파일 하나만 따로 열어 원인 확인 — [문서 진단 도구](../document_diagnostics_tool_manual.md) |
| 배치 요약의 `OVER` 건수가 0이 아님 | 변위가 `--max-disp` 임계를 넘었지만 구조는 동일 | 임계값이 너무 빡빡한지, 실제 폰트/여백 회귀인지 TSV의 `worst_page`로 좁혀 확인 |

## 이 레시피가 하지 않는 것

- 텍스트 내용 자체의 옳고 그름은 판정하지 않는다 — "값이 정확히 채워졌는가"는
  [레시피 1](01_fill_form_and_submit.md)의 `--verify`(IR 비교) 몫이다.
  `render-diff`는 그 값이 **렌더링됐을 때 레이아웃에 미친 영향**만 잰다.
- 색상·폰트 렌더링 품질의 픽셀 대 픽셀 비교는 아니다 — render tree의 노드
  위치·구조 비교이지, 실제 래스터 이미지 diff가 아니다. 진짜 시각적 이미지
  비교가 필요하면 `export-svg`/`export-png`로 뽑은 산출물을 별도 이미지 diff
  도구로 비교한다.
- `--json` 출력을 지원하지 않는다(2026-08-03 기준, `rhwp v0.8.2`) — 자동화
  게이트는 종료 코드를 1차 판정으로 쓰고, `--batch` 모드의 TSV를 2차 상세
  분석 자료로 쓴다.

## 요약

`render-diff`는 세 가지 모드를 제공한다: 파일 하나의 포맷 왕복 일관성(`--via
hwpx`), 두 파일 간 편집 전후 비교, 폴더 단위 배치 비교. 이 저장소 표본으로
실제 확인한 것: 빈 서식을 채우면 그 자리에서 `STRUCT_MISMATCH`가 나는 게
정상(값이 바뀌었으니 구조도 바뀐다)이라는 점, 같은 편집을 받은 산출물끼리는
글자 수가 같으면 `PASS`가 나온다는 점, 자기 자신과의 비교는 항상 `PASS`여야
한다는 점(회귀 도구 자체의 결정성 기준선), `--json` 출력이 없어 종료 코드를
1차 판정으로 쓴다는 점. `STRUCT_MISMATCH`를 보면 반사적으로 실패 처리하지
말고, 변위가 잡힌 노드 경로가 실제로 편집한 위치와 일치하는지부터 확인한다.

요컨대 이 레시피의 핵심 습관은 하나다 — `render-diff`가 빨간불을 켜면 곧바로
롤백하지 말고, 그 빨간불이 가리키는 노드 경로부터 읽는다. 편집한 자리와
일치하면 예상된 결과이고, 엉뚱한 자리를 가리키면 그때부터가 진짜 디버깅
시작이다.

## 관련 문서

- [레시피 1 — 서식 문서를 채워서 제출용으로 만들기](01_fill_form_and_submit.md) —
  `--verify`(IR 비교)와 이 레시피의 `render-diff`(렌더링 비교)가 서로 다른
  질문에 답한다는 점.
- [레시피 5 — 서식 하나에 여러 사람 데이터를 한 번에 채우기](05_mail_merge_batch_fill.md) —
  이 레시피에서 비교 대상으로 쓴 `batch_out/0001.hwp`를 만든 절차.
- [문서 진단 도구](../document_diagnostics_tool_manual.md) — `LOAD_FAIL`이나
  `PAGE_MISMATCH`가 나왔을 때 `info → dump → diag` 순서로 원인을 더 좁히는
  절차.
- [CLI 명령어 매뉴얼](../cli_commands.md) — `render-diff`·`export-render-tree`의
  전체 옵션.
- [레시피 4 — 출처를 모르는 문서를 처음 열 때](04_safety_check_untrusted_doc.md) —
  편집 대상 문서 자체가 낯선 출처라면, 렌더링 비교 이전에 먼저 밟는 안전
  점검 절차.
