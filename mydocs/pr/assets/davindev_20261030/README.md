# davindev PR 3건의 메인테이너 시각 검증

측정일: 2026-10-30. 기준 devel은 `2d535febe00a61585b6770cb95860fee912bbe54`,
출력 코드 후보는 `770b073d24085b056c8a0f4263e2214b8b53ae08`이다.
#8134·#8147의 원 커밋과 #8175의 승인된 충돌 보정을 함께 적용했다.
입력·Print PDF·출력 프로그램·글꼴의 SHA-256은 [provenance.json](provenance.json)에 기록했다.

## 같은 입력의 전쪽 비교

독립 기준은 기여자가 같은 저장본을 한컴 웹한글기안기에서 인쇄한 PDF다.
모든 입력은 1쪽이며 PDF·Native·fresh WASM의 전체 쪽수가 일치한다.
96dpi, compat 2022, print 출력, 색상 경계 보정 전 2px 내용 실루엣
`silhouette_raw_match_percent`를 사용했다. 각 값은 해당 문서의 전쪽 최저값이기도 하다. TSV에 표시된 소수 둘째 자리 값을 기록했다.
TSV 원문은 `base/{native,wasm}/<문서>.tsv`, `candidate/{native,wasm}/<문서>.tsv`다.

| 원본 | base Native | 후보 Native | base WASM | 후보 WASM |
| --- | ---: | ---: | ---: | ---: |
| samples/issue8122/textbox_valign_no_lineseg.hwp | 85.25% | 99.55% | 85.76% | 99.43% |
| samples/issue8118/empty_para_tac_picture.hwp | 89.20% | 99.73% | 89.61% | 99.63% |
| samples/issue8166/form_checkbox_width_1100.hwp | 88.77% | 96.21% | 88.15% | 95.98% |
| samples/issue8166/form_checkbox_width_3000.hwp | 92.10% | 96.27% | 91.89% | 96.05% |
| samples/issue8166/form_checkbox_width_9921.hwp | 96.76% | 96.62% | 96.55% | 96.40% |

PDF는 각각 `pdf/issue8122`, `pdf/issue8118`, `pdf/issue8166`의 같은 이름에
`-webhwp-print.pdf`를 붙인 파일이다. 후보 10개 측정 모두 90% 이상이며,
정상 대조군 9921HU의 최대 하락은 0.15%p다. 넓은 캡션의 내용은 보존된다.

## 실행 환경과 재현

Native는 `cargo build --locked --profile release-test --features native-skia --bin rhwp`로
각 소스를 빌드했다. 후보 WASM은 호스트에서 `scripts/wasm-pack-locked.sh --target web --out-dir pkg`로
최적화까지 새로 빌드했다. Docker도 설치되어 있으나 이번 실행은 공용 Cargo 캐시를 쓰는 호스트 경로다.
base WASM은 `21ba44e3ea75966321d7184e2d94a8620e378013`의 기존 최적화 산출물을 재사용했다.
base까지 Rust·Cargo·vendor·Studio 코드가 동일함과 기존 정식 provenance의 JS/WASM 해시를 확인했다.
후보 산출물을 Studio public에 배치한 뒤 동일 해시를 확인하고 실제 브라우저를 실행했다.

명시적 글꼴 입력은 로컬 `HBATANG.TTF`와 `gulim.ttc` 두 파일이다. 글꼴은 배포하지 않는다.
전체 글꼴 디렉터리를 공급한 초기 진단은 카탈로그 용량 한도에서 한컴바탕이 빠져 fallback을
일으켰다. 위 최종 값은 필요한 파일만 지정하여 base와 후보 모두 다시 측정한 결과다.
SVG 진단은 같은 두 파일을 가리키는 전용 디렉터리를 사용했다.

```sh
VISUAL_SWEEP_CHROME=/path/to/chrome venv/bin/python scripts/visual_sweep.py \
  --file-target form_checkbox_width_3000 \
    samples/issue8166/form_checkbox_width_3000.hwp \
    pdf/issue8166/form_checkbox_width_3000-webhwp-print.pdf \
  --rhwp-bin /path/to/exact-source/rhwp --compat 2022 --dpi 96 --embed-fonts full \
  --font-path /path/to/HBATANG.TTF --font-path /path/to/gulim.ttc \
  --silhouette-only --page-workers 1 --out output/review-native
```

실행 시 사용한 명령의 정확한 옵션은 아래 검토 문서와 대조한다. WASM에는 같은 코드의
패키지를 `--wasm-pkg /path/to/exact-source/pkg`로 지정한다. 전체 compare·overlay·review PNG는 각 backend의
문서 하위 디렉터리에 보존했으며 base와 후보를 구분했다.

## 직접 판독과 남은 차이

- #8134: 기준 PDF와 후보의 두 제목 줄이 상자 안에 들어간다. base에서 아래로 넘쳐 보이지
  않던 둘째 줄이 복원되며, 뒤 본문·배경 그림·하단 작은 글상자의 위치와 내용은 유지된다.
  저장 줄이 없는 실제 입력과 간격·빈 문단을 변형한 정렬 불변식 검사는 구분한다.
- #8147: **최신 base부터 제목 테두리 그림은 이미 정확히 한 번 그려진다.** 그림 bbox와
  소유 줄은 `(96.9, 94.5, 599.9, 91.7)`, 뒤 문단 시작은 y=207.5다. 후보에서도 같다.
  이 문서의 점수 개선과 제목 복원은 선행 #8134의 효과이며 그림 누락의 신규 해결로 해석하지 않는다.
- #8175: 1100HU에서는 체크 표시만 남고, 3000HU에서는 오른쪽 경계의 캡션이 잘린다.
  9921HU에서는 전체 캡션이 남는다. 버튼·목록·라디오·입력 영역과 그 간격은 보존된다.
  Native·WASM·SVG·Studio CanvasKit 출력에서 같은 경계를 확인했다.
- 독립 웹한글 Print와 글자의 굵기·안티앨리어싱, 체크/라디오 아이콘 크기 차이는 남는다.
  base에서도 보이는 차이이며 이번 PR에서 완전히 동일한 출력으로 해결됐다고 주장하지 않는다.
  표/그림 경계와 줄 위치를 글꼴 차이로 면제하지 않았으며 새 px 좌표 핀은 추가하지 않는다.
- 수식은 없는 입력이다. 제목 줄, 캡션의 끝, 앞뒤 내용 간격을 compare·overlay에서 직접 확인했다.

`zoom/`은 같은 좌표의 200% 확대다. 각 패널은 위부터 독립 Print, base Native,
base WASM, 후보 Native, 후보 WASM 순서다. 확대 패널은 진단용이며 전쪽 측정 범위를 바꾸지 않는다.
SVG 추가 진단도 전5쪽 모두 90% 이상이다. 폭 3000HU에서 두 family의 실제 font-face 데이터와
Unicode cmap을 확인했다. 임시 글꼴 디렉터리의 이름을 원 파일명으로 맞춘 후 검증했으며
글꼴을 포함하는 SVG 자체는 정식 증거에 복사하지 않았다.

Studio Chrome 155에서 3개 폭 × screen/print × Canvas2D/CanvasKit compat/default로
18회 캡처했다. backend 간 비교 12건이 모두 통과했고, replay hard-gate 위반·runtime 오류·
이미지 replay 실패는 0건이다. CanvasKit surface는 software였으며 GPU 검증으로 해석하지 않는다.
대표 print 캡처는 `studio/form-<폭>/`에 있다. 성능 수치는 단회 진단이므로 개선/악화의 근거로 쓰지 않는다.

## 검토 기록

- [#8134](../../pr_8134_review.md)
- [#8147](../../pr_8147_review.md)
- [#8175](../../pr_8175_review.md)

이 문서는 시각 근거이며 전체 Rust 회귀·lint 완료나 원격 merge 승인을 대신하지 않는다.
