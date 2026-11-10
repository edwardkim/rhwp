# planet6897 19개 PR 통합 검토 증적

기준은 `5ea1227505978156082121f1601cb03044a0583f`, 검증 제품 코드는
`21d62361fa670858e63fdfc475c236f112e20a3d`다. #8854는 이미 devel에서 해결되어 통합에서 제외했다.

최신 Native 44조건 2,109쪽 검증이 완료됐다. 90% 미만·쪽수 불일치·보호 기준 하락은 없다.
원 PR 입력 27조건 721쪽과 정본 보호 18조건 1,771쪽은 편람 HWPX 383쪽을 공유하며,
같은 producer의 결과를 한 번 측정해 두 역할에 연결했다. 조건별 전체 값은
`native-final-validation.json`, 원 PR별 TSV와 같은 쪽의 raw·PDF·review·overlay는 각 `cases/`에 있다.

직접 판독에서 발견한 #8752의 2쪽 그림 뒤 줄 가림과 #8734의 HWP5/HWPX 27쪽 명령문 줄 합침은
`0f2ebe265e`에서, 고용서비스 보고서 79쪽의 본문·그림·캡션 겹침은 `21d62361fa`에서 공통 프레임
소유 규칙으로 보정했다. 이 네 조건의 같은 쪽 Native/fresh WASM은 각각 100.00%이며,
`pr8752/maintainer/`와 `pr8734/maintainer/`에 전후 증적과 실제 FAIL→PASS·반례를 보존한다.
현재 코드의 관련 집중 회귀는 19개가 통과했다.

최소 글꼴 환경 전체 nextest는 13,227개 통과·실패 0개·45개 건너뜀이다. Native Skia 루트 3,852개와
관련 라이브러리 182개·추가 그림/PDF 검사, 세 Clippy·workspace build·fmt·manifest·unit tiers·doc tests도
최종 단계에서 통과했다. 상세 결과와 파생 suite 준비 오류의 해소 이력은 `rust-final-validation.json`에 있다.
원 PR 입력 27조건의 base 대비 2%p 초과 하락이 없음은 `native-base-comparison.json`에서 확인한다.

fresh WASM도 44조건 2,109쪽을 완료했다. 최저는 Native 92.85%, fresh WASM 92.78%이며
90% 미만·쪽수 불일치·정본 보호 공식 표시값 하락이 없다. 양쪽 원 PR 입력 27조건 721쪽에서
고정 base 대비 2%p 초과 하락은 없다. 각 backend의 합본은 `native-all-pages.tsv`와
`wasm-all-pages.tsv`, 판정은 `native-final-validation.json`과 `wasm-final-validation.json`이다.

실제 Studio는 일반 열기의 편집 전환과 CanvasView/PageRenderer를 통해 screen/print 44건을
확인했다. 실제 Canvas2D 28건, CanvasKit 16건이다. 글꼴 요구 목록 미준비로 Canvas2D에 fallback한
6건을 CanvasKit 통과로 세지 않는다. CanvasKit 15건은 실제 software surface와 runtime readiness를
통과했고, 원본 그림 바이트가 없는 screen 반례 1건은 `image:dataMissing`·readiness 실패를 그대로
보존했다. 지원 대조군과 최소 글꼴 4건을 포함한다. 각 backend의 글꼴 윤곽 차이를 독립 Print 점수나
동일한 실제 face로 오인하지 않으며, bounded layout trace는 paint face 식별 증거가 아니다.

SVG의 실제 browser webfont raster 8건도 모두 90% 이상이다. 79쪽·마운트 명령문·그림 걸침·TAC는
같은 내용·소속을 보존했다. SVG의 기존 WMF 내부 가는 선 재현 차이는 직접 판독 기록에 남겼다.
실제 소비 결과·원본 screenshot·초기 JS/WASM 응답과 판독은 `consumers/`에 있다. 원 PR별 검토는
`mydocs/pr/archives/pr_<번호>_review.md` 19개로 분리했다. 원격 CI·병합은 아직 별도 확인 대상이다.

Vite는 초기 JS 응답에 WASM URL 재작성과 source map을 넣는다. 두 변경만 존재함을 정확히 검사했고
wire hash와 pkg/public 원본 hash를 구분했다. WASM 초기 응답은 원본 바이트와 동일하다. 초기 진단
스크립트의 응답 수집 실패는 제품 렌더링 전에 중단해 제외했다. 완료된 installed 40건은 브라우저와
서버 종료 뒤 남은 Node 핸들을 정리했고, 후속 최소 글꼴 4건은 명시적 종료를 보완한 스크립트에서
exit 0으로 끝났다. 해당 수명주기와 완료 결과를 `consumers/browser-validation.json`에 구분했다.
100.00%는 2px 내용 실루엣 지표이며 RGB 전체 픽셀 동일을 뜻하지 않는다.

- `native-claim-index.json`: 원 PR·입력별 전쪽·주장·최저 페이지 목록.
- `protection/native/`: 현재 제품 코드의 정본 보호 18조건 전쪽 TSV와 판정.
- `code-source-association.json`: 고정 binary/package와 커밋된 실제 제품 코드의 해시 대응.
- `native-direct-reading-notes.json`, `native-direct-reading-reuse-audit.json`: 현재 Native 증적 65쪽의 직접 판독과 동일 raw/PDF 재사용 근거.
- `wasm-direct-reading-notes.json`, `wasm-direct-reading-audit.json`: fresh WASM 증적 65쪽의 직접 판독과 동일 raw/PDF 재사용 근거.
- `direct-reading-final-audit.json`: Native 65쪽·WASM 65쪽의 최종 판독 완결 확인.
- 각 `provenance.json`: 실행 producer·입력/PDF·PNG SHA-256.
- 각 원 PR의 `maintainer/`: 단계별 실제 실패 검출·보정·전후 증적.

고용서비스 보고서 51쪽의 기존 표 하단 경계와 원 HWP3 27쪽 캡션의 기존 차이는 해당 판독 기록에서
이번 수정과 구분했다. #7282의 원 PR이 제외한 KGS 서식까지 해결했다고 표시하지 않는다.
글꼴 바이너리·로그·임시 스크립트는 이 자산에 포함하지 않는다.

## 원 PR별 검토 문서

| 원 PR | 검토 문서 |
| --- | --- |
| [#8691](https://github.com/edwardkim/rhwp/pull/8691) | [fix(layout): 행 선언 합이 표 선언을 넘으면 마지막 행에서 회수한다 (#8682)](../../archives/pr_8691_review.md) |
| [#8734](https://github.com/edwardkim/rhwp/pull/8734) | [fix(layout): 어울림 그림 바깥 여백을 저장 줄 시작에 이중 적용하지 않는다 (#8730)](../../archives/pr_8734_review.md) |
| [#8747](https://github.com/edwardkim/rhwp/pull/8747) | [fix(layout): 빈 앵커 문단 뒤 빈 문단의 저장 사다리를 띠 아래 host 줄 증거로 쓴다 (#8731)](../../archives/pr_8747_review.md) |
| [#8752](https://github.com/edwardkim/rhwp/pull/8752) | [fix(layout): #7282 여러 단에 걸친 자리차지 표가 다른 단 문단의 줄도 표 아래로 민다](../../archives/pr_8752_review.md) |
| [#8762](https://github.com/edwardkim/rhwp/pull/8762) | [fix(layout): 같은 문단의 어울림 표와 TAC 표를 표 옆 차선에 흘린다 (#8753)](../../archives/pr_8762_review.md) |
| [#8774](https://github.com/edwardkim/rhwp/pull/8774) | [fix(typeset): #8770 공백 앵커 문단의 문단 기준 표 묶음은 앵커 줄을 남기고 표만 다음 쪽으로 옮긴다](../../archives/pr_8774_review.md) |
| [#8787](https://github.com/edwardkim/rhwp/pull/8787) | [fix(layout): 나뉜 자리차지 표에 host 첫 줄 들여쓰기를 더하지 않는다 (#8748)](../../archives/pr_8787_review.md) |
| [#8796](https://github.com/edwardkim/rhwp/pull/8796) | [fix(renderer): 다음 문단 저장 위치로 정한 문단 끝을 현재 줄 축으로 투영 (#8790)](../../archives/pr_8796_review.md) |
| [#8802](https://github.com/edwardkim/rhwp/pull/8802) | [fix(layout): 저장 사다리 밖 첫 문단 앞 간격을 쪽 원점에서 뺀다 (#8797)](../../archives/pr_8802_review.md) |
| [#8837](https://github.com/edwardkim/rhwp/pull/8837) | [fix(layout): 쪽 기준 자리차지 표가 있는 글 문단의 첫 줄에 앞 간격을 한 번만 쓴다 (#8836)](../../archives/pr_8837_review.md) |
| [#8839](https://github.com/edwardkim/rhwp/pull/8839) | [fix(typeset): 합성 TAC 표 줄의 바깥 여백 이중 가산과 쪽 기준 글 앞 중첩 표의 칸 높이 적층 (#8834)](../../archives/pr_8839_review.md) |
| [#8847](https://github.com/edwardkim/rhwp/pull/8847) | [fix(typeset): TAC 표 줄의 음수 줄간격도 뒤 글줄 흐름에 반영한다 (#8841)](../../archives/pr_8847_review.md) |
| [#8865](https://github.com/edwardkim/rhwp/pull/8865) | [fix(render): 그림 미지정 이미지 노드도 인쇄 등가 출력에서 생략한다 (#8858)](../../archives/pr_8865_review.md) |
| [#8868](https://github.com/edwardkim/rhwp/pull/8868) | [#8859 빈 host 양수 오프셋 자리차지 표 위 틈에 저장 줄 문단을 둔다](../../archives/pr_8868_review.md) |
| [#8886](https://github.com/edwardkim/rhwp/pull/8886) | [fix(layout): #8877 칸 안 글자 없는 줄의 TAC 표도 공유 기준선에 앉힌다](../../archives/pr_8886_review.md) |
| [#8887](https://github.com/edwardkim/rhwp/pull/8887) | [fix(composer): 긴 낱말은 줄 첫 글자 뒤로 넘기고 줄머리 금칙 묶음을 뒤 낱말과 맞춘다 (#8883)](../../archives/pr_8887_review.md) |
| [#8892](https://github.com/edwardkim/rhwp/pull/8892) | [fix(typeset): 저장 줄 없는 다음 문단 앞에는 빈 앵커 자리차지 표의 아래 바깥 여백을 흐름에 넘긴다 (#8888)](../../archives/pr_8892_review.md) |
| [#8898](https://github.com/edwardkim/rhwp/pull/8898) | [fix(layout): #8890 저장 줄 없는 공백 문단의 RowBreak TAC 표에 안 여백 inset 을 더하지 않는다](../../archives/pr_8898_review.md) |
| [#8911](https://github.com/edwardkim/rhwp/pull/8911) | [fix: 저장 줄 없는 글 문단의 합성 vpos 를 저장 되감김 쪽 경계 판정에 쓰지 않는다 (#8904)](../../archives/pr_8911_review.md) |
