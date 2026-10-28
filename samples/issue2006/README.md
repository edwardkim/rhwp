# Issue #2006 검증 샘플

## 1790387_prep_final_report.hwpx

- 출처: hwpdocs 코퍼스 `prism_downloads/질병관리청/1790387-202500020_D0150004-1-001_HIV
  노출 전 예방요법(PrEP) 수요자 타당도 검증 및 질병부담 연구 최종결과보고서.hwpx`
  (PRISM 정책연구 공개 보고서, 원본 그대로 복사).
- 원본 마지막 저장 제품은 한컴2024다. 현재 독립 기준은 동일 원본을 HWP MCP
  `--engine 2024`로 출력한
  [`1790387_prep_final_report-2024.pdf`](../../pdf/issue2006/1790387_prep_final_report-2024.pdf)의 **140쪽**이다.
  원본/PDF SHA와 변환 출처는 [검토 기록](../../mydocs/pr/archives/pr_7406_review.md)에 있다.
- KoPub 내장 2020 PDF의 140쪽은 보조 대조군으로 보존한다. KoPub 미설치 환경의
  2022 PDF 146쪽은 다른 출력 환경의 자료이며 현재 페이지 수 기준이 아니다.
  PDF Producer나 연도만으로 기준을 선택하지 않는다.
- PR #2082의 130→141쪽은 당시 전면 TAC 그림 스택 개선 결과다. 보정59는 본문 표
  안 여백과 그림/빈 후속 줄의 저장 프레임을 복원해 140쪽으로 맞췄다. 전체 140쪽의
  Native/fresh WASM 비교, 글꼴 공급, 직접 판독과 검사 근거는
  [보정59 증적](../../mydocs/pr/assets/pr7382_20260926/stage59_prep2006_validation.json)에 있다.
  원본 문서·140쪽 기대값은 유지했고 새 검사 함수는 추가하지 않았다.

저장소 루트의 review 작업 환경에서 기존 검사를 실행한다. wrapper는 파생 suite를
동적으로 찾고 `cargo nextest`를 사용한다. 파생 suite는 커밋하지 않는다.

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/run-rust-test.mjs issue_2006_1790387_prep_pagination_pin -- \
  --locked --cargo-profile release-test --target-dir target/pr-review \
  --test-threads 8 --no-fail-fast
```

페이지 수 검사는 전체 쪽수만 확인한다. 시각 판정은 동일 원본/PDF의 전체 페이지
Native/fresh WASM Visual Sweep과 영향 쪽의 직접 review/overlay를 함께 확인한다.
