## Windows 제공 글꼴 19개를 추가한 전쪽 조사 결과

전쪽 18개 원본, Native/fresh WASM 각각 1,771쪽을 측정했다. 아래 값은 다른 글꼴 환경의 실측이다.
정본 보호 TSV의 비공개 원본 글꼴 공급 조건을 재현한 결과가 아니며 기존 보호 하한을 갱신하지 않는다.
Desktop 글꼴 17개와 한국어 추가 TTC 2개의 파일명·SHA는 [글꼴 목록](os-stock-fonts.json), 입력/PDF·전체 TSV 해시는
[전쪽 증적 목록](os-stock-evidence.json)에 보존한다. 글꼴 프로그램은 첨부하지 않는다.

| 원본 | 쪽수 | Native 최저 | WASM 최저 | 90% 미만 쪽 수 N/W |
| --- | ---: | ---: | ---: | ---: |
| `20099369_yeongwol_forms.hwp` | 4 | [99.95%](os-stock/native/0b7e7d0538284271.tsv) | [99.98%](os-stock/wasm/0b7e7d0538284271.tsv) | 0/0 |
| `body-list-typed-hwp.hwp` | 1 | [100.00%](os-stock/native/2b43ec3782a748e1.tsv) | [100.00%](os-stock/wasm/2b43ec3782a748e1.tsv) | 0/0 |
| `2025 행정업무운영 편람(최종).hwp` | 383 | [93.23%](os-stock/native/2fb2575d58042dcf.tsv) | [88.72%](os-stock/wasm/2fb2575d58042dcf.tsv) | 0/1 |
| `control-styles-hwp.hwp` | 1 | [100.00%](os-stock/native/4314830495cfde87.tsv) | [100.00%](os-stock/wasm/4314830495cfde87.tsv) | 0/0 |
| `hwp3-sample16-hwp5.hwpx` | 64 | [81.66%](os-stock/native/43c0a207f6daa0bb.tsv) | [96.85%](os-stock/wasm/43c0a207f6daa0bb.tsv) | 1/0 |
| `hwp3-sample16-hwp5-2022.hwp` | 64 | [51.00%](os-stock/native/4bac5a5d1702df5f.tsv) | [50.89%](os-stock/wasm/4bac5a5d1702df5f.tsv) | 2/2 |
| `re-03-latin-only-hancom.hwp` | 1 | [100.00%](os-stock/native/52ee61fe7201e36b.tsv) | [100.00%](os-stock/wasm/52ee61fe7201e36b.tsv) | 0/0 |
| `angle-2400.hwpx` | 1 | [100.00%](os-stock/native/59f6ab1155623bda.tsv) | [100.00%](os-stock/wasm/59f6ab1155623bda.tsv) | 0/0 |
| `issue3307_outline_number.hwpx` | 9 | [95.02%](os-stock/native/69bcf084e2ae31a5.tsv) | [99.23%](os-stock/wasm/69bcf084e2ae31a5.tsv) | 0/0 |
| `exam-social-insert.hwp` | 4 | [99.03%](os-stock/native/71f98206e45c354a.tsv) | [99.02%](os-stock/wasm/71f98206e45c354a.tsv) | 0/0 |
| `issue2007_nested_cell_pagination_42065.hwp` | 17 | [90.03%](os-stock/native/78e1720edfe2277d.tsv) | [87.75%](os-stock/wasm/78e1720edfe2277d.tsv) | 0/1 |
| `hwp3-sample16-hwp5-2024.hwp` | 64 | [81.68%](os-stock/native/7df4af8174abd82a.tsv) | [96.85%](os-stock/wasm/7df4af8174abd82a.tsv) | 1/0 |
| `2025 행정업무운영 편람(최종).hwpx` | 383 | [93.23%](os-stock/native/941fe0c3549c0017.tsv) | [88.72%](os-stock/wasm/941fe0c3549c0017.tsv) | 0/1 |
| `jongbu_law.hwpx` | 6 | [40.74%](os-stock/native/a8e95b4accc79fa8.tsv) | [98.30%](os-stock/wasm/a8e95b4accc79fa8.tsv) | 3/0 |
| `hwp3-sample10.hwp` | 763 | [81.92%](os-stock/native/b1a9055c4c7e6f09.tsv) | [79.84%](os-stock/wasm/b1a9055c4c7e6f09.tsv) | 32/42 |
| `exam-social-insert-delete.hwp` | 4 | [99.03%](os-stock/native/ccd0cd0b7e76fe2f.tsv) | [99.02%](os-stock/wasm/ccd0cd0b7e76fe2f.tsv) | 0/0 |
| `pic-crop-01.hwp` | 1 | [100.00%](os-stock/native/cdacd874b6e6ac66.tsv) | [100.00%](os-stock/wasm/cdacd874b6e6ac66.tsv) | 0/0 |
| `shape-group-02.hwp` | 1 | [100.00%](os-stock/native/d03a9dde3d5dc7ac.tsv) | [100.00%](os-stock/wasm/d03a9dde3d5dc7ac.tsv) | 0/0 |

Native/fresh WASM의 전체 쪽수는 독립 PDF와 모두 일치했다. 영역 마스킹·쪽 제외는 사용하지 않았다.
90% 미달이 있는 환경은 통과로 판정하지 않는다. 특히 법령·중첩 표·sample16 등의 backend 차이는
OS 기본 글꼴 보유만으로 정본 전체 환경을 재현했다고 볼 수 없음을 보여준다.
편람의 일반 실행 결과는 전용 `--handbook-gate`의 KoPub 웹폰트 사전 공급을 실행한 결과가 아니다.
따라서 이 표로 기존 편람 gate의 통과·실패나 WMF 130쪽 보호 기준을 바꾸지 않는다.
기존 보호 검증에 필요한 별도 원본 글꼴·명시 환경은 통합 담당자의 책임으로 계속 유지한다.

이 실험의 19개 파일은 Windows 기본 글꼴 전체가 아니다. 이 결과만으로 전체 Windows 기본 글꼴 환경이나 macOS의 전쪽 재현 가능성을 단정하지 않는다.
