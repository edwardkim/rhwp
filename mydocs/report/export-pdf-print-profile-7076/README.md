# #7076 — PDF 기본 프로필이 `Screen` 이라 빈 누름틀 안내문이 인쇄된다

재현체: `tests/fixtures/issue_7076/ship-collision-analysis-form.hwp`
(공개 행정규칙 별지 서식, 1쪽, `lastSavedWith` = hancom-office-2020)

| 이미지 | 내용 |
| --- | --- |
| `p1-before.png` | 수정 전 기본 `export-pdf` (= `--profile screen`) |
| `p1-after.png` | 수정 후 기본 `export-pdf` (= `--profile print`) |
| `p1-oracle.png` | 한/글 2020 정본 PDF (문서를 저장한 버전으로 새로 생성) |
| `p1-before-after-oracle.png` | 위 셋을 나란히 |

## 글자 멀티셋 대조 (1쪽 추출 텍스트)

```text
  정본 202자 · 수정 전 353자 · 수정 후 152자
  정본에 없는데 인쇄된 글자 : 수정 전 192자  →  수정 후 0자
```

수정 후 글자는 정본의 부분집합이다(차집합 0). 정본에만 있는 50자는 두 프로필 모두에서
같은 방향으로 빠지는 PDF 텍스트 추출(ToUnicode) 축이며, 래스터로 겹쳐 보면 수정 후는
정본과 같은 빈 서식이다 — 이 결함과 다른 축이다.

layer SVG 글자 수로도 같은 결과가 나온다: `Screen` 440자 → `Print` 202자이고,
그 202 는 정본 추출 글자 수와 같다.
