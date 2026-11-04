# 새 base #8116의 기능 보존과 통합 검증

## 범위와 이전 단계의 실제 상태

새 devel은 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`(#8116, #8080)다.
이전 base `0bb36e6f976e9ebeba0b1c48d86a2c5bdaee7f7c`의 통합 head
`7b5299787e15c345da13f92f0b3fcc462df55eeb`는
`backup/review-semanticist21-before-faa885-20261027`에 보존했다.
390개 커밋을 리베이스한 직후 head는 `7d9d2b8ae9600279350a6892c875a7b6b1e6ee20`이다.
원 PR 45개를 유지하며 새 외부 PR을 체리픽하거나 다른 실문서를 수집하지 않는다.

이전 source의 실제 결과는 다음과 같으며 새 source의 통과로 대신하지 않는다.

- 집중 검사 48개 PASS. 원본 #7518의 6→8쪽, 앞쪽 여백과 차트의 세 동일 실행물
  RED(exit 101)/GREEN(exit 0)도 확인했다.
- Native 169조건·384쪽과 #8188 네 입력·256쪽이 통과했다. 후자는 raw 100%와
  정수 미일치 0/0이다. 두 혼합 원본의 Native/fresh WASM 전체 32쪽도 98% 이상이다.
- release lib 자식 작업의 명시적 terminal summary는 3,983 PASS / 0 FAIL /
  12 ignored다. 리베이스 대기 전 부모만 종료했으므로 이 자식의 정확한 exit code는
  별도로 포착하지 않았다. 테스트 summary와 exit code를 혼동하지 않는다.
- 전체 nextest·후속 Skia·문서·Studio는 시작 전이었다. 새 base가 들어와 대기 작업을
  superseded로 기록했으며 실행 중인 Cargo·rustc는 종료하지 않았다.
- 이전 source의 광범위 WASM은 디스크 부족으로 기록된 127조건 중 117 PASS,
  10개 측정 불가 뒤 종료했다. fontTools와 journal 쓰기의 `OSError 28` 및 실제
  세션 exit 1을 확인했다. 이를 renderer 회귀나 전쪽 통과로 판정하지 않는다.

원본·PDF·글꼴·Git worktree가 없는 과거 렌더 중간 폴더, 사용이 끝난 실행물과
동일한 중복 raster만 정리했다. 현재 증적과 RED 실행물, 공유 target은 유지했고
약 9.1GB 여유를 확보했다.

## 충돌과 공통 결과의 통합

리베이스의 세 충돌은 다음과 같이 해소했다.

1. `reflow_empty_table_host`: 기존의 쪽/종이 기준 비-TAC 개체와 새 base의 글앞/글뒤
   비흐름 장식을 모두 빈 표 호스트 판정에서 제외한다. 두 helper를 현재 NoLS 경로에서
   함께 소비하며, 저장 줄 앵커의 다른 호출부로 조건을 넓히지 않는다.
2. `split_float_successor_spacing_before`와 `is_non_flow_decoration`: 같은 삽입 위치의
   독립 함수이므로 둘 다 유지한다.
3. `reflow_float_host_line`: 새 base의 장식 import와 기존 `neutral_anchor_metrics` 및
   `ResolvedParaStyle`을 모두 유지한다.

자동 병합 뒤 실제 소비 경로를 대조하니, 새 base가 표를 비흐름 제어에 포함하면서
기존 `neutral_anchor_metrics`도 NoLS overlay 표 호스트에 진입하게 됐다. 여기서는 첫
글자 모양을 선택하지만 새 base의 실제 layout은 `unstored_overlay_table_host_line_metrics`의
최대 글꼴을 선택한다. 두 선택을 독립적으로 유지하면 측정·조판·배치의 줄이 달라질 수 있다.

독립 근거는 #8116의 같은 원본·웹한글기안기 Print와 기존 NoLS 표 호스트 계약이다.
NoLS overlay 표 호스트만 새 base의 결과를 직접 소비하도록 통합했다. 공통 결과에
글꼴 크기를 함께 반환하고 layout, 빈 앵커 측정·조판·문단 layout이 같은 줄높이·줄간격·
글꼴 크기를 사용한다. 다른 부동 앵커의 기존 font 선택과 HWP3 cap은 유지한다.
문서 번호·임의 좌표·높이 clamp를 추가하지 않았다.

## 새 base 기능의 검증 범위

원 PR #8116이 devel에 추가한 다음 네 기존 원본·Print만 기능 보존 대상으로 확인한다.
새로운 사용자 문서를 요청하거나 별도 변형 HWP를 수집하지 않는다. 제공된 기준은
웹한글기안기 Print이며 MCP 2020/2024의 새 출력으로 바꿔 기록하지 않는다.

| 원본 (`samples/issue8080/`) | 독립 Print (`pdf/issue8080/`) | 전체 쪽수 |
| --- | --- | ---: |
| `cell_nested_table_rows.hwp` | `cell_nested_table_rows-webhwp-print.pdf` | 1 |
| `overlay_table_beside_title.hwp` | `overlay_table_beside_title-webhwp-print.pdf` | 5 |
| `picture_cell_row_height.hwp` | `picture_cell_row_height-webhwp-print.pdf` | 5 |
| `title_between_floating_shapes.hwp` | `title_between_floating_shapes-webhwp-print.pdf` | 4 |

기존 `issue_8080_no_lineseg_overlay_table_flow` 15개 검사를 실행하고, 새 base와 후보의
같은 원본·PDF·글꼴·DPI에서 Native/fresh WASM 전체 15쪽씩을 비교한다. 현재 개선한
두 혼합 원본과 차트·셀 그림의 정상 대조군도 먼저 확인한다. base 대비 페이지별 raw
하락 제한은 이 작업의 사용자 지시인 0.1%p이며, 정확히 90%는 통과한다.

이후 기존 169조건·384쪽과 두 혼합 원본, #8188 네 입력을 새 source에서 확인한다.
완료 페이지의 98% 자격도 유지해야 한다. source가 바뀌었으므로 이전 PNG/TSV를
현재 재출력으로 표시하지 않는다. Cargo는 공용 `target/pr-review`에서 8 jobs로
한 작업씩 실행하고 테스트 12 threads를 사용한다. 새 base·후보의 실제 검증, 전체
Rust·Skia·문서·Studio와 개별 리뷰 갱신이 끝나야 PR 준비 완료로 판정한다.

## 선행 검사에서 확인한 두 원인

현재 Native282에서 #8116의 15개 중 14개가 통과하고 중첩 표 글줄 포함 검사가 실패했다.
같은 원본의 실제 render tree를 새 base와 비교하면 중첩 표의 측정 행 합은 둘 다
118.0px인데 후보만 부모의 남은 높이 105.7px를 `inferred_viewport_split`으로 재해석한다.
마지막 행을 35.8px에서 23.5px로 줄인 뒤 두 번째 글줄은 원래 원점에 남아 10.4px 넘친다.
원본에는 이 표를 나눈 저장 줄이나 source cut이 없으며 독립 웹한글 Print도 온전한 표다.

생산자는 `resolve_row_heights`의 실제 행 벡터이고, 실제 소비는 `layout_table`의 누적
행 경계다. 중간의 부모 잔여 높이 기반 추론이 그 결과를 덮어썼다. 글앞/글뒤 표는 흐름을
전진시키지 않으므로 잔여 흐름 높이로 소속을 나누지 않게 공통 overlay 판정으로 배제한다.
명시적인 source cut, 일반 흐름 표의 기하 fallback과 부모 셀 clip 계약은 그대로 소비한다.
확인은 기존 #8116의 글줄 포함·장식 host·뒤 본문 검사와 기존 혼합 원본에서 수행한다.

차트 4건의 89.00396~89.38709%는 별도 문제다. 이전 정상 Native280으로 같은 입력을
현재 다시 실행하면 통과하지만 Native282에는 현재 차트 제목의 `dominant-baseline`
형식과 분리한 Cartesian 모듈이 없다. base worktree 빌드 뒤 root path 의존성의 예전
mtime을 재사용한 공유 target 신선도 문제다. 내용 SHA를 바꾸거나 target을 삭제하지
않고 실제로 달라진 의존성 source의 mtime을 갱신한 뒤 재컴파일과 차트 소비를 확인한다.
이전 Native282의 결과는 보정 전 실제 관측으로 보존하며 최종 통과 근거로 쓰지 않는다.

중복 base worktree에만 있던 samples/PDF/fixture/PR 이미지 체크아웃을 기존 base와 같은
sparse pattern으로 제외했다. 기본 작업공간 원본·Git 객체·모든 빌드 source SHA는
불변이며 이 정리와 종료된 임시 Chromium 프로필 정리 후 여유 공간은 약 8.2GB다.

### 첫 가설의 기각과 확정 원점의 소비 수정

Native283은 차트 99.80761%를 복구했지만 중첩 표는 97.66218%로 그대로였고,
기존 검사도 다시 14 PASS / 1 FAIL이었다. 원본 CTRL_HEADER를 직접 읽으니 부모는
글앞 표이고 문제의 자식은 attr `0x182a2210`, 문단 세로·단 가로 기준 자리차지 표다.
자식도 overlay일 것이라는 첫 가설은 틀렸다. 해당 배제 조건을 제거한다.

자식의 세로 오프셋은 1,203HU다. 호출 중 가용 영역은 y=821.5, h=105.7인데
확정된 배치 원점은 y=805.5다. 실제 부모 하단은 같아도 높이를 이전 원점에서 재어
16px가 사라졌고, 측정 행 합 118px를 105.7px로 잘랐다. 원점 override를 실제 배치에만
쓰고 행 수용은 예전 원점으로 수행한 불일치다. 새 공통 viewport는 같은 부모 하단을
유지하고 확정 자식 원점을 시작으로 삼는다. 바꾼 높이는 임의 여유나 특정 문서 수치가
아니라 같은 물리 경계의 좌표 변환이다. 명시적인 source cut은 기존 viewport를 유지한다.

수정 전 Native283의 전체 측정은 실제 회귀가 확인되어 superseded로 종료하고,
관측된 TSV와 PNG를 보존했다. 새 base175조건 측정은 독립 프로세스에서 끝났으며
다음 후보에서도 그대로 같은 base 실행물을 사용한다. 이 시점에는 수정 후 확인을 아직 실행하지 않았다. 최종 실행 결과는 아래에 기록했다.


## 최종 소스의 실제 검증 결과

검증 코드·테스트 source는 `04c92da78165ccbf63b467855591df183655a98d`, 고정 base는 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`다. 같은 기존 원 PR 45개의 범위만 검증했으며 새 HWP 문서로 확장하지 않았다.

- 일반 조건은 기존 175조건·415쪽과 기존 API 출력 33조건·42쪽으로, 각 backend 208조건·457쪽이다. Native 최저 92.05207%, fresh WASM 최저 91.53306%이며 미달·누락·쪽수 불일치는 없다.
- 같은 원본·Print·글꼴·DPI의 실제 base 공통 828쪽과 API 84쪽에서 0.1%p 초과 하락은 없다. #6133의 실제 빈 둘째 쪽 복원을 판독했으며 없는 base 둘째 쪽의 raw를 만들지 않았다.
- #8116의 기존 네 입력·총 15쪽과 기존 15개 회귀를 모두 통과했다. 확정된 자식 원점과 부모 하단으로 같은 가용 영역을 측정·배치가 소비하며 기대값을 완화하지 않았다.
- #8188의 지정 네 입력 × 64쪽 × Native/fresh WASM은 전체 512쪽에서 raw 100%와 candidate/reference 정수 미일치 0/0을 확인했다. 실제 원 PR의 217개 글꼴 프로그램을 동일하게 공급했다.
- fmt·Native/WASM/workspace/Skia Clippy·workspace build·release lib·전체 nextest·Native Skia 공식 묶음·doctest를 종료 코드로 확인했다. 혼합 컷·바깥여백·차트는 같은 바이너리에서 수정 전 101/수정 후 0도 확인했다.
- Studio Node 2,597 PASS / 0 FAIL, 실제 fresh 글꼴 추적·production build·브라우저 3종을 모두 종료 코드 0으로 확인했다. 수정한 스크립트 계약은 39 PASS / 0 FAIL / 1 skipped다.
- 이 시점에는 넓은 대조군의 TSV 원문 840개와 API 입력·Print 각 33개를 준비했다. 아래 최종 제출 단계에서 PR 관련 조건만 선별하고 부정확한 대표 연결을 교체했다. 원문 로그와 중간 run manifest는 커밋하지 않았다.

[전체 실행·원문·직접 판독·원 PR별 근거](../pr/assets/s21_20261027/README.md)를 따른다. raw와 RGB 진단값을 구분하며 기존 glyph·선 농도 잔차를 완전 일치로 주장하지 않는다. 원격 CI·통합 merge·사용자가 지정한 옵션 2 기록 PR과 오늘할일·원 PR별 comment/close·정리는 실제 수행 결과로 별도 확정한다.


## PR 주장에 맞춘 최종 증적 선별

PR과 관계없는 초기 일반 대조군을 제출 자료에서 제거했다. 변경 전 base의 90% 미만은 비교 자료로 허용하며 변경 후 후보의 미달은 허용하지 않는다. 현재 제출은 Native/fresh WASM 각각 **197조건·256쪽**, 최저 raw **93.22230% / 93.52322%**다. 전체 관련 쪽의 90% 미만·누락·측정 불가·독립 Print 쪽수 불일치와 같은 base 대비 0.1%p 초과 하락은 모두 0쪽이다. #8188의 별도 512쪽 raw100·정수 미일치0/0은 유지한다.

45개 PR의 의미를 기준으로 39개는 실제 입력·편집 후 상태·페이지의 대표 review/overlay를 연결했다. 6개는 원시 레코드·조회·함수·빈 줄 계약의 실제 실행을 연결하고 다른 문서의 PNG를 해당 결함의 증거로 사용하지 않았다. #8090은 두 구역을 캐시한 뒤 앞 구역 미주를 편집하는 실제 fresh WASM 세션에서 마지막 쪽의 편집 대상·캐럿을 확인했다. #7904는 실제 제품 입력 처리·CommandHistory·WasmBridge와 WASM092에서 확정·취소·Undo/Redo를 실행했다. UI this/cursor stub의 실행을 실제 브라우저 이벤트 자동화로 보고하지 않는다. #8035의 세 삭제 시나리오는 개체 없는 독립 문단의 글자 모양 경계와 비교했다. 같은 편집 저장본 6개의 MCP 2020 Print와 Native/fresh WASM은 전쪽 raw100이다.

이 추가 확인은 기존 원 PR의 재현 동작만 보완한 것이며 다른 실제 HWP 문서로 범위를 확대하지 않았다. 전체 Rust·Studio 코드는 최종 검증 source 04c92da78165ccbf63b467855591df183655a98d에서 바꾸지 않았다. [최종 주장·쪽·실행 색인](../pr/assets/s21_20261027/README.md)을 따른다. 현재 제출용 실루엣 TSV 원문은 base/후보 각각 197조건×2경로와 #8188 8개로 총 **796개**다. 원격 제출·CI·merge는 실제 수행 뒤 별도로 기록한다.
