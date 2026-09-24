# #6639 / PR #7260 검토 입력과 기준 출력

아래 reviewer 자료의 검토 코드 SHA: `e5135e3f19ff26222254f1474adc87d2af04e870`.
기여자의 후속 수정·검증은 문서 끝의 「2026-09-24 재리뷰 대응」에 구분해 기록한다.
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

## 2026-09-24 재리뷰 대응

제품·테스트 수정 SHA: `cf3e9fc1da3ba9cc949eb94ae6ddfb61f9baba5a`.
PR 도입 전 비교는 첫 PR 커밋의 부모 `236a601da803b53429e9090eef652c661dd3bfe2`다.
두 버전을 각각 빌드해 같은 원본의 `(0,0,2,31)`에 140%를 적용하고 원래 모양 ID를 복원했다.
reviewer의 입력·기준 PDF·검토 기록은 그대로 보존했다.

### 반복 편집의 RowBreak 경계

수치 역행이 사라지는 상태는 합성만의 문제가 아니다. 기존 입력
`samples/task2430/1382000_domestic_violence_survey.hwp`의 `(0,93,0,0)`은 실제 `RowBreak`
표이며, 저장된 문단 76/77의 첫 vpos가 `64680 / 64462`다(index는 0부터).
109문단을 140%로 바꾸면 문단 76이 64462보다 앞서 역행이 사라진다. 원래 모양 ID를
복원할 때 종전 코드는 문단 77을 `69080`으로 옮겼고 수정본은 저장 원점 `64462`를 유지한다.
이 입력의 SHA-256은 `a3c6a227d26c41c7de9aa258f470001a629da90fa606cdddcbd385add43b7381`이며,
기존 한컴 출력은 `pdf/issue2430/1382000_domestic_violence_survey-2020-print.pdf`다.
이는 실제 지원 입력의 경계 보존 검증이며, 그 문서의 140% 한컴 편집 출력과의 시각 일치 판정은 아니다.

최초 서식 flush가 좌표를 바꾸기 전에 문단의 경계 여부를 보존한다. 이후에는 숫자 역행을
재추론하지 않는다. 기존 ladder와 batch 종료 순회를 재사용하고 새 의존성·문서별 임계값은 없다.
snapshot은 경계를 보존하며 문단 분할의 새 문단은 이어지는 조각이다. 폭 reflow는 기존 좌표계를
다시 만들므로 경계를 해제한다. setter마다 셀 전체를 순회하는 작업은 추가하지 않았다.

정식 회귀 `rowbreak_origin_survives_shrink_then_restore`와
`saved_rowbreak_origin_survives_repeated_formatting`을 추가했다. 전자는 합성 입력의
batch/eager, 두 번의 축소·복원, snapshot 왕복 및 경계 첫 문단의 텍스트 입력/삭제를 검사한다.
후자는 위 실제 HWP의 두 번의 축소·복원을 검사한다. 수정 전 제품 코드 `e5135e3f`에 연결한
두 검사는 모두 FAIL, 기존 RowBreak reflow 대조 검사는 PASS였다. 수정 후 #6639는 10/10 PASS다.

### 원본의 편집·복원과 남는 화면 차이

아래는 96dpi SVG 좌표다. 초기 화면과 원래 모양 ID 복원 화면은 각각 **PR 도입 전/수정 후
SVG 전체가 byte 동일**하다. 140%에서는 756개 XML 노드 중 글자 622개의 위치만 달라지고
문자열·줄 구성·셀/표 외곽은 같다. 첫 문단을 제외한 문단 시작점이 함께 위로 이동한다.

| 상태 | 표 바탕 높이 | 대상 셀 Y / 높이 | 마지막 글줄 baseline Y |
| --- | ---: | ---: | ---: |
| 초기 160%, 양쪽 동일 | 1009.1200 | 328.1237 / 540.6004 | 858.5904 |
| 도입 전 140% | 1024.5737 | 341.2933 / 540.6004 | 869.3600 |
| 수정 후 140% | 1024.5737 | 341.2933 / 540.6004 | 806.9600 |
| 원래 모양 ID 복원, 양쪽 동일 | 1025.3067 | 341.2933 / 541.3333 | 871.7600 |
| 전체 snapshot 복원, 양쪽 동일 | 1009.1200 | 328.1237 / 540.6004 | 858.5904 |

140%의 문단 첫 vpos는 도입 전 `[0,1440,8640,18720,27360,31680,33120,34560,36000,37440]`,
수정 후 `[0,1260,7560,16380,23940,27720,28980,30240,31500,32760]`다.
마지막 두 줄은 `[37440,38700]`에서 `[32760,34020]`으로 바뀐다. 화면의 마지막 글줄이
62.4px 위로 이동하며 마지막 문단의 글자는 유지된다. 이 입력에는 표 뒤의 가시적인 본문이
없어 하단 빈 행·표 외곽 위치를 비교했다. 별도 후속 본문의 정상 배치까지 검증한 것은 아니다.

원본의 저장/로드 줄 수는 문단별 `[1,5,7,6,3,1,1,1,1,2]`(28줄)이고 제공된 한컴 PDF는
`[1,4,6,5,3,1,1,1,1,2]`(25줄)다. 추가 줄바꿈은 초기 상태와 PR 도입 전부터 있다.
글꼴 계측의 개별 원인은 분리하지 않았으므로 특정 폰트 결함으로 단정하지 않는다.

모양 ID 복원이 최초 표 높이까지 복원하지 않는 현상도 기존 코드와 같다. 서식 편집은
`raw_stream`을 비우며, `Document::layout_profile`의 `session_edited`가 켜진 뒤에는
`HeightMeasurer`가 미편집 TAC 표에 적용하는 저장 높이 축소를 사용하지 않는다.
모양 ID만 돌려도 그 편집 상태는 남는다. 전체 문서 snapshot을 복원하면 초기 SVG와 같았다.
이 PR의 복원 주장은 **원래 모양 ID와 셀 문단 vpos**이며, Studio undo stack/화면 전체의
픽셀 복원이나 한컴 출력 일치를 뜻하지 않는다. 독립된 높이·줄 구성 차이는 이번에 수정하지 않았다.

| SVG 상태 | 도입 전/수정 후 공통 SHA-256 |
| --- | --- |
| 초기 / snapshot 복원 | `87b0960611b6983a044acaf564559e7804e1f191f8f64baeeb5c4b3751c70801` |
| 원래 모양 ID 복원 | `bc7e098eb0928f00c3bedf61c999918c66b98f29e408ab086231eb669564a69b` |

같은 제품 SHA에서 새로 빌드한 WASM을 Chromium에서 실행했다. 초기·140%·모양 ID 복원 SVG가
각각 Native와 byte 동일하며 batch 및 비배치 복원 호출을 모두 확인했다. 실제 Studio UI의
undo 버튼 검증은 아니다. 140% SVG 해시는 위 reviewer 자료의 `fa0ff3a1…`와 같다.

- [도입 전 140% / 한컴 비교](../../mydocs/pr/assets/pr_7260_rereview_base140.png)
- [수정 후 140% fresh WASM / 한컴 비교](../../mydocs/pr/assets/pr_7260_rereview_fixed140.png)
- [수정 후 원래 모양 복원 / 한컴 비교](../../mydocs/pr/assets/pr_7260_rereview_restore160.png)
- [수정 후 원본 fresh WASM / 한컴 비교](../../mydocs/pr/assets/pr_7260_rereview_original160.png)

원본은 Native/fresh WASM 전체 Visual Sweep의 compare·overlay·review를 모두 직접 확인했다.
양쪽 1쪽 완료, 자동 flagged 0, pixel match 91.23547%, 내용 중심 proxy 7.15330%지만
추가 줄바꿈·행 경계 차이가 보이므로 시각 일치 통과로 쓰지 않는다. Windows/Chromium/webfont,
원본 PDF raster는 Poppler 26.07.0, 편집 비교는 PDFium 96dpi와 canonical 비교 helper를 썼다.
편집 비교는 render-tree heuristic 미실행이며 이전 macOS 수치와 직접 비교하지 않는다.

재현은 위 명령의 코드 checkout을 해당 SHA로 바꿔 실행한다. 원본에서 초기 SVG와 모양 ID를
저장하고 10문단을 140%로 batch 적용한 뒤 렌더한다. 저장한 ID로 `setCellParaShapeId`를 호출해
다시 렌더하고, 별도로 편집 전 snapshot을 복원해 렌더한다. 각 checkout은 자기 소스로 빌드한
산출물을 사용해야 한다. 실제 RowBreak 정식 회귀는 위 #6639 focused 명령에 포함된다.

제품 SHA `cf3e9fc1`을 별도 review worktree에서 검증했다. 전체 release-test는 **9,993 passed /
51 skipped**, #6639는 10 passed, #4118은 1 passed다. Native Skia lib는 4,112 passed /
13 ignored, 이미지 누락 2개·직접 PDF 출력 4개도 통과했다. fmt, Native/WASM32/workspace
all-targets Clippy(`-D warnings`), workspace build, manifest 계약 23개 및 정책 base
`505661360e9a2d596f55300d0cb0c5222f0e14b4` 비교가 통과했다. 원본 fidelity의 text-only /
all-SVG / layout-ledger도 완료했다. 뒤따르는 문서·PNG 커밋은 제품·테스트 코드를 바꾸지 않는다.
