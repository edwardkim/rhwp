# 02 — render-diff 두 파일

편집 전 vs 후, 또는 같은 종류의 산출물끼리.

```bash
rhwp render-diff samples/form-01.hwp batch_out/0001.hwp
```

레시피 06 실측 (빈 서식 vs `myMsg01`="김철수 귀하"):

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

텍스트 모드 종료 코드 1. **버그가 아니다.** 빈 누름틀이 실제 값으로
바뀌면 그 자리 텍스트런 구조가 달라진다. 핵심은 경로
`Page/Body2/Column0/TextLine10/TextRun0` 가 편집한 필드와 맞는가다.

같은 글자 수 산출물끼리:

```bash
rhwp render-diff batch_out/0001.hwp batch_out/0002.hwp
```

```
status: PASS
최대 변위: 0.00 px
```

값은 달라도("김철수 귀하" vs "이영희 귀하") 글자 수가 같으면 구조가
유지된다. 메일머지에서 특정 값만 레이아웃이 깨지는 행을 찾을 때 쓴다.

## JSON

```bash
rhwp render-diff 전.hwp 후.hwp --json
```

`mode` 는 `"pair"`, `via` 는 null, `sourceA`/`sourceB` 가 두 경로.
하드 실패는 exit **3** (`regression: true`).

## 페이지 필터

`-p N` 은 0 부터. 한컴 쪽번호 1 과 혼동하지 않는다.
