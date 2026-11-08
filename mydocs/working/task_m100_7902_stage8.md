# #7902 8단계 — 변환 CFF 메타데이터의 실제 소비 확인

Issue: #7902. [7단계](task_m100_7902_stage7.md)는 커밋했고 최신 devel `f5251e7de4` 위에서
원래19문서·835쪽 범위를 유지한다. renderer PR의104쪽 하락 보류 조건은 그대로다.

## 진단 근거와 범위

독립 Type3에서 얻은 원본 윤곽을 CFF로 변환한 font에는 glyph hint 명령이나 gasp가 없다.
[OpenType gasp 사양](https://learn.microsoft.com/en-us/typography/opentype/spec/gasp)은
gasp가 없을 때 rasterizer가 기본 정책을 적용할 수 있고,0x000A는 grayscale/다축 smoothing을
선호하며 grid-fit 비트를 포함하지 않는다고 설명한다.
[FreeType 계약](https://freetype.org/freetype2/docs/reference/ft2-gasp_table.html)은 이 표를
주로 TrueType bytecode hinting에 이용한다고 설명한다. 따라서 CFF의 Native/Chrome이 실제
이를 사용하는지 추측으로 구현하지 않고 확인한다.

- 입력은 대상 PDF에서 새 glyph를 얻지 않은 기존 독립 표본의 공개 helper 출력이다.
  `public-restored-font/HGSMJ-original.otf` SHA는
  `ecfdaa807e66b97a41965251939dbecebc53fef174c5aa8e8e016dc70b03048a`다.
- public helper 출력 자체와, 그 파일에 gasp version1/전 크기0x000A만 추가한 파일을 비교한다.
  CFF·cmap·hmtx·hhea 등 기존 테이블의 바이트 동일성을 먼저 검사한다. 원래 font는 보존한다.
- 실제 producer는7단계에서 검증한 Native/fresh WASM을 재사용한다. Rust/Studio src는
  동일하며 backend·DPI·원본·Print·논리 원점·측정 마스크·반경을 바꾸지 않는다.
- 같은104쪽을2개 font variant×2개 backend로 나눠 한 배치로 확인한다. 이는 선택 쪽 진단이며
  전체 문서 검증이나 실제 적용으로 보고하지 않는다.
- helper 출력과 메타데이터 variant가 출력에 차이가 없으면 원인 해결로 채택하지 않는다.
  차이가 있어도 실제 consumer와 독립 기대를 연결한 뒤 영향 대조군·전쪽 무회귀를 확인한다.
  font metadata를 점수에 맞춰 조정하거나 family/문서 ID 예외를 추가하지 않는다.

ignored 실행 기록은 `output/pr-review/issue7902-render-repair-20261014/`의
`run-font-metadata-diagnostics.py`, `font-metadata-diagnostic-result.json`에 보존한다.

## 실제 결과

| font variant | Native104쪽 | WASM104쪽 | 기존 실제 PNG와 동일 |
| --- | ---: | ---: | --- |
| public helper의 bearing 복원본 | 99.94062% | 99.92986% | 양쪽 SHA-256 동일 |
| 같은 복원본 + gasp0x000A | 99.94062% | 99.92986% | 양쪽 SHA-256 동일 |

gasp variant SHA는 `c0af95c6e0f63c533736d7a61ad1e161d94f389baaa36b7f5987b77e913a226f`다.
기존 glyph/메트릭 테이블은 모두 동일했고 네 실행이 exit0으로 끝났다. WASM export의 실제
loaded 목록에서 각 variant SHA와 `한양신명조` family를 확인했다. Native의 별도 face 선택
trace는 이번 진단에서 실행하지 않았다. 이 결과는 테스트한 경로의 출력 동일성만 입증한다.

**메타데이터 단독 변경은 해결책으로 채택하지 않는다.** public helper나 원본 공급 font를
수정하지 않았고 Rust를 재컴파일하지 않았다. 다음 분석은7단계의 Type3 윤곽 재생 결과를
기반으로 실제 glyph paint와 실제 face byte 전달·소유를 추적한다. source 전진폭·기준선·
프레임 예약을 다시 바꾸거나 동일한 전쪽 검증을 반복하는 것으로 이 차이를 숨기지 않는다.
보류 중인104쪽 Native0.01791%p·WASM0.01559%p는 그대로다.

`font-metadata-diagnostic-pixel-identity.json`에 네 출력의 PNG SHA와 실제 WASM face 적재
확인을 연결했다. 독립 기준 PDF·좌표·실루엣 알고리즘은 그대로다.
