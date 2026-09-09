# #6950 Stage 1 — 앵커 줄 보정의 원인 계보와 보호 범위

- Issue: [#6950](https://github.com/edwardkim/rhwp/issues/6950)
- 작성일: 2026-09-09
- 브랜치: `task_m100_6950`
- 제품 기준: `13c92feb67d2bf0ae62349f41c5f5cd83845a4a5`
- 조사 시작 HEAD: `2450a645517d8e765f81c6fe7defe4c07f08ab6c` (기준 이후 문서·fixture만 추가)
- 상위 문서: [수행계획](../plans/task_m100_6950.md)
- 상태: 역사 실행과 기존 보호 조건 조사 완료. 제품 구현은 하지 않았으며 [구현계획](../plans/task_m100_6950_impl.md) 승인 대기.

## 1. 결론과 판정의 한계

**현재 겹침은 속성 파싱 소실이 아니라, 일반 표 배치와 공간 예약에서 앵커 줄 보정을 선행 TAC
형제가 있는 문단으로 제한한 조건에 의해 재현된다.** 같은 입력을 과거 코드로 실행했을 때
`980c80203` 직전에는 본문 아래였던 표가 해당 커밋부터 본문 위로 올라온다.

다만 이 결함은 보정 확장 이전에도 있었다. 정확한 계보는 **기존 미해결 → 보정 확장으로 해소 →
다른 문서의 회귀를 막기 위한 제한으로 재발**이다. `980c80203`이 최초 발생 커밋이라고 하거나,
어느 배포판에서 처음 회귀했는지까지 확인했다고 말하지 않는다.

기존 제한을 없애면 #6718의 4쪽 넘침이 3.2px에서 73.6px로 악화된다. 따라서 조건을 삭제하거나
`tac_sibling_float_anchor_offset_px`를 모든 문단에 무조건 적용하는 수정은 채택하지 않는다.

## 2. 입력과 현재 증상

- 원본 fixture: `samples/hwpx/20260909-para-table.hwpx`
- SHA-256: `cbf2ee7235861e93011d80834bfc49525349f6776c929b98ebd4f06003581d07`
- 한컴 2024 PDF: `pdf/hwpx/20260909-para-table-2024.pdf`
- PDF SHA-256: `c24244980c2c428b06575a1d948349a0971ba168898956a2bbfcccda816dc5ad`
- MCP 출력의 엔진·입력 전처리·폰트 범위는 수행계획 8절에 기록했다. 재변환하지 않았다.

대상은 section 0 / paragraph 1 / control 0(모두 0 기준)의 4행×3열 표다. 앞의 표 세 개는
paragraph 0에 속하며, **같은 문단의 선행 TAC 형제가 아니다**.

원본과 CLI dump에서 `TOP_AND_BOTTOM`, `treatAsChar=0`, `flowWithText=1`, `allowOverlap=0`,
`vertRelTo=PARA`, `vertOffset=2896`을 확인했다. 표 제어 문자는 174자 텍스트 끝에 있다.

| 항목 | 값 |
| --- | --- |
| 저장 줄 시작 문자 위치 | 0 / 47 / 88 / 132 |
| 저장 줄 vpos (HWPUNIT) | 31880 / 34064 / 36248 / 38432 |
| 현재 텍스트 줄 상단 (96 DPI, px) | 474.0 / 503.1 / 532.3 / 561.4 |
| 현재 표 상단·높이 (px) | 516.4 / 178.6 |
| 첫 줄과 앵커 줄의 차이 | 6552 HWPUNIT = 87.36px |
| 표 폭 + 좌우 바깥 여백 | 47622 + 566 = 48188 HWPUNIT (줄 폭과 같음) |
| 다음 빈 문단의 저장 vpos | 55291 HWPUNIT |

저장 값은 `38432 + 2896 + 13397 + 566 = 55291`로 일치한다. 현재 표 상단은
`474.0 + 2896/75 + 283/75 ≈ 516.4px`로 설명된다. 마지막 앵커 줄로 기준점을 옮기는 후보는
약 603.8px다. **603.8px는 과거 rhwp 실행값이기도 하지만 한컴 PDF에서 측정한 절대 좌표가 아니다.**
원본에 별도의 다섯 번째 LineSeg가 있다고 해석하지 않는다.

## 3. 동일 입력의 역사 실행

### 3.1 방법

기존 공개 fixture 세 개만 같은 Linux 환경·CLI 옵션으로 실행했다. 코퍼스 전수 조사, 입력 재저장,
합성 fixture 생성, 제품 코드 수정은 하지 않았다. 각 소스 checkout에서 `--locked`로 빌드했다.

```bash
cargo build --locked -p rhwp --bin rhwp \
  --manifest-path <해당-소스-checkout>/Cargo.toml \
  --target-dir /home/edward/mygithub/rhwp-shared-review-target -j 2
node output/6950/history-probe.mjs <측정명> <해당-소스-checkout>
```

진단 스크립트는 `export-render-tree <입력> -p <0기준쪽> -o <출력>`과
`dump <입력> --section 0 --para <문단>`을 실행한다. 각 `summary.json`에 소스 SHA,
실행 파일 SHA-256, 실제 인수, 표·텍스트 bbox와 본문 TextRun 하단 초과량을 기록했다.
이는 집중 좌표 진단이지 기존 전체 회귀 테스트의 실행을 대체하지 않는다.

| 측정명 / 소스 | 이번 표 상단 | #6718 4쪽 본문 하단 초과 | #6879 7쪽 라벨 / 후행 표 상단 |
| --- | ---: | ---: | ---: |
| `before-extension` / `1f591b5cad` | 516.4px | 3.2px | 909.1 / 128.0px |
| `before-guard` / `672eb9f649` | 603.8px | 73.6px | 98.2 / 182.1px |
| `after-guard` / `980c802035` | 516.4px | 3.2px | 98.2 / 182.1px |
| `current` / `2450a6455` (제품 `13c92feb67`) | 516.4px | 3.2px | 98.2 / 182.1px |

정확한 SHA:

- 보정 확장 `9d91bd1c9`의 부모: `1f591b5cad5dccb38a45a20ea6443da6feda9f11`
- 제한 추가 `980c80203`의 부모: `672eb9f649497e54aa09146f1d83213eff9838bb`
- 제한 추가: `980c802035dee6f36b0585816236b39aee45cd87`

로컬 증적은 `output/6950/history/<측정명>/`에 있다. 이 디렉터리는 Git 제출물이 아니므로
핵심 값과 명령을 본 보고서에도 남긴다. `before-guard`와 `after-guard`는 인접 커밋 비교다.
TAC 사례 #6879는 제한 이후에도 개선이 유지되는 반면, TAC 형제가 없는 이번 샘플은 개선을 잃는다.

### 3.2 실행 파일 재사용 정정

과거 checkout들이 같은 Cargo target을 사용한 뒤 주 checkout에서 통상 `cargo build`만 실행했을 때
0.13초 만에 끝났지만 실행 파일은 `before-extension`의 해시와 같았다. 처음 만든 `current`
좌표 기록은 현재 버전 증거에서 제외했다. #6879의 라벨 위치까지 과거 결과와 같아 교차 검사로 발견했다.
현재 복원은 별도 컴파일 인자로 library 재컴파일을 유도하고 기본 CLI 빌드를 다시 수행한 뒤 확인했다.
향후 A/B는 source SHA뿐 아니라 실행 파일 해시와 보호 사례의 동작까지 확인한다.

```bash
cargo rustc --locked -p rhwp --lib \
  --target-dir /home/edward/mygithub/rhwp-shared-review-target -j 2 \
  -- -C metadata=task6950-current-verify
cargo build --locked -p rhwp --bin rhwp \
  --target-dir /home/edward/mygithub/rhwp-shared-review-target -j 2
node output/6950/history-probe.mjs current /home/edward/mygithub/rhwp
```

두 컴파일은 각각 1분 07초·1분 21초, 종료 코드 0. 복원 CLI SHA-256은
`0754a711ef2d7d990dac020a99e46ec1eb90896246d5621ea3ad6737f21eb916`이며 위 표의 현재 결과를 얻었다.
무효 기록은 `output/6950/history/rejected-current-stale/`로 분리했다.
임시 detached worktree `/tmp/rhwp-6950-history.vVvqeqZk`는 clean 확인 후 `git worktree remove`로
제거했다. 소스는 해당 커밋에서 재구성할 수 있고, 원래 checkout·공유 캐시·측정 증적은 보존했다.

## 4. 왜 제한이 추가되었는가

`9d91bd1c9`는 기존 분할 경로의 저장 앵커 해석을 일반 배치·공간 예약에도 넓혔다.
`980c80203`은 그 확장이 #6718을 악화시킨 실행 결과를 근거로 같은 문단에 선행 TAC 형제가
있을 때만 보정하도록 좁혔다. 현재 주요 호출 위치는 다음과 같다.

| 위치 | 역할 |
| --- | --- |
| `layout.rs::stored_float_anchor_line_top` | 제어 문자의 원본 위치를 저장 줄에 대응 |
| `layout.rs::stored_float_anchor_offset_hu` | 저장 첫 줄부터 앵커 줄까지의 거리 산출 |
| `layout.rs::tac_sibling_float_anchor_offset_px` | 선행 TAC가 없으면 0 반환 |
| `layout.rs::layout_table_item` | `para_y_for_table`에 보정하여 실제 표 배치 |
| `typeset.rs::place_table_with_text` | 같은 보정으로 표 상하단을 예약하여 후속 흐름에서 제외 |

**도입 이유와 주석의 상세 원인 설명은 구분해야 한다.** 제한 커밋은 바로 앞
`rewind_anchor_snapped`를 무효화했다고 설명한다. 그러나 해당 스냅은 코드상 **빈 호스트**에만
적용되며, #6718의 paragraph 23은 실제 텍스트가 있는 문단이다. 따라서 이 스냅이 그 문단에서
실제로 실행되었다고 확정하지 않는다. 확인한 것은 일반 보정의 70.4px 추가가 실제 악화를
발생시켰다는 점이며, 해당 사례의 정확한 런타임 기준점 변환은 다음 절의 확인 대상이다.

## 5. 수정 설계에서 빠뜨리면 안 되는 반증

#6718도 단순한 저장 줄 검증을 통과한다.

- 호스트 paragraph 23의 저장 vpos는 `9120 / 11760 / 14400`으로 증가한다.
- 텍스트 끝 앵커, 선행 TAC 없음, 양수 offset 3518인 자리차지 표다.
- `14400 + 3518 + 33297 + 566 = 51781`로 다음 문단 vpos와 정확히 일치한다.
- 실제 4쪽 텍스트 상단은 235.0 / 270.2 / 305.4px이고 기존 표 상단은 378.5px다.
- 앞 문단 22는 이전 쪽에서 이어진 부분 문단이며, 저장 줄 5에서 vpos가 0으로 재시작한다.
  이번 샘플의 앞 문단 세 표 배치에는 같은 continuation이 없다.

따라서 **호스트 LineSeg 증가 여부 + 다음 문단 좌표 일치만으로 적용 대상을 구분할 수 없다.**
반대로 ‘직전 문단이 continuation이면 항상 제외’라는 새 우회 조건도 아직 검증된 규칙이 아니다.
원본 좌표와 현재 쪽 흐름 좌표를 같은 기준으로 변환하고, 현재 표 기준점이 앵커 진행량을 이미
포함하는지 확인해야 한다. 문서명, 문단 번호, 버전 번호, 임의의 px 임계값으로 두 샘플을 나누지 않는다.

## 6. 표준 시각 증적과 검증 범위

수행계획 9절의 표준 출력을 재사용했다.

- 나란히 비교: `output/6950/visual-standard/para-table/compare/compare_001.png`
- 겹침 비교: `output/6950/visual-standard/para-table/overlay/overlay_001.png`
- 통합 검토: `output/6950/visual-standard/para-table/review/review_001.png`

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 약 6.15%.
높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다

한컴 출력에서는 본문 아래에 표가 있고 rhwp에서는 겹침을 확인했다. 선행 표 간격·폰트 등도
수치에 포함되며, 이 점수 전체를 이번 결함으로 귀속하지 않는다. 이번 단계에서는 과거 버전마다
PNG를 재생성하거나 Studio WASM을 교체하지 않았다. 현재 표준 출력은 CLI SVG의 webfont raster다.

## 7. 다음 단계와 보호 불변식

1. 이미 진행된 앵커 거리와 아직 반영되지 않은 거리를 구분하는 배치 기준점을 확인한다.
2. 동일 판단을 일반 배치와 공간 예약에 적용한다. 그림만 아래로 이동시키는 보정은 금지한다.
3. 이번 본문-표 겹침뿐 아니라 뒤 문단이 표 안으로 들어오지 않는지 확인한다.
4. #6718의 기존 넘침 상한과 #6879의 TAC 라벨·후행 표 순서를 유지한다.
5. #6860의 분할 경로, #5807/#2439 등 기존 연관 보호 테스트를 재사용한다.
6. 실제 입력의 신규 회귀 테스트는 `tests/cases/`에만 작성한다. 원본 재저장·합성 정답지 생성은 하지 않는다.

현재 단계에서 확인한 인접 커밋 회귀는 확정 사실이다. 모든 배치 맥락을 안전하게 구분하는
일반 판정식은 아직 구현·검증되지 않았다. 세부 절차·중단 조건은 구현계획으로 분리한다.

## 8. 용어

- **TAC (Treat As Character)**: 글자처럼 취급. 여기서 문제 표는 TAC가 아닌 자리차지 표다.
- **앵커 줄**: 표 제어 문자가 속한 문단의 저장 줄. 표 자체의 첫 행과 다르다.
- **LineSeg (Line Segment)**: 원본 문단의 줄별 문자 시작 위치·세로 위치 등 저장 조판 정보.
- **HWPUNIT**: 1/7200인치. 96 DPI에서는 75 HWPUNIT가 1px다.
- **vpos (Vertical Position)**: 저장된 세로 위치. 좌표 기준 변환 없이 화면 절대 y와 같다고 볼 수 없다.
- **continuation**: 이전 쪽에서 시작한 문단·표가 다음 쪽에서 이어지는 상태.
- **공간 예약 / exclusion**: 표가 차지하는 구간을 기록하여 후속 텍스트가 그 안에 배치되지 않게 하는 처리.
- **bbox (Bounding Box)**: 렌더 결과의 외접 사각형. 위치와 크기를 비교하기 위한 값이다.
