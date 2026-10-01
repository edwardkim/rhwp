# PR #7433 검토 증적

원 code head `4905a31d0746fa86e23fe03655f4538079e2d0a1`, reviewer postmelee, 2026-10-01 KST. [리뷰](../../archives/pr_7433_review.md) / [사전 보고서](../../archives/pr_7433_report.md).

## 입력·기준과 원본성

- 원본 `samples/issue7419/` 두 HWPX는 원 code head에 커밋된 합성 입력입니다. zip 실제 blob과 실행 파일의 SHA-256을 대조했습니다. metadata의 한컴 2024 표기를 실제 한컴 생성 출처의 증거로 승격하지 않습니다.
- 기준 PDF 두 개는 사용자가 승인한 그 원본 두 파일을 기존 인증된 한컴 MCP의 2024 엔진 `13.0.0.3901`에서 변환한 결과입니다. 입력 전처리 없음, 1쪽씩, 반환·다운로드 크기/SHA 일치. PDF Creator/Producer는 Hancom PDF 1.3.0.550 / PDF 1.6; source/출력 대응과 실제 내용 판독으로 채택했습니다. 인증 설정·주소·원시 응답은 공개하지 않습니다.
- 아래 `inputs/` 두 반례는 reviewer가 원본 ZIP의 section0.xml 속성 하나만 바꾼 합성 변형입니다. 텍스트·구조·다른 속성의 동일성을 XML로 검사했습니다. 실제 한컴 저장본도 아니고 한컴 변환을 실행한 입력도 아닙니다. 변형의 한컴 출력 일치나 페이지 수를 주장하지 않습니다.
- [common-height-4838.hwpx](inputs/common-height-4838.hwpx): 첫 `hp:tbl/hp:sz/@height` 4800 → 4838HU; 0.5067px 경계에서 기존 잘림이 남음.
- [page-height-28000.hwpx](inputs/page-height-28000.hwpx): `hp:pagePr/@height` 84186 → 28000HU; body 하단 259.9467px보다 자란 table 끝점이 4.64px 큼.
- [일반 기준 PDF](../../../../pdf/tac_two_line_cell_no_lineseg-2024.pdf), [중첩 기준 PDF](../../../../pdf/tac_nested_two_line_cell_no_lineseg-2024.pdf).

## 반례 재생성

원본의 ZIP entry 정보·바이트는 section0.xml 이외 모두 유지합니다. lxml 직렬화로 XML 표기만 정규화되며, semantic diff는 지정한 속성 하나뿐입니다.

```python
from pathlib import Path
from zipfile import ZipFile
from lxml import etree
source = Path('samples/issue7419/tac_two_line_cell_no_lineseg.hwpx')
ns = {'hp': 'http://www.hancom.co.kr/hwpml/2011/paragraph'}
for name, xpath, height in [
    ('common-height-4838.hwpx', '//hp:tbl/hp:sz', '4838'),
    ('page-height-28000.hwpx', '//hp:pagePr', '28000'),
]:
    with ZipFile(source) as zin, ZipFile(name, 'w') as zout:
        for info in zin.infolist():
            data = zin.read(info.filename)
            if info.filename == 'Contents/section0.xml':
                xml = etree.fromstring(data)
                xml.xpath(xpath, namespaces=ns)[0].set('height', height)
                data = etree.tostring(xml, encoding='UTF-8', xml_declaration=True)
            zout.writestr(info, data)
```

## 실제 실행과 바이너리

모든 Cargo 검증의 공유 target은 `target/pr-review`입니다. source Native binary SHA-256 `5c177fedf0d0a4e2e52e4409aae9c8520685b149b4a27c11a6a53469eaf1707a`, source WASM `b1e2909eeecdb7a631ca7e7643a45a5316ae2afc5205a7b709e011a3e386a6f6`, glue `2b7e7bb01cbbff0cb0d3c9a3222c6cb187f9d9710045013bcfb077d7abef4133`입니다.

```sh
cargo build --locked --profile release-test --target-dir target/pr-review --bin rhwp
cp target/pr-review/release-test/rhwp output/pr-review/pr7433-review-20261001/rhwp-source-4905a31d
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir output/pr-review/pr7433-review-20261001/source-wasm-pkg --no-opt
venv/bin/python scripts/visual_sweep.py --file-target two-line samples/issue7419/tac_two_line_cell_no_lineseg.hwpx pdf/tac_two_line_cell_no_lineseg-2024.pdf --file-target nested samples/issue7419/tac_nested_two_line_cell_no_lineseg.hwpx pdf/tac_nested_two_line_cell_no_lineseg-2024.pdf --rhwp-bin output/pr-review/pr7433-review-20261001/rhwp-source-4905a31d --pages 1 --dpi 96 --embed-fonts full --font-path '/Applications/Hancom Office HWP Viewer.app/Contents/Resources/Hnc/Shared/TTF/Install' --out output/pr-review/pr7433-review-20261001/source-native-sweep
```

Native build와 Sweep은 exit 0입니다. WASM Sweep은 위 Sweep에 `--wasm-pkg output/pr-review/pr7433-review-20261001/source-wasm-pkg`를 추가하고 out을 `source-wasm-sweep`으로 변경해 exit 0입니다. Docker daemon이 실행되지 않아 `--no-opt` 진단 빌드를 사용했으며 최적화된 배포물은 미검증입니다.

Native export-svg의 `--debug-overlay --embed-fonts=full --font-path <같은 글꼴 경로>`와 `node scripts/rasterize-svg-webfonts.mjs --input <svg> --output <png>`로 반례 실제 화면을 캡처했습니다. 새 WASM에서도 `scripts/export-wasm-for-sweep.mjs --pkg <위 pkg> --input <각 반례> --out <별도 output>`를 실행해 같은 page/Table/Cell 좌표를 확인했습니다. 모두 exit 0입니다. 공개 PNG는 실제 full embed 캡처이며 AI 생성 이미지가 아닙니다.

## 임시 출력과 판독

조사 root는 `output/pr-review/pr7433-review-20261001/`. Native/WASM 각 Sweep의 `two-line`·`nested`의 p1에 `compare/compare_001.png`, `overlay/overlay_001.png`, `review/review_001.png`를 생성하고 직접 열었습니다. 보존 파일은 대표 review·standalone overlay와 두 반례의 실제 화면입니다. 원시 raster·생성 SVG/JSON·metric/run manifest·MCP 응답·로그·실행 바이너리·글꼴 파일은 commit하지 않습니다.

- 일반 p1: Native/WASM 모두 visual_accuracy_proxy_percent 18.10631, 2px silhouette 93.72093, gate passed.
- 중첩 p1: Native/WASM 모두 visual_accuracy_proxy_percent 16.84943, 2px silhouette 90.59971, gate passed.
- 판독: 원본 두 사례의 셀 확장을 확인했습니다. 다음 표와 중첩 내용이 기준보다 약 4px 위인 차이는 남습니다. 전체 화면 pixel match와 내용 일치율을 구분하고 90% gate를 최종 수용 판단으로 쓰지 않습니다.

## 최종 보존 파일

| path | bytes | SHA-256 |
| --- | ---: | --- |
| `mydocs/pr/assets/pr_7433/height_boundary_4838.png` | 34676 | `930670dc577fa8a1d35657d776380216c2887f3a1ca34d3c4568c412c2ad1b22` |
| `mydocs/pr/assets/pr_7433/native_nested_overlay_001.png` | 43158 | `3b0d0309838be3befb59413516a14f52ba00ed6fabcfe95a9d5c1c321f641517` |
| `mydocs/pr/assets/pr_7433/native_nested_review_001.png` | 95504 | `25fe1adbd12a0ac181b98b59f501125ae35ca91db78365209e03e34489e1c82b` |
| `mydocs/pr/assets/pr_7433/native_two-line_overlay_001.png` | 37451 | `c1137612a435be5dcf01ce828c8048f5eb4350cfeba4ff3205baa70df5c2b242` |
| `mydocs/pr/assets/pr_7433/native_two-line_review_001.png` | 79396 | `85a2883d34fdd7acf669810a9cd30dd063e40753852c0d8007ec58c03279b375` |
| `mydocs/pr/assets/pr_7433/page_boundary_28000.png` | 21386 | `f6d9cb5404a283a1f77cda40b68f22cadb45d6499c479cbd065a3a49b09e6e58` |
| `mydocs/pr/assets/pr_7433/wasm_nested_overlay_001.png` | 44632 | `5300e4fe04fd8fd21b99580224e2f65b5decce0298a3629ae6b9ea09ddff823b` |
| `mydocs/pr/assets/pr_7433/wasm_nested_review_001.png` | 96715 | `e8708f2074d011de2bd91b339458e5c164593d117a18bc1ff19d4b6d1abe5024` |
| `mydocs/pr/assets/pr_7433/wasm_two-line_overlay_001.png` | 38885 | `02045385e0c14566d74b968ebbdf518b314e178b2c8dd1659a7a83763fec13ef` |
| `mydocs/pr/assets/pr_7433/wasm_two-line_review_001.png` | 80994 | `55ad28af10f0f7e7a3c906e5cf97ee102728fb0f9244ea9d9fb60b24c1cdb7a5` |
| `mydocs/pr/assets/pr_7433/inputs/common-height-4838.hwpx` | 8398 | `bbfb52b87f03ea5e2a73abbcf3f2e17fbe76a7566d92f62c357d67d4479ccf42` |
| `mydocs/pr/assets/pr_7433/inputs/page-height-28000.hwpx` | 8391 | `8d3a298e85cf65d64e06da76c3911b927b6d1d1b056714f1fd1cf3c27be37d71` |
| `samples/issue7419/tac_two_line_cell_no_lineseg.hwpx` | 8457 | `08a6740ac106b24fbe8dd6d632137ba5748ab8fb2a95187882b6be734f5c36c2` |
| `samples/issue7419/tac_nested_two_line_cell_no_lineseg.hwpx` | 8428 | `59f8fa47e3766124d7e267e4b95e97f9a7050ca8b963261c6bd24c46f517dd76` |
| `pdf/tac_two_line_cell_no_lineseg-2024.pdf` | 13905 | `e72cfff4a48510e124f21cce7709545b34be4196256a50030d11aa807e52ef3f` |
| `pdf/tac_nested_two_line_cell_no_lineseg-2024.pdf` | 12157 | `36e60468e546a0686d8adc6cf8be6a97be26d6225886a325550c99cfc257af2a` |
