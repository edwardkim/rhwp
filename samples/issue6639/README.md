# #6639 / PR #7260 검토 입력과 기준 출력

검토 코드 SHA: `e5135e3f19ff26222254f1474adc87d2af04e870`.
원본은 [이슈 첨부 ZIP](https://github.com/user-attachments/files/31736768/rhwp-table-cell-minimal-repro.zip)의
`rhwp-table-cell-minimal-repro.hwp`이며, 첨부와 SHA-256이 같다. 이 자료는 reviewer가 보완한 것으로
기여자의 구현 변경과 구분한다. 시각 일치 통과나 새로운 제품 회귀를 선언하는 자료가 아니다.

## 파일 역할과 생성 경로

| 파일 | 역할 |
| --- | --- |
| [원본 HWP](rhwp-table-cell-minimal-repro.hwp) | 이슈 원본 160%, 변형 없음 |
| [한컴 HWPX](issue6639-hancom-160.hwpx) | 원본 HWP를 한컴에서 HWPX로 저장한 중간 입력 |
| [160% 대조 입력](issue6639-reference-input-160.hwpx) | 대상 10문단의 저장 줄 캐시만 제거한 대조군 |
| [140% 기준 입력](issue6639-reference-input-140.hwpx) | 동일 캐시 제거 후 대상 줄간격 속성만 140%로 변경 |
| [원본 기준 PDF](../../pdf/issue6639/issue6639-original-160-2020.pdf) | 원본 HWP를 한컴에서 직접 출력 |
| [160% 대조 PDF](../../pdf/issue6639/issue6639-reference-160-2020.pdf) | 캐시 제거·HWPX 변환 영향 확인 |
| [140% 기준 PDF](../../pdf/issue6639/issue6639-reference-140-2020.pdf) | 독립 속성 변경 후 한컴 재조판 출력, GUI 편집본은 아님 |

공식 client 0.9.0을 `npx`로 호출하고 비동기 `start → status → download`로 처리했다.
원본은 `info --json`의 `format=hwp5`, `lastSavedWith=null`이다. 파생 HWPX는
`format=hwpx`, `lastSavedWith.product=hancom-office-2020`, `version=11.0.0.9136`이다.
모두 `--engine 2020`을 명시했으며 client/server 크기·SHA-256 검증을 통과했다.

| 산출 | job ID |
| --- | --- |
| 원본 PDF | `e017f44b-0ce7-4914-b845-e0572dbed989` |
| 한컴 HWPX | `02e9ae3e-ea30-4f03-9334-54fda86b5a19` |
| 160% 대조 PDF | `435bbbb7-4e5c-413d-93fb-866f8950221b` |
| 140% 기준 PDF | `75fefc7a-10d1-4cb3-a4db-e296e834d832` |

서버 보고값은 Hancom `11.0.0.9136`, backend `hwp-managed-direct-dll-host`,
PDF mode `hancom2020_pdf_driver_one_up`, input_preprocess `none`, 등록 글꼴 2개/실패 0개다.
PDF 3개 모두 Creator `Hwp 2020 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`, PDF 1.4,
1쪽 A4(595 × 841 pt)이며 실제 이미지를 열어 확인했다. 버전 표기를 출력 결함으로 간주하지 않는다.
서버 주소·인증정보·환경 파일은 보존하지 않는다. MCP 접근 권한이 없는 기여자는 제공된 PDF를 사용한다.

## 파생 입력의 정확한 변경

한컴 HWPX의 `Contents/section0.xml`에서 셀 31의 10문단만 `paraPrIDRef=18`을 사용한다.
160%/140% 파생 입력 모두 이 10문단의 `hp:linesegarray`만 제거해 한컴이 다시 조판하게 했다.
140% 입력은 추가로 `Contents/header.xml`의 paraPr 18에서 case/default 양쪽
`hh:lineSpacing type=PERCENT` 값을 160에서 140으로 바꿨다. 다른 ZIP 멤버 내용은 보존했다.
rhwp가 계산한 좌표를 기준값으로 넣지 않았다. 이 변형은 정상 GUI 편집을 수행했다는 증거는 아니다.

160% 대조 PDF는 원본 PDF와 96dpi raster가 픽셀 단위로 완전히 같았다. 이 입력에서 해당 변형 경로의
무변경 대조를 확인한 것이며, 140% 상태나 모든 문서에서의 동등성까지 일반화하지 않는다.
PDF 해시는 생성 시각 등이 달라 서로 다르므로 두 산출을 각각 보존했다.

재생성하려면 다음 코드를 저장소 루트에서 실행한다(동일 파일이 이미 있으면 덮어쓰지 않는다).

```python
from pathlib import Path
import re, zipfile
root = Path('samples/issue6639')
with zipfile.ZipFile(root / 'issue6639-hancom-160.hwpx') as z:
    infos = z.infolist()
    data = {i.filename: z.read(i) for i in infos}
section = data['Contents/section0.xml'].decode()
header = data['Contents/header.xml'].decode()
assert len(re.findall(r'paraPrIDRef="18"', section)) == 10
removed = 0
def clear_cache(match):
    global removed
    paragraph = match.group()
    if re.search(r'^<hp:p\b[^>]*\bparaPrIDRef="18"', paragraph):
        paragraph, n = re.subn(r'<hp:linesegarray>.*?</hp:linesegarray>', '', paragraph)
        assert n == 1
        removed += 1
    return paragraph
section = re.sub(r'<hp:p\b[^>]*>.*?</hp:p>', clear_cache, section)
assert removed == 10
shape = re.search(r'<hh:paraPr\b[^>]*\bid="18".*?</hh:paraPr>', header)
shape140, n = re.subn(r'(<hh:lineSpacing\b[^>]*\bvalue=")160("[^>]*>)',
                     r'\g<1>140\2', shape.group())
assert n == 2
for spacing in [160, 140]:
    output = root / f'issue6639-reference-input-{spacing}.hwpx'
    assert not output.exists(), output
    changed = dict(data)
    changed['Contents/section0.xml'] = section.encode()
    changed['Contents/header.xml'] = (header if spacing == 160 else
        header[:shape.start()] + shape140 + header[shape.end():]).encode()
    with zipfile.ZipFile(output, 'w') as z:
        for info in infos:
            z.writestr(info, changed[info.filename])
```

## 직접 비교와 한계

원본 160%의 Native/fresh WASM Visual Sweep은 각각 1쪽의 compare·overlay·review를 산출했다.
자동 flagged page는 0이지만 직접 판독에서 문단 줄바꿈과 표 가로선 위치 차이를 확인했다.
Native pixel match 91.15192%, 내용 중심 visual accuracy proxy 8.54662%다.
실제 140% 편집의 Native/fresh WASM SVG는 byte 단위로 동일하다(SHA-256
`fa0ff3a16e57fbc06ed5c64528b93597cba9e65e5f869601bcbec5447c56758a`).
이 SVG에 동일 입력의 font-face 정책을 공급하고 canonical Visual Sweep의 compare/overlay/review
함수로 독립 140% PDF와 비교했다. pixel match 93.04333%, proxy 14.46969%다.
편집 상태의 render-tree heuristic은 실행하지 않았으므로 완전한 편집 상태 sweep이라고 쓰지 않는다.
수치·자동 후보 0건은 시각 통과 근거가 아니며, 줄바꿈·표 높이 차이는 직접 판독했다.

- [160% Native 비교](../../mydocs/pr/assets/pr_7260_original160_native_review.png)
- [160% fresh WASM 비교](../../mydocs/pr/assets/pr_7260_original160_wasm_review.png)
- [140% 실제 편집 비교](../../mydocs/pr/assets/pr_7260_edited140_native_wasm_review.png)

대표 PNG만 공개 증적으로 포함한다. 중간 SVG·로그·점수 JSON은 커밋하지 않는다.
기준선·허용치를 바꾸지 않았다. 원본 HWP의 SHA-1은 `440f7f15266051192eeae1318dfbfa39ddcf5e20`이다.
PDF SHA-1은 순서대로 원본 `b97cf2e9ffd41fe254d92f10eba07da62a3b7036`,
160% 대조 `11e1de122bbe6e2194af5df24079a34986da92b3`,
140% 기준 `dc8c6230d95601890a527be9ff88424851c481b7`이다.

| 저장소 경로 | Bytes | SHA-256 |
| --- | --- | --- |
| `samples/issue6639/issue6639-hancom-160.hwpx` | 25342 | `a9ece196dfdeee718b04c95494b0f4868cfe5eb58b3e61cd53df371e7d57504b` |
| `samples/issue6639/issue6639-reference-input-140.hwpx` | 23715 | `c13bc0f9b86235a32181eef53d5a0c04a61faa78ceb85636570f4fa17ee126e7` |
| `samples/issue6639/issue6639-reference-input-160.hwpx` | 23714 | `6430caae3eb4fea53ffcbe049b8ff525e1f42386bb4a42adce5a7f43c081a0c1` |
| `samples/issue6639/rhwp-table-cell-minimal-repro.hwp` | 6656 | `983df661a2316881457ee4604c3084895bd4f6b350df4953c6c53cca8c162801` |
| `pdf/issue6639/issue6639-original-160-2020.pdf` | 20828 | `ebd1cd8ef64741c648211db9174cae7118e59fcbc49fba39b57ab5c63b2e9bc0` |
| `pdf/issue6639/issue6639-reference-140-2020.pdf` | 20831 | `0aff1a33d8d97a0adf0bd329bf173dab42e617f88a2549068d16e247bda7bc97` |
| `pdf/issue6639/issue6639-reference-160-2020.pdf` | 20828 | `a17453b03acc4a4684e23f8843fe2e4d9baedaeb5b6c12eb319bf393344c77ac` |
