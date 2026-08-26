# pdf/issue5447 — #5447 B2 판정 한컴 2022 변환본, 읽기 전에

**이 PDF 38건은 "정상 렌더 증명" 이 아니다.** 그 중 둘은 **한컴이 잘못 그린 그림**이고,
그것이 이 자료가 존재하는 이유다. [#5447](https://github.com/edwardkim/rhwp/issues/5447)
B2 스파이크는 "한컴이 우리 편집을 받아 주는가" 뿐 아니라 **"한컴이 막아 주지 않는 편집은
무엇인가"** 를 물었고, 후자의 답이 아래 경계 2건이다.

원본(판정 입력)은 [`samples/issue5447/`](../../samples/issue5447/), 판정 원장은
[`samples/issue5447/MANIFEST.json`](../../samples/issue5447/MANIFEST.json),
결론은 [`mydocs/report/task_m100_5447_report.md`](../../mydocs/report/task_m100_5447_report.md) 다.

## 이 자료가 증명하는 것

> `c:f` 참조 범위를 **갱신하지 않은 채** 행·계열 개수를 바꾼 문서를 한글 2022 가 정상적으로
> 열고, **바뀐 개수대로 그리며**, 데이터 편집기도 바뀐 행 수를 보여 준다.
> 15개 판정 단위 중 14가 대조군 대비 렌더가 바뀌었고, 같은 변종의 `.hwp` 와 `.hwpx` 는
> **15/15 픽셀 동일**하게 그려진다.

여기서 `c:f` 무갱신 정책이 확정됐다 — B2 본구현은 A1 범위 파서도 열 자리올림(`$Z`→`$AA`)도
만들지 않는다.

## 이 자료가 증명하지 않는 것 — 경계 2건

| 파일 | 한컴의 반응 | 왜 문제인가 |
|---|---|---|
| `원형대원형-계열추가-*` | **대조군과 래스터 완전 동일**(`e16a2a67…`) | ofPie 는 2번째 계열을 그리지 않는다. 오류도 경고도 없다 |
| `시가고가저가종가-계열삭제-*` | 렌더는 바뀐다 | 범례는 4→3 인데 `c:upDownBars` 캔들 장치가 남아 고가·저가를 몸통 삼아 전부 검은 박스로 그린다. HLC 로 정상 전환된 것이 아니다 |

둘 다 **오류 없이 열린다.** 그래서 "열렸다 = 통과" 가 아니고, **B2 엔진이 fail-closed 로
막아야 한다**는 것이 이 PDF 들이 내놓는 결론이다. 대표 시각 대조는
[`mydocs/pr/assets/pr_5647_issue5447_boundary_pie_series_add.png`](../../mydocs/pr/assets/pr_5647_issue5447_boundary_pie_series_add.png) ·
[`…_boundary_stock_series_delete.png`](../../mydocs/pr/assets/pr_5647_issue5447_boundary_stock_series_delete.png).

## 명명 규약

상위 [`pdf/README.md`](../README.md) §2 의 `{원본 stem}-2022.pdf` 에, 같은 stem 이 두 포맷으로
존재하므로 `pdf/issue_4055_b1_spike/` 의 포맷 구분자 선례(`00-control-hwp-2020.pdf`)를 합쳤다.

```text
{원본 stem}-hwp-2022.pdf     ← samples/issue5447/{원본 stem}.hwp   의 변환본
{원본 stem}-hwpx-2022.pdf    ← samples/issue5447/{원본 stem}.hwpx  의 변환본
```

38 원본 ↔ 38 PDF 가 1:1 이다. 어느 PDF 가 어느 원본에서 왔는지는 원장의
`original_path` / `hancom_pdf_path` 쌍이 정본이다.

## 래스터 판정 — 스트림 해시를 쓰지 않는 이유

PDF 안 스트림을 통째로 해시하면 폰트·구조처럼 **문서 출처가 다른 데서 오는 차이**가 섞여
판정이 틀린다. [#4100 보고서 §4-1](../../mydocs/report/task_m100_4100_report.md) 에서 실제로
한 번 틀렸고 래스터로 다시 재서 바로잡았다. **그리기 결과를 판정하려면 그리기 결과를 잰다.**

원장은 래스터를 두 축으로 적는다. **절대 해시는 렌더러와 그 버전에 딸린 값이므로 도구를
섞어 비교하지 않는다.** 도구를 넘어 성립해야 하는 계약은 원장의 `invariants` 다.

| 축 | 도구 | 비고 |
|---|---|---|
| `pymupdf_144dpi_rgb_sha256` | PyMuPDF 1쪽 144dpi RGB 픽스맵 | 추가 설치 없이 도는 기본 축. `pdf/task4097/` 선례와 같은 방식 |
| `pdftoppm_144dpi_ppm_sha256` | poppler `pdftoppm -r 144 -f 1 -l 1` | **보고서 §2 표의 축.** poppler 25.12.0 재계산으로 15개 판정 단위 전건 일치 확인(2026-08-20) |

```bash
python tools/hancom_chart_judgment_verify.py                       # PyMuPDF 축
python tools/hancom_chart_judgment_verify.py --rasterizer pdftoppm # poppler 축

# poppler 가 없으면 컨테이너로 같은 값을 낼 수 있다
docker run --rm -v "$PWD/pdf/issue5447:/w:ro" minidocks/poppler sh -c \
  'cd /w; for f in *.pdf; do pdftoppm -r 144 -f 1 -l 1 "$f" /tmp/o; sha256sum /tmp/o-1.ppm; rm -f /tmp/o-1.ppm; done'
```

## SHA-256

파일별 전체 해시는 [`samples/issue5447/MANIFEST.json`](../../samples/issue5447/MANIFEST.json) 의
`hancom_pdf_sha256` 이 정본이다. README 에 복사본을 두지 않는다 — 두 곳에 적으면 한 곳이
조용히 늙고, 그 조용함이 #5447 이 고치려는 실패 모드다.

```bash
python -c "import json;print('\n'.join(f\"{e['hancom_pdf_sha256']}  {e['hancom_pdf_path']}\" for e in json.load(open('samples/issue5447/MANIFEST.json',encoding='utf-8'))['entries']))"
```

## 용량과 LFS

38 PDF 합계 **3.54 MB**, 최대 단일 파일 113,064 B(110 KB). `.gitattributes` 의 LFS 추적 대상은
`pdf-large/**/*.pdf` 뿐이고 임계는 50 MB 이므로 **이 폴더는 LFS 대상이 아니다** — 일반 git
으로 보존한다([`pdf-large/README.md`](../../pdf-large/README.md) 사용 규칙).

## 재생성

한컴 변환은 사람 손이 필요하다. 원본을 다시 만드는 절차는
[`samples/issue5447/README.md`](../../samples/issue5447/README.md) 에, 변환 자동화 스크립트는
상위 [`pdf/README.md`](../README.md) §4 에 있다.
