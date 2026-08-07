# task4097 한컴 판정 PDF — 읽기 전에

**이 PDF 들은 "정상 렌더 증명"이 아니라 대조 실험 증거다.** 변환본 두 개는 열면 사실상
백지인데, 그것은 이 판정과 **무관한 별개 결함**
([#4141](https://github.com/edwardkim/rhwp/issues/4141) — `relative_sizes=0` 직렬화, 이 판정
과정에서 발견해 등록) 때문이다.

| 파일 | 무엇인가 |
|---|---|
| `SO-SUEOP.pdf` | 원본 `samples/SO-SUEOP.hwp` 를 한컴에서 직접 PDF 출력 (46쪽, 정상) |
| `SO-SUEOP-before-hwp.pdf` | **#4097 수정 전** rhwp HWP3→HWP5 변환본의 한컴 PDF 출력 |
| `SO-SUEOP-after-hwp.pdf` | **#4097 수정 후** 변환본의 한컴 PDF 출력. before 와의 차이는 원본 `.hwp` 기준 **중첩 CFB 루트 CLSID 16바이트뿐** |

## 이 자료가 증명하는 것 (정확히 한 문장)

> CLSID 16바이트만 다른 두 변환본이 한컴에서 **46쪽 전부 픽셀 동일**하게 렌더된다
> (PyMuPDF 72dpi 픽스맵 SHA-256, 다른 쪽 0) — 즉 **CLSID 는 HWP3 글맵시 축의 렌더에 가시
> 효과가 없다**("둘 다 정상" 분기, `mydocs/working/task_m100_4097_stage5.md` §2.4).

before/after 가 **같은 결함(#4141)으로 똑같이 깨져 있으므로** 이 비교가 격리하는 변수는 CLSID
하나다 — 백지 상태가 대조 실험의 유효성을 해치지 않는다.

## 이 자료가 증명하지 않는 것

- "HWP3 변환본이 한컴에서 정상 동작한다" — #4141 해소 전까지 어떤 자료로도 증명 불가.
- **#4097 본체(차트 축)의 정상 동작** — 그 증거는 이 PDF 가 아니라
  `tests/issue_4097_mini_cfb_root_clsid.rs` 의 바이트 동일성 테스트(CI 실행)와 2026-08-05 한컴
  판정 기록(`mydocs/working/task_m100_4055_stage4.md`, `mydocs/orders/20260806.md`)이다.

## SHA-256

```text
78a4ca30f870e744aa13701c45d269dbe5c9d3741d3f3636311f20cfc945c629  SO-SUEOP.pdf
ee0a4be9ca20391fdc114f6ad0c2ac144b38609f61b48dd95241e409eb6e97cd  SO-SUEOP-before-hwp.pdf
7621bf3da6a84db2f144858c565a3907bd0d86ed189bfbdb56145009bd613537  SO-SUEOP-after-hwp.pdf
```

#4141 해소 후 같은 쌍을 다시 만들면 이 축의 **양성 판정**(글맵시 정상 표시·더블클릭 거동)이
비로소 가능해진다 — 그때 이 디렉터리를 갱신한다.
