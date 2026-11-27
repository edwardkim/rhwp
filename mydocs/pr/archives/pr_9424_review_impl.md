# PR #9424 수식 선택 보완안

## 재현

검토 대상은 cozykbin 님의 첫 기여 `1e9194008e`를 최신 `devel`의 `3fdaa04258`에 충돌 없이 병합한 `c765fa2ac9`다. #9139의 원본과 셀에서 정방향·역방향 16가지 드래그를 검사했다. 제출된 수정으로 둘째 줄과 셋째 줄의 선택 표시가 복원된다. [원 PR](https://github.com/edwardkim/rhwp/pull/9424) · [드래그 비교](../assets/pr9424/wrapped-selection.png)

1. rhwp의 공개 API로 새 문서에 `next`를 입력하고 맨 앞에 인라인 수식 `x+1`을 삽입해 HWPX로 저장한다. 이 파일의 첫 문단에는 정상 저장 과정에서 줄 정보가 생략된다. 동일한 파일을 양쪽 Studio에서 다시 연다. [저장본](../assets/pr9424/core-export-inline-equation.hwpx)
2. 글자에 캐럿을 두고 `Home`, `Shift+Right`를 누른다. 두 버전의 논리 선택은 모두 0–1이다. 최신 `devel`은 수식 폭을 덮는 35.1 CSS px 사각형을 표시한다. PR 후보는 사각형 0개를 반환한다.
3. `Home`, `Right`, `Shift+Right`를 누르면 두 버전 모두 1–2를 선택한다. `devel`은 수식 다음 한 글자 폭인 9.2 CSS px를 칠하고, 후보는 수식까지 포함한 44.4 CSS px를 칠한다. [같은 키 입력의 화면 비교](../assets/pr9424/equation-selection.png)
4. 저장소의 `samples/hwpx/eq-002.hwpx`는 저장된 줄이 있으면 양쪽에서 기존 동작을 유지한다. 문단 1의 선택적 줄 정보만 생략한 대조본에서 같은 두 차이가 재현된다. [공개 표본](../../../samples/hwpx/eq-002.hwpx)

## 원인과 보완 범위

`get_selection_rects_for_target_native`는 합성 줄만 있는 문단에서도 `collect_rendered_line_starts`를 호출한다. 이 함수는 줄의 `TextRun`과 인라인 `FormObject`에서 시작 위치를 얻는다. 실제 캐럿 탐색은 앞쪽 `Equation`도 논리 한 칸으로 센다. 수식으로 시작하는 줄에서 선택 쿼리의 시작이 1로 밀린다. [줄 시작 수집](https://github.com/edwardkim/rhwp/blob/1e9194008e7b599b4fb9777a88826ec2ba79f392/src/document_core/queries/cursor_nav.rs#L2251) · [캐럿 탐색](https://github.com/edwardkim/rhwp/blob/1e9194008e7b599b4fb9777a88826ec2ba79f392/src/document_core/queries/cursor_nav.rs#L1930)

보완은 선택 사각형과 캐럿이 줄에 실제로 놓인 글자·인라인 수식의 논리 경계를 함께 사용하도록 하는 범위다. 각 줄에서 해당 문단 또는 셀이 소유한 개체를 확인한다. 승인 시 원 기여를 보존하고 공개 API 회귀와 Studio 마우스·키보드 조작으로 자동 줄바꿈, 수식 단독 선택, 뒤 글자 선택, 저장 줄이 있는 대조군을 재검증한다.

공개 WASM 패키지로 같은 저장본을 확인하는 명령은 아래와 같다. 수식을 단독 선택할 때 사각형이 빠지면 종료 코드 1을 반환한다. 새 입력은 `--make`를 붙여 별도 경로에 만들 수 있다.

```sh
node mydocs/pr/assets/pr9424/equation-probe.mjs pkg mydocs/pr/assets/pr9424/core-export-inline-equation.hwpx
```

## 보완 결과

작업지시자 승인 뒤 `a37c9cce46`에서 캐럿·수식의 공통 주소를 보완하고 새 회귀 검사에서 전후 실패·통과를 확인했다. 본문·셀 드래그 16건과 수식·글자 분리의 결과는 최종 검토에 기록했다. [최종 검토](pr_9424_review.md)
