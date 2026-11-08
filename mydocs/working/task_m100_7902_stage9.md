# #7902 9단계 — HFT 원본 윤곽의 실제 paint 경로 연결

Issue: #7902. [8단계](task_m100_7902_stage8.md)의 메타데이터 단독 진단은 커밋했고 미채택했다.
기준 devel은 `f5251e7de4`, 구현 전 head는 `bbc5b951d`다. 원래19문서·835쪽을 유지한다.

## 근거·규칙·범위

원본 HWPX header는 한양신명조를 `type="HFT"`로 선언한다. 독립 Print의 Type3 원본 program과
같은 곡선을 공급한 CFF를 같은 논리 원점·크기로 직접 재생한 진단은 WASM104쪽을
99.92986→99.93860%로 개선했다. glyph125회이며 배치·마스크를 바꾸지 않았다.
HFT의 실제 윤곽 재생과 Font API의 raster를 구분해 검증한다. 일반 TTF raster 정책은 근거가 아니다.

- 선언한 HFT 여부는 언어 슬롯별 원본 font face의 `alt_type`에서 보존한다. 표시 이름·문서 ID나
  fullwidth-dot 같은 다른 속성으로 추측하지 않는다.
- 적용 범위는 Print 계열의 보통 글자, 원본 HFT, 실제 선택된 정상 CFF face, cmap에 있는
  단일 scalar다. 굵게/기울임/그림자/외곽/양각 등 효과·복잡 cluster는 기존 painter가 소유한다.
- 원본 Type3 변환이 사용하는 표준 em matrix의 CFF1 named glyph를 bounded 공통 contour로
  만든다. 같은 원점·effective draw size·장평 변환을 소비하고 source 전진폭·줄 높이는 유지한다.
- Native는 기존 typeface 선택 결과와 첫 요청 face가 일치할 때 그 실제 CFF table을 쓴다.
  Canvas는 FontFace 적재가 끝난 host가 같은 바이트를 명시 전달해야 한다. family만 보고
  별도 파일을 찾지 않고, 없는 bytes·glyph·지원 범위는 원래 paint로 돌아간다.
- 기존 exact-font 등록 API는 재컴포즈·측정 snapshot도 갱신하므로 paint 전용 공급으로
  재사용하지 않는다. 별도 세션 상태는 저장 문서/조판을 변경하지 않고 bounded bytes만 소유한다.

## 실제 값의 소비 경로

`원본 font face → ResolvedCharStyle의 언어별 source-kind → TextStyle → 기존 replay positions →
Native SkiaTextReplay / WebCanvasRenderer의 char 원점 → 공통 CFF contour → 기존 CTM/clip/fill`.
Canvas bytes는 `FontFace 실제 적재 → HwpDocument 세션 등록 → render 호출의 immutable snapshot`이다.
font/style/문서의 저장 정보는 다시 쓰지 않는다. 효과·marker·옛한글·TTF의 각 별도 분기도 대조한다.

## 배치 검증 계획

1. source-kind·공통 contour·세션 공급·양쪽 painter·host loader를 한 배치로 연결한다.
2. 하나의 공유 target에서 Native/fresh WASM을 순차 빌드한다.
3.104쪽과 TTF 대조군14/40쪽, rowbreak7/15쪽 등 영향 경계를 양쪽에서 먼저 비교한다.
   직접 review를 읽고 방향이 맞을 때 전쪽 Native/fresh WASM과0.01%p 비교로 진행한다.
4. 시각 선행 조건을 충족한 실제 source에서 필요한 정식 관계/px 검사와 font-program 계약을
   보완한다. font-only 합성 계약과 한컴 문서 일치 증거는 구분하고 수정 전 FAIL/수정 후 PASS를 확인한다.
5. 마지막 구현에서만 필수 전체 회귀·lint를 실행한다. PR 전 devel head 변경을 확인하고,
   실제 변경이면 리베이스·영향 검증 뒤 자동 승인된 PR/CI/옵션2 문서/merge/후속 처리를 진행한다.

## 첫 배치의 실제 실행 결과

`stage9-producer-build.json`에 구현 전 head와 변경 파일별 SHA-256, 실제 Native/WASM 바이너리
SHA를 기록했다. Native dev 빌드228초·fresh WASM217초가 exit0으로 끝났다. 이 산출물은 아래
지원하지 않는 프로그램 교체 보완 전의 **첫 진단 producer**이며 최종 head 전쪽 검증과 구분한다.

| 104쪽 | devel 기준 | 보정 전 | HFT 윤곽 replay | 기준 대비 하락 |
| --- | ---: | ---: | ---: | ---: |
| Native | 99.95853% | 99.94062% | 99.95192% | 0.00661%p |
| fresh WASM | 99.94545% | 99.92986% | 99.93860% | 0.00685%p |

104쪽은 양쪽 모두0.01%p 보류 조건을 해소했다. Native render tree는 이전과 바이트까지 같다.
본문의 제목·번호·한글/ASCII 간격, 위 표와 그림·후속 문단을 review와 확대 crop에서 대조했다.
전체 그림 경계의 기존 소수 좌표 차이는 남아 있으며100% 일치나 전체 범위 직접 판독 완료로
보고하지 않는다. 그림·뒤 문단과 프레임 수정은 되돌리지 않았다.

6761-later14/40쪽, rowbreak HWP/HWPX7/15쪽은 Native6개·WASM6개 모두 이전 PNG와
SHA-256이 같다. Native6개의 render tree도 모두 같다. `stage9-boundary-audit.json`,
`stage9-native-boundary-identity.json`, `stage9-p104-native-detail.png`,
`stage9-boundary-*/{native,wasm}/<key>/review/`에 원시 증거를 보존했다.

## fallback 보완과 집중 계약 검사

- 실제 새 FontFace가 TTF·WOFF 등 지원 범위 밖 프로그램으로 교체되면 이전 CFF 바인딩을
  지우고 원래 painter에 맡긴다. 빈/과대 바이트는 오류로 거절한다. source HFT가 아닌 family는
  등록하지 않는다. Native도 컬러·variable·실제 bold/italic face를 윤곽 경로에서 제외한다.
- 독창적인1000UPM 사각형 CFF로 font-only 계약3개를 추가해 **3 PASS**를 확인했다.
  소수 기준선1/4px 이동 보존, 일반 CFF/Screen의 기존 격자, 실제 public Core 등록의 멱등성·
  프로그램 교체·문서 정보 보존을 검사한다. 실제 한컴 문서의 px 정답지로 사용하는 fixture가 아니다.
  생성 script와 MIT 출처를 함께 보존했고 두 번 생성한 SHA가 같았다.
- 실제 Chrome의 FontFace 공급·같은 bytes 전달·원래 API 부재 fallback·launch 정리 검사3개가
  **3 PASS**다. 새 bridge는 `FontFace.load/add` 완료와 바이트 동일성도 실행으로 검사했다.
- 등록 바이트/face-index와 family를 명시 전달하며 별도 파일 탐색·문서 ID·좌표 보정은 없다.

## 남은 검증

최종 producer에서19문서·835쪽씩 전체 Native/fresh WASM과 고정 devel의0.01%p 비교를 다시
확인해야 한다. 신규 font 계약의 음성 대조, 필요한 중요한 완료 페이지의 관계/±2px 보호,
최종 전체 Rust·Native Skia·lint도 남아 있다. 위 선택 쪽 결과만으로 PR 준비 완료·이슈 해결·
merge 승인으로 판정하지 않는다. 사용자의 PR/CI/옵션2/merge/후속 자동 승인은 유지한다.
