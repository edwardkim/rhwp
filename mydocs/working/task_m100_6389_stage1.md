# Issue #6389 단계 1 — 현재 잔여 범위 확인과 재검증

Issue: [#6389](https://github.com/edwardkim/rhwp/issues/6389)

## 분석

- 사용자 지시: 오래된 이슈부터 한 건씩, 분석 → 코드 수정·검증 → 결과보고 → 커밋 순서.
- 착수 base: `5720d3f1646d6c25e51c8d7cbb0b4cc6bc5feb4f`, branch `codex/issue6389-kopub-20260916`.
- 원 이슈는 KoPub돋움체 face의 1.0em 오측정, 저장 줄 셀의 넘침과 재조판 +1줄을 보고했다.
- 현재 face 상수는 이미 0.872em이다(PR #6484 → 통합 #6490). 저장 줄 압축도 #6412의 고유 변경이 통합됐다.
  이슈 마지막 코멘트의 압축 미완료 설명과 실제 반영 이력을 대조해야 한다.
- 남은 논점은 원래 폰트가 설치된 PDF와 대체 폰트 PDF의 환경 차이이다. 상수를 다시 바꾸거나
  특정 샘플 이름으로 치환 환경을 추정하지 않는다. 원 증상이 현재 재현되는지부터 확인한다.
- 기존 입력·PDF 경로를 재사용하며 사본을 추가하지 않는다.

## 검증 계획

1. 저장 줄 압축·face 폭의 현재 구현과 병합 출처를 확인한다.
2. 최신 source로 기존 #6389 집중 검증과 편람 p68 Visual Sweep을 수행한다.
3. 증상이 남으면 원인 계층을 보정하고 해당 반례·영향 범위를 검증한다. 이미 해결된 경우에는
   불필요한 생산 코드 변경 대신 재검증 근거와 미완료 환경 문제를 명확히 기록한다.
4. 실제 실행 결과를 이 문서에 채우고 결과보고 후 함께 커밋한다. 다른 이슈는 이번 회차에 포함하지 않는다.

## 현재 확인 결과

### 기존 수정의 반영 이력

- 저장 줄 압축: #6412의 변경 `810db7289`가 통합 commit
  `28057f02ea5731cbc0566f67eb9c0129e47287e6`에 포함됐다.
- KoPub돋움체 폭: #6484 → #6490, merge
  `3b301f725ab48985f19c25784b68448ed4257bfd`. 현재 `kopub_char_width`는 0.872em이다.
- 따라서 #6490 후속 코멘트의 “저장 줄 소비 시 맞춤 압축이 남았다”는 설명은 현재 코드와 다르다.
  실제 미완료 범위는 설치/미설치 환경에 따른 명시적 조판 폰트 선택이다.

### 이번 변경

기존 `issue_6389_cell_stored_ladder_compresses_to_fit.rs`의 셀 경계 검증을 유지하면서,
한컴 PDF에서 독립 추출한 대상 셀 16줄의 줄 경계와 전체 텍스트를 함께 고정했다.
특히 `※ 분리등록한 첨부물의 경우` 문단은 3줄이어야 한다. 16줄에는 내부 예시 표 2줄도
포함되므로 누락이나 중복도 검출한다. PDF 단어 추출이 삽입하는 공백만 정규화한다.
기존 8px trailing space 허용치를 늘리지 않았고 생산 코드·baseline을 바꾸지 않았다.

### 입력과 기준

이미 Git에 있는 파일을 그대로 사용했다. HWP/HWPX/PDF 사본을 추가하지 않았다.

| 자료 | SHA-256 |
|---|---|
| `samples/2025 행정업무운영 편람(최종).hwp` | `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f` |
| `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2020.pdf` | `6c7be7602cb92bb9b5e6a0b66e9cd80700fceabeade89fabd1a0fcd32adc4413` |
| `pdf/2025 행정업무운영 편람(최종)-2010-no-ttf.pdf` | `c33fbf6be76ae43e8254321ec99b5ab3963e492ff4b6179d7c9717ecac2a05e0` |

새로 빌드한 CLI의 `info --json`에서 HWP 저장 제품은 `hancom-office-2024`, 버전은
`13.0.0.3622`로 확인했다. 이번에는 기존 KoPub PDF의 실측 근거를 재사용했으며 PDF를
새로 변환하지 않았다. 현재 한컴 2024 엔진 출력과 동일하다는 뜻은 아니다.

### 시각 검증

- source `5720d3f1646d6c25e51c8d7cbb0b4cc6bc5feb4f`에서 별도 `release-test` CLI를 빌드했다.
- 검증 worktree: `/Users/tsjang/rhwp-issue6389-review-20260916`.
- 전용 target: `/Users/tsjang/rhwp/target/issue6389-20260916`.
- Visual Sweep: Chrome/webfont, 96 DPI, HWP p68 / KoPub PDF p68.
- 직접 이미지 확인: 원래 `○` 문단들의 우측 셀 넘침이 없고, 대상 셀 16줄과 `※` 문단
  3줄이 유지된다. 글꼴 잉크·자간, 내부 예시 표와 뒤 문단의 세로 위치 차이는 남는다.
  페이지 전체의 픽셀 일치나 전체 문서의 해결을 주장하지 않는다.
- 자동 진단: flagged 0, pixel match 91.4395%, ink proxy 79.25197%.
  위 수치만으로 통과 판정하지 않았으며 직접 비교 범위를 함께 기록했다.
- `no-ttf` PDF의 동일 내용은 **p69**다. p68 대 p68 최초 비교는 서로 다른 내용이므로
  유효한 정확도 비교에서 제외했다. p69를 별도로 렌더해 대상 내용을 확인했다.
  `pdffonts -f 69 -l 69`에서 `Haansoft Batang`만 확인되었다. KoPub PDF p68에는
  `KoPubDotumLight/Bold`, `KoPubBatangLight`가 있다. 대체 폰트의 잉크·페이지 구성은
  KoPub 출력과 다르며, no-ttf 환경 재현 통과로 처리하지 않는다.

증적: [p68 Visual Sweep](assets/issue6389-20260916/p068-kopub-review.png),
[실행 provenance](assets/issue6389-20260916/run_manifest.json),
[overlay 수치](assets/issue6389-20260916/overlay_metrics.json),
[한컴 PDF에서 추출한 줄](assets/issue6389-20260916/pdf-lines.json).

```bash
/Users/tsjang/rhwp/venv/bin/python scripts/visual_sweep.py \
  --file-target issue6389 'samples/2025 행정업무운영 편람(최종).hwp' \
  'pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2020.pdf' \
  --rhwp-bin /Users/tsjang/rhwp/target/issue6389-20260916/release-test/rhwp \
  --pages 68 --dpi 96 --out /private/tmp/issue6389-sweep
```

### 집중 검증

- 기존 테스트: 1 passed, 0 failed, 218 filtered, 0.33초.
- 16줄 계약을 추가한 테스트: 1 passed, 0 failed, 218 filtered, 0.34초.
- PDF XML의 16줄과 Rust 기대값을 별도로 대조: 공백 정규화 후 16/16 일치.
- 생산 코드 복원 후 최종 재빌드·집중 테스트: 1 passed, 0 failed, 218 filtered,
  0.36초, exit 0. [최종 실행 로그](assets/issue6389-20260916/focused-test.log).
- 잘못된 1.0em을 검증 worktree에서 임시 복원한 검증: 0 passed, 1 failed, exit 101.
  기존 셀 우변 검사는 통과했지만 `※` 문단이 3→4줄(전체 16→17줄)이 되어 새 줄 경계
  assertion에서 실패했다. [실패 로그](assets/issue6389-20260916/mutant-test.log).
  생산 코드는 즉시 HEAD로 복원했으며 임시 변경은 후보에 포함하지 않는다.
- 변경 Rust 파일 `rustfmt --check`, `git diff --check`, 문서 상대 링크 검사 통과.
- 테스트 내용 변경으로 파생 harness hash가 달라진 초기 manifest 검사는 drift를 보고했다.
  검증 worktree에서 `--prepare`를 다시 실행한 뒤 고정 base SHA로 `--check` 통과:
  1,337 sources / 5,770 static test attrs / 48 integration targets. 파생 파일은 커밋하지 않는다.
- 전체 회귀·WASM·3종 Clippy는 이 회차에서 실행하지 않았다. 이번 변경은 기존 테스트와
  검증 문서이며, PR/push 준비 단계의 필수 검증을 통과한 것으로 표시하지 않는다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo test --locked --profile release-test \
  --target-dir /Users/tsjang/rhwp/target/issue6389-20260916 \
  --test regression_suite_005 issue_6389 -- --nocapture
```

## 종료 경계

원래 p68의 넘침과 줄 경계는 현행 코드에서 재현되지 않는다. 다만 원 이슈 댓글에 남은
명시적 폰트 환경 선택 기능은 미구현이며, 86712 미설치 환경의 재조판을 완료했다고 쓰지 않는다.
이 단계 커밋 당시에는 범위를 확인 중이었다. 이후 사용자의 “더이상 묻지 말고 알아서 진행” 지시에 따라
추가 질문 없이 [2단계](task_m100_6389_stage2.md)에서 명시적 폰트 환경 선택의 구현·검증 결과를 기록했다.
#6389를 닫거나 다음 이슈를 시작하지 않았다.
