# PR #8410 리뷰 — PDF 패턴 이미지와 문자 외곽선 보존

## 최종 판정

**승인 — #8403의 패턴 이미지·문자 외곽선 수정, 저장 왕복과 시각 검증을 완료했다.**
2026-10-31 20:58 KST에 code head의 모든 관련 CI와 정책 감사 성공을 확인했다.
아래 판정은 code head에 대한 self-review다. 이 기록을 포함하는 문서 trailing head의
CI·fast-pass 판정과 최신 devel 병합 가능성을 확인한 뒤 승인된 머지를 수행한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#8410](https://github.com/edwardkim/rhwp/pull/8410) / edwardkim / devel |
| 관련 이슈 | [#8403](https://github.com/edwardkim/rhwp/issues/8403) |
| 검증한 code head | `2e37e8f15025832c94dbc157061dd2ad1ddb261d` |
| 최신 기준 SHA | `99a176b7cb34fde54454c03433baa1382b2de9fa` |
| Native 생산 코드 | `21a4902f142bc424c7a941e6328fb61a5f34b9e5` |
| fresh WASM 생산 코드 | `b13334f327193b7a6ed76c9f501f6fdee71c5e11` |
| 경로·승인 | collaborator self-merge, reviewer 미지정. 사용자가 PR·self-review·merge를 승인 |

두 생산 커밋 사이의 차이는 테스트 helper의 cfg(test) 소속이며 제품 코드는 동일하다.
후속 변경은 검증 하네스·문서·공개 입력 첨부다. 바이너리·HTTP 자산 해시와 명령은
[최종 검증 기록](../../tech/investigations/issue-8403/validation/rebase/README.md)에 보존했다.

## 변경과 self-review

- coloured tiling pattern의 이미지가 추출되지 않고 검정 fill이 남던 문제를 공통 walker에서
  수정했다. page·Form·pattern의 Matrix, BBox, step과 clip을 적용하며 source 순서를 유지한다.
  cycle·깊이·tile·operator 및 byte 예산을 우회하지 않는다.
- PDF `Tr` 1·2의 채움·획 색, 폭, alpha, join·miter를 TextPaint와 공통 resolved style/paint를
  통해 Native·SVG·Canvas2D·CanvasKit에 전달한다. 글자별 임의 bold나 줄 높이 보정으로
  실루엣 점수를 맞추지 않았다. 비지원 affine·dash·glyph clipping은 명시적으로 거부한다.
- HWP/HWPX 확장은 version 1, 1 MiB, 8,192 records와 style fingerprint·ordered IDs로 제한한다.
  PDF Worker handoff에서 논리 원본을 유지해 crop 기준 크기 손실을 막는다.
  외부 편집기의 기본 표현과 RHWP 확장 보존은 구분하며 `MetadataReduced` 안내를 유지한다.
- 변경은 paint·이미지 추출·저장 경로다. 줄 구성·pagination·rowspan의 컷이나 예약 높이를
  변경하지 않아 분할 조판 재설계는 비해당이다. 같은 공통 paint를 각 backend가 소비하는지,
  획을 더하면서 기준선과 앞뒤 본문 위치가 바뀌지 않는지 확인했다.
- 기존 SVG 검사 helper는 nested crop viewport를 관측하도록 보정했다. 기존 배치 기대값과
  허용 오차를 완화하지 않았다. 공개 합성 PDF 연산자 계약과 실물 시각 증거를 구분했다.

통합을 막는 결함은 발견하지 못했다. 세부 구현·예산·저장 제약은
[완료 보고서](../../report/task_m100_8403_report.md)를 따른다.

## 입력과 시각 검증

[입력 manifest](../../tech/investigations/issue-8403/validation/inputs/manifest.json)의
공개 HWP/HWPX 6개가 검토 commit에 포함되어 있고, 첨부된 동일 바이트를 fresh WASM에서
다시 열어 38쪽을 검증했다. 공개 PDF의 독립 기준은 직접 작성한 PDF 연산자와 Poppler/MuPDF다.
이를 한컴에서 인쇄한 PDF라고 표시하지 않는다. 사용자 비공개 원본과 로컬 글꼴은 게시하지 않았다.

| 범위 | Native 최저 raw | fresh WASM 최저 raw | 판정 |
| --- | ---: | ---: | --- |
| 원본 8쪽과 HWP/HWPX 저장 왕복 | 94.70% | 94.77% | 전쪽 90% 이상 |
| 공개 패턴 5쪽과 저장 왕복 | 99.98% | 99.98% | 통과 |
| 문자 획 7쪽·14곳 편집과 저장 왕복 | 100% | 100% | 통과 |
| 기존 PDF 12쪽 직접 가져오기 | 99.20% | 98.85% | 통과 |
| #8397 HFT 763쪽 보존 | 96.61% | 99.17% | 각 backend에서 base/candidate 전쪽 PNG 동일 |

[페이지별 base/candidate TSV](../../tech/investigations/issue-8403/validation/rebase/base-candidate.tsv)는
기본 대조 241개 페이지 모두 90% 이상이며 최대 하락 0.02%p, 2%p 초과 하락 0건이다.
추가 HFT 대조는 두 backend 각각 763쪽 바이트 동일로 새 devel 기능 보존을 확인했다.
전체 쪽수·순서·누락·중복과 저장 후 내용도 별도로 검사했다.

실제 Studio 화면과 공개 review/overlay를 확대해 패턴의 빨강·파랑 열과 crop 경계,
문자 획 내부의 빈 공간·한글 모양·기준선·뒤 내용 위치를 대조했다.
[직접 판독 기록과 그림](../../tech/investigations/issue-8403/validation/rebase/README.md)을
점수와 함께 확인했다. 공개 대조군은 실제 CanvasKit도 통과했다. 비공개 원본의 CanvasKit
요청은 일부 글꼴 미준비로 Canvas2D에 fallback된 사실을 기록했고 원본 CanvasKit 성공으로
주장하지 않았다. HFT Native의 기존 제목 글꼴 차이 역시 이번 개선으로 주장하지 않는다.
기존 12쪽 PDF 전체 CLI 변환의 표준 자원 한도는 유지하며, 분할 선택 검증과 구분한다.

## 로컬 검증과 GitHub CI

- 전체 nextest 12,233 PASS·45 skip, Native lib 4,040 PASS·12 ignore.
- Studio 2,700 PASS·기존 skip 2와 TypeScript 통과.
- 신규 첨부 6개를 명시한 보안 20 PASS, 6종 래칫 81 PASS, helper 1 PASS.
- fmt·Native/WASM/workspace lint·manifest·unit tier·문서 링크 통과.
  [명령과 최종 결과](../../tech/investigations/issue-8403/validation/rebase/finish-local.json).

같은 code head에서 아래 run 모두 성공했고 CI Impact Policy도 success였다.

| 검사 | 실행 |
| --- | --- |
| CI: lint·Frontend·Native·Archive A–E·집계 | [41194300790](https://github.com/edwardkim/rhwp/actions/runs/41194300790) |
| CodeQL | [41194300619](https://github.com/edwardkim/rhwp/actions/runs/41194300619) |
| Render Diff | [41194300572, attempt 3](https://github.com/edwardkim/rhwp/actions/runs/41194300572/attempts/3) |
| Adapter | [41194300676](https://github.com/edwardkim/rhwp/actions/runs/41194300676) |
| Proptest | [41194300663](https://github.com/edwardkim/rhwp/actions/runs/41194300663) |

Render Diff 최초 두 실행은 checkout 디스크 부족으로 제품 검사가 시작되지 않았다.
사용자가 수동 재실행한 3차는 코드·workflow 변경 없이 성공했다. KTX 비교는 0.01077%,
biz_plan·tac-case-001은 0% 차이로 통과했고 CanvasKit 검사도 성공했다.
로컬에서 준비했던 디스크 정리 보정은 PR에서 제외했다. 실패 당시 디스크를 채운 경로는
미확정이며, 재실행 성공을 그 근본 원인 해결의 증거로 삼지 않는다.

## 남은 통합 조건

문서-only trailing head에서 실제 CI 재사용과 필수 검사 success, 최신 base와 merge-tree,
원격 head 일치를 확인하고 정상 squash merge한다. 관리자 우회는 사용하지 않는다.
원격 후속 댓글은 현재 승인 범위에 포함하지 않았다. merge 뒤 devel 동기화·이슈 상태 확인과
작업 전용 산출물 정리를 수행한다.
