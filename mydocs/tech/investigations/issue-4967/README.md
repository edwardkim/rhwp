---
kind: investigation
status: active
canonical: mydocs/plans/task_m100_4967.md
last_verified: 2026-08-25
---

# Issue #4967 — W8 font face 교정 qualification

이 디렉터리는 W8 tracker의 첫 process canary인 rank 8 `KoPubWorld바탕체 Light`의 교정 적격성 증거를
보존한다. 현재 단계는 제품 font mapping 변경이 아니라 기존 W3·W4·W5·W7.5 증거의 호환성과 실사용
cohort를 판정하는 query 단계다.

## Stage W8-Q0 경계

- private W3 journal을 다시 parse하거나 10k corpus를 재실행하지 않는다.
- rank 8을 실제로 사용한 문서의 경로·이름·본문·hash는 owner-only local output에만 둔다.
- tracked baseline에는 aggregate, evidence digest와 privacy gate만 남긴다.
- W5 exact/subst/missing Hyper-V ladder는 재사용하며 이번 단계에서 VM을 실행하지 않는다.
- v2 registry와 다섯 runtime projection은 읽기 전용이다.

재현 도구는 `scripts/font_rank8_qualification.py`, 계약 테스트는
`scripts/tests/test_font_rank8_qualification.py`다. local-only 입력이 있는 메인테이너 환경에서 projector를
실행하면 `rank8_private_cohort.json`은 mode `0600`, 공개 baseline은 mode `0644`로 생성된다.

## Stage W8-Q1 current 기준선

W5에서 봉인된 generator로 rank 8 fixture를 재현했다. fixture의 생산 계보는 #4963, qualification 소비
목적은 #4967이며 bytes SHA-256은 W5 ladder와 같은
`f6edc8fc43dfd3256385e9752979c14a7041e50c06d36be47cef6e3486835084`다. font bytes는 포함하지 않는다.

`scripts/font_rank8_trace_baseline.mjs`는 다음을 fail-closed로 고정한다.

- current release native와 Docker WASM의 1,556-record trace canonical byte parity
- `maxCharacters=4096`의 전건 완료와 누락 0
- layout metric entry·match kind·heuristic width source 분포
- 표 셀 28,980 HWPUNIT, 글상자 29,434 HWPUNIT의 실제 content width와 대표 장평·자간별 frame slack
- absolute path·private corpus identity·font bytes·full trace의 tracked output 유입 0

결과 정본은 `rank8_current_trace_baseline.json`이다. Q1에서는 현행 상태만 측정하며 metric DB, fallback,
paint·supply 규칙은 바꾸지 않는다. Canvas2D·CanvasKit actual paint는 trace만으로 관찰할 수 없으므로
`studioSnapshotRequired` 상태를 그대로 보존한다.

## Stage W8-Q2 exact metric 가설

`scripts/font_rank8_metric_hypothesis.py`는 외부 font root의 W5 exact TTF와 현행 registry가 가리키는
`font-kopubworld@1.0.3` OTF·WOFF2를 읽어 다음 경계를 분리한다.

- TTF와 CDN OTF·WOFF2는 bytes·name table·outline identity가 아니다.
- 세 source는 fixture 53개 codepoint의 advance가 같고, TTF와 CDN source의 전체 공통 cmap 25,970개도
  advance mismatch가 0이다.
- CDN OTF와 WOFF2는 26,089개 cmap advance 및 fixture outline digest가 서로 같다.
- current trace의 ratio → letter spacing → justification transform을 1,556건 모두 재생한 뒤 base advance만
  exact `hmtx`로 바꾼다.
- fixed-frame 대표 6축에서 metric capacity crossing의 앞당김·신규 발생은 0이다.

결과 정본은 `rank8_metric_hypothesis.json`이다. Q2는 `layout-metric` 하나만 Q3 검증 대상으로
qualification하며 font identity·paint identity, 배포 권한 또는 제품 변경을 승인하지 않는다.
