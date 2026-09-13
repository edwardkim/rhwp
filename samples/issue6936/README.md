# #6936 굵게 PDF 재현 자료

Issue: [#6936](https://github.com/edwardkim/rhwp/issues/6936)

6개 글꼴(돋움·바탕·굴림·새굴림·함초롬돋움·함초롬바탕)의 보통/굵게를
16pt로 배치한 새 공개 합성 입력이다. 12문단, 1페이지, 공백 제외 158자다.

- `bold-faces.hwpx`: 기존 `samples/issue5874/italic-repro.hwpx`의 ZIP/XML 구조를 사용해
  글꼴·글자 속성·본문을 새로 만든 입력. 기존 문서를 이름만 바꾸어 중복 추가한 자료가 아니다.
  새굴림의 원본 font face는 `New Gulim`이고 화면 라벨은 `새굴림`이다.
- `bold-faces.hwp`: 위 HWPX를 한컴 12.0.0.4605로 저장한 입력. 한컴이 font face를 `새굴림`으로
  정규화하고 저장한 LineSeg를 포함한다. **시각 비교의 기준 입력은 이 HWP다.**
- `../../pdf/issue6936-bold-faces-2020.pdf`: 동일 HWP를 한컴 MCP engine 2020으로 변환한 독립 기준.
  `start → status → download`를 수행했으며 두 작업의 job ID·해시는
  `../../mydocs/pr/assets/issue_6936/hancom-conversion.json`에 있다.
- 최초 수제 HWPX에는 한컴 저장 LineSeg가 없어 직접 내보낸 PDF의 문단 위치가 다르다.
  HWPX는 파서·글꼴 이름 회귀에 사용하고, 이를 HWP 기준과 위치가 같은 시각 증거로 주장하지 않는다.

## 글꼴과 두께

검증에는 `win10-ted:C:/Windows/Fonts`의 `batang.ttc`, `gulim.ttc`, `NGULIM.TTF`,
`HANBatang.ttf`, `HANBatangB.ttf`, `HANDotum.ttf`, `HANDotumB.ttf`를 읽기 전용으로 복사해
macOS CLI의 `--font-path`로 제공했다. 글꼴 파일 자체는 커밋하지 않는다.
각 파일의 출처·SHA-256은 `../../mydocs/pr/assets/issue_6936/font-environment.json`에 기록한다.

한컴 기준 PDF에서 일반 face의 합성 굵게는 `Tr 2`와 `w/Tf = 2.66/133 = 0.02`를 사용한다.
이 비율을 SVG 좌표의 글꼴 크기에 적용한다. 함초롬 두 family는 실제 Bold face를 사용하므로
추가 획을 합성하지 않는다. 글꼴 크기·위치·원본 문자는 유지한다.

## 대조군 재현

최신 기준은 `70bf40af2a2818e72bd58b4fa66e2d4c06de2b51`이다.
같은 전용 target과 같은 글꼴 경로에서 각 코드 상태를 순차 빌드했다.

```sh
cargo build --locked --bin rhwp --target-dir /Users/tsjang/rhwp/target/issue6936-20260913
/Users/tsjang/rhwp/target/issue6936-20260913/debug/rhwp export-pdf \
  samples/issue6936/bold-faces.hwp \
  --font-path /private/tmp/rhwp-6936-20260913/fonts \
  -o /private/tmp/rhwp-6936-20260913/after.pdf
```

| PDF | 코드 상태 | 공백 제외 추출 문자 | 텍스트 show 연산 |
| --- | --- | ---: | ---: |
| `issue6936-before.pdf` | 위 devel의 PDF·vendor·layout-name 파일 | 158 | 158 (`Tr 0`) |
| `issue6936-stroke-only.pdf` | 합성 획+New Gulim 적용, vendor는 devel | 207 | 207 (`Tr 0`+`Tr 1`) |
| `issue6936-after.pdf` | 최종 수정본 | 158 | 158 (49개 `Tr 2`) |

세 PDF는 모두 `../../pdf/`에 있으며, 같은 입력을 사용한다. 대조 실험 뒤 소스는 모두 복원했다.
New Gulim을 제외한 모든 행은 수정 전후 글자 원점이 동일하다. New Gulim 두 행은 기존의
HCR Dotum 치환이 없어지며 원래 글꼴 advance를 사용한다(최대 원점 차이 5.200012pt).

PDF 구조·텍스트 추출·렌더 증적은 다음 명령으로 재생성한다(pypdf, pymupdf, Pillow 필요).

```sh
python mydocs/pr/assets/issue_6936/inspect_pdf.py
```

`comparison.png`는 같은 페이지 좌표를 144 DPI로 자른 한컴/수정 전/수정 후 비교다.
`hancom.png`, `before.png`, `stroke_only.png`, `after.png`는 전체 페이지이며 재배치하지 않았다.
전체 페이지 픽셀 일치나 Windows 네이티브 CLI 실행을 검증했다는 뜻은 아니다.

## 입력과 기준 PDF SHA-256

- `samples/issue6936/bold-faces.hwpx`: `c751bf79f44411377c2d58db158068cad04e6b1f0fa8d830a6e867d27dc8b59a`
- `samples/issue6936/bold-faces.hwp`: `eefe5d89c29a8d7c8ba3a3148033fb3c5c2e960205ef1ff927ec36873089a3a7`
- `pdf/issue6936-bold-faces-2020.pdf`: `5b0dc916098847ed3c6e3eadae468f33a9810ee58de3e91c1e2bb2f3a7e2ae12`
