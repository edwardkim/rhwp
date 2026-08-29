---
kind: reference
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-08-29
---

# 저장소 정답지(`pdf/`)로 조판을 판정하는 법

`pdf/` 에는 한글이 직접 뽑은 출력 **573 장**이 들어 있다. 그 쪽수·텍스트는 "한글이 이 문서를
어떻게 조판했는가" 의 정답이므로, **한컴오피스 설치본 없이** rhwp 의 조판을 판정할 수 있다.

## 1. 기존 오라클 도구와 무엇이 다른가

| | [한글 페이지 충실도 오라클](hangul_page_oracle.md) | [한글 버전별 대조](hangul_version_oracle.md) | **이 문서** |
| --- | --- | --- | --- |
| 필요 환경 | Windows + 한컴오피스 + `pyhwpx` (COM) | 〃 (버전별 설치본) | **클론만** |
| 정답지 | 그 자리에서 한글로 연 결과 | 〃 | 저장소에 **이미 있는** `pdf/` |
| 대상 | 원본 ↔ 저장본 왕복 | 한글 버전 간 차이 | **원본 ↔ 한글 출력** |
| 누가 돌릴 수 있나 | 한컴 설치 환경 | 〃 | **모든 기여자·CI** |

기존 도구는 한글을 **실행**해 정답을 만든다. 이쪽은 이미 만들어져 커밋된 정답을 **읽는다.**
서로 대체하지 않는다 — 기존 도구는 새 문서·새 조건에서 정답을 만들 수 있고, 이쪽은 코퍼스가
고정된 대신 아무 환경에서나 재현된다.

이 축을 게이트로 만든 것이 `tests/cases/oracle_page_count_baseline.rs` 다.

## 2. 정답지 꺼내기 — sparse-checkout 에 안 보인다

`pdf/` 는 sparse-checkout 대상이 아니라 작업 트리에는 없지만 오브젝트에는 있다.

```bash
git -c core.quotePath=false ls-tree -r HEAD --name-only pdf/     # 573개 목록
git show "HEAD:pdf/<파일>.pdf" > /tmp/oracle.pdf
```

sparse 설정을 건드릴 필요가 없다.

## 3. 문서와 정답지 짝짓기

정답지 파일명은 `<이름>[-접미사].pdf` 이고 접미사는 한글 버전·폰트 조건이다
(`-2022`, `-2020-kopub`, `-no-ttf`, `-current` 등). 샘플은 `samples/**/<이름>.hwp|.hwpx` 다.

```
samples/basic/sungeo.hwp            <->  pdf/basic/sungeo-2022.pdf
2025 행정업무운영 편람(최종).hwp     <->  pdf/…-2010-kopub.pdf, -2020-kopub.pdf, -2024.pdf
```

### 디렉터리까지 봐야 한다 — 이름만 보면 다른 문서를 집는다

저장소에는 **같은 이름의 서로 다른 문서가 44 종** 있다.

```
samples/KTX.hwp        27쪽  「AI-반도체 해외실증 지원 사업 공모 안내서」
samples/basic/KTX.hwp   1쪽  실제 KTX 노선도
```

파일명만으로 짝지으면 둘이 `pdf/KTX-2022.pdf`(27 쪽)와 `pdf/basic/KTX-2022.pdf`(1 쪽)를
**함께** 후보로 갖는다. 그러면 각자 상대의 쪽수로도 "일치" 판정을 받아 **진짜 불일치가
가려진다.**

같은 디렉터리의 정답지가 있으면 그것만 쓰고, 없으면 이름 후보를 그대로 쓴다 — 정답지가
`pdf/` 최상위에만 있는 문서가 많다. 정답지 후보가 2 개 이상인 샘플은 134 개이고 그중
96 개가 디렉터리로 좁혀진다(편람은 후보 10 개 중 7 개만 자기 것이다).

**같은 문서에 정답지가 여럿이면 쪽수의 집합으로 다룬다.** 그중 하나와 맞으면 일치로 본다 —
한글 버전 차이를 결함으로 오인하지 않기 위한 보수적 판정이다. 편람은 정답지가 383·388·389 로
셋 다 다르다.

실측(2026-08-29, devel `f6a6bee8f3`): 짝지어진 샘플 **562 개**.

## 4. 반드시 걸러야 하는 것 — 모아 찍기

`print_method` 가 모아 찍기(4·5)면 한글이 **한 장에 여러 쪽을 실어** 뽑으므로 장 수가 애초에
다르다. 거르지 않으면 정상 문서가 불일치로 잡힌다.

```bash
rhwp info "<문서>" --json | jq '{pageCount, printMethod, printMethodImpliesNup}'
```

Rust 에서는 `model::document::print_method_implies_nup(doc.doc_info.print_method)` 다.
값별 실측표는 그 함수 주석에 있다(#6208 / #6268).

### 간접 신호로 추측하지 말 것 — 실제로 두 번 틀렸다

`oracle_page_count_baseline` 픽스처를 만들며 겪은 오판이다.

**1차 — 정답지 쪽수를 그대로 비교.** 최악 사례가 `issue5866/memo_field_hwp5.hwp`
(정답지 20, rhwp 40)였는데, 정답지가 A4 **가로** 841x595 이고 한 장에 `- 39 -`·`- 40 -` 가
함께 있었다. 모아 찍기였고 **rhwp 가 맞았다.**

**2차 — "시트 = ceil(쪽수/N)" 규칙으로 보정.** 이번에는 **세로로 뽑힌 정답지까지 삼켰다.**
`hwpx/hancom-hwp/hwpx-02.hwp` 는 정답지가 A4 세로 5 쪽인데 `ceil(9/2)=5` 라서 2-up 으로
오인돼 통과했다 — **차 4 짜리 진짜 불일치가 필터에 가려졌다.**

**3차 — 문서가 선언한 `print_method` 만 신뢰.** 오탐과 오음성이 동시에 사라졌다.

용지 방향·쪽번호 표기 같은 간접 신호는 두 방향 모두로 틀린다. 문서의 선언을 쓴다.

## 5. 쪽수 말고도 볼 수 있는 것

정답지는 텍스트도 갖고 있으므로 **무엇이 그려졌는가**를 판정할 수 있다. 쪽수가 같아도
내용이 다를 수 있고, 반대로 쪽수가 달라도 내용은 같을 수 있다.

```python
import pypdfium2 as pdfium
d = pdfium.PdfDocument("oracle.pdf")
print(d[3].get_textpage().get_text_range())
```

실사용 예 두 가지다.

**(a) 있어야 할 것이 없는가** — `pdf/exam_science-2022.pdf` 4쪽의 바탕쪽 글자는
`32 32`·`* 확인 사항` 뿐이고 기본 짝수 바탕쪽의 `31` 이 **없다.** rhwp 가 `31` 을 함께
그리면 바탕쪽을 두 겹 그린 것이다(#6334). 정답지에 없는 글자가 나타나는지를 회귀 시험의
단언으로 쓸 수 있다.

**(b) 쪽수 차이가 내용 손실인가 조판 밀도인가** — `basic/sungeo.hwp` 는 정답지 94 쪽,
rhwp 86 쪽으로 8 쪽 차이인데, 텍스트 전체를 공백 제거하고 대조하면 정답지 48,614 자 대
rhwp 48,600 자로 **거의 같다.** 글자가 사라진 게 아니라 한 쪽에 더 많이 담은 것이다.
실제로 그 문서는 7 개 쪽이 본문 높이를 최대 35.4px 넘겨 채운다.

## 6. 판정 순서

1. `rhwp info --json` 으로 쪽수와 `printMethodImpliesNup` 을 본다. 모아 찍기면 쪽수 대조에서
   제외한다.
2. 정답지 쪽수와 대조한다. 여러 정답지가 있으면 집합으로 본다.
3. 다르면 **내용부터 대조한다** — 텍스트 총량이 비슷하면 조판 밀도, 크게 빠지면 내용 손실이다.
4. `rhwp dump-pages` 로 어느 쪽에서 갈리는지 좁힌다. `used` 가 `body_area h` 를 넘으면
   그 쪽이 과적재다.
5. `rhwp layout-anomaly --json` 으로 그 쪽의 기하 신호(overflow·off-canvas·overlap·
   text-overlap)를 본다.

## 7. 이 축을 지키는 게이트

| 게이트 | 무엇을 고정하나 |
| --- | --- |
| `tests/cases/oracle_page_count_baseline.rs` | 정답지 대비 쪽수 (555 문서 / 일치 538) |
| `tests/cases/text_overlap_baseline.rs` | 글자끼리 겹쳐 못 읽는 것 |
| `tests/cases/off_canvas_baseline.rs` | 종이 밖에 그려져 안 보이는 것 |

픽스처 재생성은 `tools/oracle_page_count/regenerate.py`(`pypdfium2` 필요)다. Rust 시험은
만들어진 TSV 만 읽는다 — PDF 파서 의존을 들이지 않기 위해서다.

## 8. 한계

- 코퍼스가 저장소에 있는 것으로 고정된다. 새 문서로 정답을 만들려면 한컴 설치본이 필요하고,
  그건 [한글 페이지 충실도 오라클](hangul_page_oracle.md) 의 몫이다.
- 정답지를 뽑은 한글 버전이 파일명 접미사로만 표시된다. 버전별 차이를 정밀하게 다루려면
  [한글 버전별 대조](hangul_version_oracle.md) 를 본다.
- 조판이 실행 환경에 따라 갈리는 문서가 있다(#6325). 정답지 대조 결과가 로컬과 CI 에서
  다를 수 있으므로, 게이트의 기준선은 어느 환경에서 뽑았는지 함께 기록한다.
