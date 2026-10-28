# 개인정보의 명시 TTF 전진과 직접 Canvas 괘선 소비 보정

Issue: #7445. 단계 보고서 C442를 따른다. 이 자료는 개인정보66쪽의 프로그램 소유와
직접 Canvas 소비 보정 범위다. **전체 corpus·전체 회귀·actual36 등 PR 필수 검증은 별도로 남아 있다.**

## 입력과 실행 신원

- 원본 `samples/task2097/75544_pii_bunseok.hwpx`, SHA256
  `f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1`.
- 독립 기준 `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`, 한컴2020 Print66쪽, SHA256
  `0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770`.
- 후보는 parent `ff48a44dac59e32001fe3f838fdc46184f62fae1` 위의 style_resolver와
  direct WebCanvas 수정이다. Native217/fresh WASM218은 같은 runtime2201파일/hash를 고정했다.
- Native CLI SHA256: `1f5d908744c269b6246a4ec2f2f6b0eb7b6d243b82d3509a4e895929cc47a9be`.
- WASM SHA256: `ea286b02517af0c0f7210ab223c85c2776c85fdad2009a3e3ac3b63400f5c75c`.
- JS SHA256: `23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591`.
- Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
  root pkg/Studio public 및 실제7736 초기화 응답216을 확인했으며 Docker 최적화 검증이 아니다.
- 192DPI·compat2022·Print·PDFium·색상 경계 보정 전2px·90%와 같은4a07 고정 기준을 유지했다.
  pdftoppm은 별도 감사다. 세 TrueType 공급 경로와 선언/기본 환경은 앞 단계와 같으며,
  이 문서에 맞지 않는 HFT 부분 검토 프로그램을 공급하지 않았다.

## 전쪽과 devel 비교

219에서 네 조건264쪽을 새로 렌더·산출했다. 각 TSV는1~66쪽이 한 번씩 있으며,
같은 backend의 선언/기본 환경66쪽 PNG는 byte동일하다. 네 경로의 중첩 table/cell/문단
소유별 공백 제외 Unicode 본문도 전쪽 일치한다. 이는 출력 경로 간 일치이며 독립 PDF
텍스트 정답을 대신하지 않는다. 기존 #5846 원문 내용·종료 검사는 유지한다.

| backend | 환경 | 정본 최저 raw | 90% 미만 | 쪽수 | pdftoppm 감사 최저 |
| --- | --- | --- | --- | --- | --- |
| native | pii66-declared | 94.58132% | 0 | 66/66 | 94.12869% |
| native | pii66-default | 94.58132% | 0 | 66/66 | 94.12869% |
| wasm | pii66-declared | 94.60117% | 0 | 66/66 | 93.78715% |
| wasm | pii66-default | 94.60117% | 0 | 66/66 | 93.78715% |

18·43쪽은 네 조건 모두100%다. 같은4a07 대비264쪽 및 수정 전202/204 대비264쪽에서
0.1%p 이상 하락은0쪽이다. 바뀐 PNG는 각 조건의1/2/18/41/43/45쪽뿐이며 나머지60쪽은
수정 전과 byte동일하다. raw100은 RGB100% 일치를 뜻하지 않는다. 얇은 획·본문 일부의
작은 프린지는 남으며 전체 PR의 완료로 확대하지 않는다.

- [전쪽 수정 전후 비교](before-after.tsv), [같은 devel4a07 전쪽 비교](devel-after.tsv).
- 수정 전 TSV는 [C441 after](../pii-source-frame-and-overpaint/README.md)의202/204 결과다.
  새 후보 TSV는 이 폴더의 `after/`, 같은 고정 base는 `base4a07/`에 바이트 그대로 보존했다.

## 원인과 회귀

원문은 HCI Poppy type1와 Palatino Linotype type1 substFont를 선언한다. 독립 PII18/F7·
43/F4는 Palatino Type0/CIDFontType2의 꺾쇠 /W500이다. 별도 한컴2020 Print8조건에서
원 face type1+명시 TTF 대체 두 조건은500, 대체 없는 type1와 실제 type2 여섯 조건은605다.
독립 프로그램 표본 PDF와 글꼴·글리프 진단은 local-only ignored output에만 보존했다.
큰 글자 합성본의 불일치 저장 줄 프레임은 시각 수용·새 회귀 근거로 쓰지 않았다.

실제 PII의 정식 새 검사2건은 같은 소스를 frozen202/217에 연결해2FAIL→2PASS이며,
원본 프로그램 검사 파일 전체도 suite024에서21PASS/233선택 제외다. 원본의19검사는
제거·완화하지 않았다. source IR의 명시 type·일반/굵은 프로그램, 자연 전진과 실제 공통
Print paint의 내용·소유·순서·누락·중복·비겹침을 보호한다. 양쪽100%·전66쪽90%와
직접 판독 후 독립 PDF의 제목 끝205.52/172.72CSSpx도 ±2CSSpx로 보호했다.
수정 전18쪽 끝209.72가 실제 FAIL하므로 helper 호출만으로 통과하는 검사가 아니다.

Legacy PageRenderTree와 header/footer preview API도 공통 `paint_strokes`를 소비하게 했다.
실제 PII28쪽 Legacy의 누락 선은 일반 Layer API와 같은 잉크160픽셀/최저192로 복원됐다.
header/footer API의 합성 소비 대조군은 이전에 누락했던 검정→흰색 순서4건을 확인했다.
이 합성본의 footer 일부는 페이지 밖이며 footer 배치 정확성·독립 Print 수용으로 쓰지 않는다.
정상 Studio CanvasView·CanvasKit 준비·actual36 검증은 별도다. 특히 PII의 작업 한도50000
fallback과 미준비 글꼴을 CanvasKit 통과로 숨기지 않는다.

## 대표 review와 overlay

양 backend18/43쪽을 같은 독립 PDF와 직접 판독해 꺾쇠 뒤 한글·셀·본문·줄바꿈을 확인했다.
수정 전은 같은202/204 원본 PNG에서 새 패널만 만들었으며 재캡처 결과로 쓰지 않았다.
원본 PNG·입력/PDF·producer와 재사용 hash는 ignored repair227 copy/audit에 보존했다.

### native 18쪽

before: [review](before-native-p018-review.png) · [overlay](before-native-p018-overlay.png)

after: [review](after-native-p018-review.png) · [overlay](after-native-p018-overlay.png)

### native 43쪽

before: [review](before-native-p043-review.png) · [overlay](before-native-p043-overlay.png)

after: [review](after-native-p043-review.png) · [overlay](after-native-p043-overlay.png)

### wasm 18쪽

before: [review](before-wasm-p018-review.png) · [overlay](before-wasm-p018-overlay.png)

after: [review](after-wasm-p018-review.png) · [overlay](after-wasm-p018-overlay.png)

### wasm 43쪽

before: [review](before-wasm-p043-review.png) · [overlay](before-wasm-p043-overlay.png)

after: [review](after-wasm-p043-review.png) · [overlay](after-wasm-p043-overlay.png)
