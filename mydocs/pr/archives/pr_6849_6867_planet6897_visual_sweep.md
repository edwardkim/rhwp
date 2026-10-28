# planet6897 통합 검증 및 시각 증적

## 통합 판정: 메인터너 보정 완료, 수용 가능 (검토 범위 한정)

2026-09-08, 기준 `91147aec332651309029faac521d19106305c6f5`, 체리픽 완료 head(보정 전) `d66f2a6a5aaccc8deae4d77877835ce1a5fdeb86`. CI가 통과한 비초안 PR #6849·#6851·#6853·#6857·#6859·#6867의 8개 커밋을 출처 보존 체리픽했다. 전체 테스트 종료 후 본 기록과 개별 리뷰를 작성했다.

## 최종 검증 대상과 판정

**통합 판정은 메인터너 보정 완료, 수용 가능(검토 범위 한정)이다.** #6853의 물리 18·19쪽 그래프 분할과 추가 보정 중 발생한 #1939 HWPX 왕복 렌더 회귀를 해결했다. 최종 미커밋 작업 트리에서 8스레드 전체 회귀 9,241건이 통과했다. 아래에 명시한 글꼴·세부 간격 및 개별 PR의 범위 밖 차이를 전체 시각 일치로 해석하지 않는다. 최종 커밋의 원격 CI와 머지 승인은 별도다.

최종 대상은 위 체리픽 완료 SHA와 미커밋 제품 코드·테스트 보정의 합이다. 커밋된 SHA만으로 최신 검증 대상을 지칭하지 않는다. 원 PR CI는 종전 확인 시점의 기록이며 최종 통합 커밋의 원격 CI는 아직 확인하지 않았다.

## 보정 이력과 실제 실행 결과

| 단계 | 결과 및 해석 |
|---|---|
| 초기 통합 | #6776 고정 페이지 가정 실패. 그림의 단일 존재·경계 검사로 바꿨으나 시각 문제는 남았음 |
| 중첩 빈 문단 복원만 적용 | 전체 9,241 통과, 46 건너뜀, 316.805초. 그래프가 18쪽에 남아 해결로 판정하지 않았음 |
| 빈 문단·표 뒤 간격 조판 보정 | 18쪽 그래프 없음, 19쪽 그래프, 20쪽 큰 그림 확인. 전체 9,240 통과·#1939 1건 실패·46 건너뜀, 390.437초 |
| HWP5 원본/계보 HWPX 동등성 추가 보정 | **9,241 통과·실패 0·46 건너뜀**, exit 0. 8스레드, no-fail-fast, 실행 388.355초, 느린 테스트 3건 |
| 최종 테스트 빌드 | release-test 프로필, 6분 33초. 실행 ID daba02f9-1099-415a-98a1-d7dbc5d85541, 78개 바이너리 |
| 최종 범위 | #6776의 19쪽 그래프 경계·단일 그림 배치·비-TAC 음성 대조, #6854 고아 쪽, #1939 왕복 렌더를 전체 실행에 포함 |

#1939 중간 실패는 HWP5 원본에만 보정이 적용되어 자기-export HWPX와 475.87px 차이가 생긴 것이다. hwp5_stored_pagination_layout 공통 조건으로 정정했고 1px 허용치나 구조 동일성 검사를 완화하지 않았다. 제품 코드에 페이지 번호·그림 ID·임의 여백을 넣지 않았다. 보정 상세는 [#6853 리뷰](pr_6853_review.md)를 따른다.

## 이전 검증과 최종 재실행의 구분

Native Skia 선택 76건, Studio 1,493 통과·2 제외, Node 114건, Python 81건, fmt·TypeScript·native/WASM Clippy, 개발용 WASM 빌드, CLI 저장·재개방 3건은 이전 제품 코드의 검증 결과다. 최종 조판·왕복 변경 뒤에는 전체 Rust 회귀를 재실행했으며 이 별도 검사들은 재실행하지 않았다. 최적화 release WASM, 전체 브라우저 E2E, 새 10,000건 비공개 코퍼스 A/B, Windows/Linux 로컬 실행은 완료했다고 주장하지 않는다. 서로 다른 스레드 수·실행 조건의 소요 시간을 성능 개선 근거로 사용하지 않는다.

## 원문 등록과 새 MCP PDF

Mac korea_downloads에서 원문 7개를 찾아 기존 samples 파일과 SHA-256 동일성을 확인했다. 확장자 기준 HWP 3개, HWPX 4개이며 바이트를 그대로 보존했다. 단, 78494 문서는 .hwpx 확장자와 달리 실제 내용이 HWP5임을 추가 확인했다. 이미 등록된 파일이므로 중복 사본이나 가짜 HWP 변환본을 추가하지 않았다. 원래 파일명·저장 버전·원문 해시는 각 MANIFEST.json을 따른다.

새 MCP에서 모두 start → status → download 비동기로 재산출했다. 요청 timeout은 1,800초였다. engine 2020은 호환 프로필 명칭이며 실제 한컴 버전은 12.0.0.4605다. engine 2024의 실제 버전은 13.0.0.3901이다. 서버 주소·인증 정보는 기록하지 않는다. PDF 서명·종료 표식·바이트 수·클라이언트 SHA-256·쪽수를 확인했고 모두 50MB 미만이다.

| 원문 샘플 | 재산출 PDF | 쪽수 | engine / 실제 버전 | PDF SHA-256 |
|---|---|---:|---|---|
| [113424_evaluation_guideline.hwpx](../../../samples/issue6551/113424_evaluation_guideline.hwpx) | [113424_evaluation_guideline-2024.pdf](../../../pdf/113424_evaluation_guideline-2024.pdf) | 46 | 2024 / 13.0.0.3901 | `f2db782bb42cabd2b96cd8985415554803ad7d4322904177a9e1bc903115bf6d` |
| [78494-virtual-convergence-industry-decree.hwpx](../../../samples/issue6776/78494-virtual-convergence-industry-decree.hwpx) | [78494-virtual-convergence-industry-decree-2020.pdf](../../../pdf/78494-virtual-convergence-industry-decree-2020.pdf) | 74 | 2020 / 12.0.0.4605 | `367fb3749f571521eef2036fe4327ce5b9cf6b7728966257328c1144f08ff8c9` |
| [36367506-water-facility-approval.hwpx](../../../samples/issue6776/36367506-water-facility-approval.hwpx) | [36367506-water-facility-approval-2020.pdf](../../../pdf/36367506-water-facility-approval-2020.pdf) | 3 | 2020 / 12.0.0.4605 | `fe82e237046659fdd770811dbd30c8f199d155d9737f1a8d0732f19f2049deda` |
| [70833-electrical-safety-rule-regulatory-analysis.hwp](../../../samples/issue6854/70833-electrical-safety-rule-regulatory-analysis.hwp) | [70833-electrical-safety-rule-regulatory-analysis-2020.pdf](../../../pdf/70833-electrical-safety-rule-regulatory-analysis-2020.pdf) | 18 | 2020 / 12.0.0.4605 | `ae8b6bd5a07fa543dd9f69be750690851846a3e4e09668e3158bb05fa71bc126` |
| [22037757-chuncheon-personnel-rule-annex13.hwpx](../../../samples/issue6854/22037757-chuncheon-personnel-rule-annex13.hwpx) | [22037757-chuncheon-personnel-rule-annex13-2020.pdf](../../../pdf/22037757-chuncheon-personnel-rule-annex13-2020.pdf) | 15 | 2020 / 12.0.0.4605 | `2df4d48c365dbae60ab88b32969720e7d83a211084620d97f93f2606dd90498e` |
| [1613000-202200037-air-traffic-controller-cbta.hwp](../../../samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp) | [1613000-202200037-air-traffic-controller-cbta-2020.pdf](../../../pdf/1613000-202200037-air-traffic-controller-cbta-2020.pdf) | 204 | 2020 / 12.0.0.4605 | `71145684dae73b459f088a8e10acc0335a7585c5345d7207401a3d988696fe6b` |
| [30269-anticorruption-recommendation-toc.hwp](../../../samples/issue6844/30269-anticorruption-recommendation-toc.hwp) | [30269-anticorruption-recommendation-toc-2020.pdf](../../../pdf/30269-anticorruption-recommendation-toc-2020.pdf) | 22 | 2020 / 12.0.0.4605 | `d01f9ff59664a9c5b4e1a67e38f0d1c295aac8e22e3fbc07017f938f197822c3` |

## 시각 대조 범위와 한계

- visual_sweep.py의 현재 통합 CLI, Chrome/webfont SVG 래스터 경로, 96 DPI로 선택 쪽을 대조했다. PC 전체 화면 캡처가 아니다.
- 그라데이션: 5·7·29쪽. 방향을 확인했지만 7쪽의 일부 상자형 대체 글리프는 남아 있다. 전체 글꼴 충실도를 승인하지 않는다.
- TAC: 최종 형식 동등성 보정까지 적용한 CLI를 새로 빌드해 기존 새 MCP PDF의 물리 18·19·20쪽과 재대조했다. 18쪽에는 그래프가 없고, 19쪽 그래프와 20쪽 큰 그림이 각각 본문 상단에 온전히 놓인다. 세 PNG를 최신 산출물로 교체했다. 글꼴·세부 간격까지 동일한 것은 아니다.
- 음성 대조: 비-TAC 문서 1~3쪽. 고아 쪽: HWP 13~15쪽, HWPX 9~11쪽. 글꼴·행 위치 차이는 별도로 남는다.
- float: rhwp 물리 183쪽과 PDF 물리 186쪽이 대응한다. 기존 PDF 185쪽 비교는 잘못된 대응이므로 최종 증적에서 제외했다. PDF는 앞 표의 꼬리를 포함하여 문구의 Y 위치가 다르다. 전체 201/204쪽 차이는 미해결이다.
- 목차: 4쪽 형상 대조. bbox 승인 근거는 구조 테스트이며 PDF만으로 실제 선택·클릭을 검증하지 않는다.
- 구/신 PDF의 최초 선택 18쪽 래스터 중 17쪽은 동일하고 TAC 20쪽은 달랐다. 따라서 TAC 증적은 새 PDF로 재산출한 것을 쓴다. 그 외 동일한 쪽의 패널은 기존 패널을 재사용했으며 처음부터 새 PDF로 생성했다고 주장하지 않는다. float 186쪽은 새 PDF에서 별도 렌더링했다.
- 자동 flag가 비어 있어도 전체 시각 일치로 판정하지 않는다.

## 최종 보관 및 코멘트 계획

PDF 7개와 개별 리뷰에서 직접 인용하는 PNG 12개만 최종 증적으로 보관한다. 임시 SVG·JSON·중간 PNG·.log는 이번 추가 파일에 포함하지 않는다. 최종 PNG는 ../assets/pr_6849_6867_planet6897_20260908/ 아래에 있다.

머지 이후 원 PR/이슈 코멘트에는 실제 merge SHA의 raw 이미지 URL을 Markdown 이미지로 직접 삽입하여 본문에서 보이게 한다. PDF도 동일 SHA의 경로를 연결한다. 실제 PR/devel CI가 모두 통과한 뒤에만 수용 및 close 사실을 기록한다. 기존 코멘트가 있으면 수정하고 중복 게시하지 않는다. 로컬 통합 실패는 해소됐지만 새 원격 CI 확인·PR 생성·머지·코멘트·close는 수행하지 않았다.

- [PR #6849 개별 검토](pr_6849_review.md): 승인 (변경 범위 한정)
- [PR #6851 개별 검토](pr_6851_review.md): 승인 (그라데이션 축 방향 한정)
- [PR #6853 개별 검토](pr_6853_review.md): 메인터너 보정 완료, 수용 가능 (그림 흐름·페이지 분할 범위)
- [PR #6857 개별 검토](pr_6857_review.md): 승인 (빈 고아 쪽 제거 한정)
- [PR #6859 개별 검토](pr_6859_review.md): 승인 (문구 누락 방지 한정)
- [PR #6867 개별 검토](pr_6867_review.md): 승인 (탭 런 bbox 계약 한정)

## 원본 PR PDF와 재산출 PDF의 해시 비교

사용자 지시에 따라 체리픽 완료 HEAD의 pdf/ 원본 바이트와 새 MCP 다운로드 파일의 SHA-256을 직접 비교했다. 7개 모두 해시가 달라 동일 파일로 삭제한 재산출본은 0개다. 새 해시는 위 기준 PDF 표에 기록했다. 파일 해시 차이만으로 시각적 내용이 달라졌다고 주장하지 않는다.

| PDF | 원본 HEAD SHA-256 | 비교 |
|---|---|---|
| 113424_evaluation_guideline-2024.pdf | `e05bd3ec209a09825954004cf204d701e3428366b6700d707a10b70d939b332c` | 새 PDF와 다름, 보존 |
| 78494-virtual-convergence-industry-decree-2020.pdf | `f6e8eca2ff0de55271d3ccb04b08d68244aee9ca6424de42fcce3bc55ffcb5f7` | 새 PDF와 다름, 보존 |
| 36367506-water-facility-approval-2020.pdf | `74fc13ac6bbfc706814270e1f9cf3c092c8e0621e369a0984a725022c429f1c0` | 새 PDF와 다름, 보존 |
| 70833-electrical-safety-rule-regulatory-analysis-2020.pdf | `6186340bd7ea889f9bf9bfc1d7de0df09455e49d29d1644e7ef8a77d420bef2f` | 새 PDF와 다름, 보존 |
| 22037757-chuncheon-personnel-rule-annex13-2020.pdf | `847a0bcdd721ba33fff94a6f2b8dd6faea54e818f9fdf28405c0d49eca4e67f0` | 새 PDF와 다름, 보존 |
| 1613000-202200037-air-traffic-controller-cbta-2020.pdf | `6ad5521bde00e9cda7631191ce8d2e57f66a890816aab8ed8496eb65113003b3` | 새 PDF와 다름, 보존 |
| 30269-anticorruption-recommendation-toc-2020.pdf | `a3ba2c831939a3ebb60e4070d66929908d7d5a0f0a9571e3cba6876746ef8609` | 새 PDF와 다름, 보존 |

## TAC 증적의 생성 시점과 재현 범위

현재 tac-p018.png, tac-p019.png, tac-p020.png는 #1939 형식 동등성 보정까지 적용한 최종 미커밋 작업 트리에서 CLI를 새로 빌드하여 재산출한 자료다. 이전 중간 보정 단계 PNG를 그대로 재사용하지 않았다. 바이너리 /tmp/rhwp-6776-final-origin-parity-bin의 SHA-256은 6f5e9f7ce8de9ea5d78840a56dd8201a48571e55c11386e5abfbebd0b8708296이며 release-test 빌드는 1분 37초, exit 0이다.

- 원문: samples/issue6776/78494-virtual-convergence-industry-decree.hwpx. SHA-256: 5090e9d20ce75814febeb4b79e106ea6abfd2546e511f229900d137d42623617.
- 기준 PDF: pdf/78494-virtual-convergence-industry-decree-2020.pdf. SHA-256: 367fb3749f571521eef2036fe4327ce5b9cf6b7728966257328c1144f08ff8c9. 이번 증적 갱신에서는 PDF를 다시 변환하지 않았다.
- 재산출 결과: 18쪽 그림 없음, 19쪽 그래프 y=78.9667px/h=312.4px, 20쪽 큰 그림 y=78.9667px/h=725.36px. 세 대조 PNG를 직접 열어 확인했다.
- 보관: 코멘트에서 인용할 세 PNG만 기존 assets 경로에 반영했다. 임시 SVG·render tree JSON·로그·중간 래스터는 /tmp에 남기며 커밋 대상에 추가하지 않는다.

재현 명령:

```sh
python3 scripts/visual_sweep.py \
  --rhwp-bin /tmp/rhwp-6776-final-origin-parity-bin \
  --dpi 96 --key tac-6776-final-origin-parity \
  --hwp samples/issue6776/78494-virtual-convergence-industry-decree.hwpx \
  --pdf pdf/78494-virtual-convergence-industry-decree-2020.pdf \
  --pages 18,19,20 --out /tmp/rhwp-6776-final-origin-parity-visual
```

개별 판정의 수용 범위와 남아 있는 글꼴·세부 간격 차이는 후속 코멘트에도 그대로 적는다. 이번에는 증적만 재산출했으며 9,241건 전체 회귀는 중복 실행하지 않았다.

## 최신 검증: 조판·왕복 보정 완료, 8스레드 전체 회귀 통과

- 검증 대상: 체리픽 완료 head에 중첩 빈 셀 문단 복원, NO_LS 빈 문단 높이 보존, TAC 표 뒤 간격 반영, HWP5 원본/계보 HWPX 공통 조판 조건 및 #6776 경계 테스트 보정을 더한 미커밋 작업 트리.
- 최종 전체 회귀: 9,241건 실행, **9,241 통과·실패 0·46 건너뜀**, 느린 테스트 3건, exit 0. 실행 388.355초, 테스트 바이너리 빌드 6분 33초.
- 실행 ID: daba02f9-1099-415a-98a1-d7dbc5d85541. 78개 테스트 바이너리, 8스레드, no-fail-fast로 끝까지 실행했다.
- #1939 HWP5 원본 → HWPX 왕복 렌더 테스트와 #6776의 물리 19쪽 그래프 배치·용지 경계 테스트를 포함해 모두 통과했다. 허용치 완화나 실패 테스트 제외로 통과시킨 것이 아니다.
- 시각 대조: #1939 형식 동등성 보정까지 포함한 최종 작업 트리의 CLI를 새로 빌드하고 기준 PDF와 물리 18·19·20쪽을 재산출했다. 18쪽에는 그래프가 없고, 19쪽에는 312.4px 그래프가 y=78.9667px에, 20쪽에는 725.36px 그림이 y=78.9667px에 배치됨을 SVG 좌표와 세 대조 PNG에서 확인했다. CLI 빌드는 exit 0, 1분 37초이며 바이너리 SHA-256은 6f5e9f7ce8de9ea5d78840a56dd8201a48571e55c11386e5abfbebd0b8708296이다. 원문과 기준 PDF는 변경하지 않았다.
- 남은 한계: 글꼴 대체와 일부 세부 간격 차이는 남는다. 페이지 경계 해결을 픽셀 단위 동일성으로 주장하지 않는다. Native Skia·Studio·Node/Python 계약·fmt·TypeScript·Clippy·WASM의 이전 결과를 이번 최종 변경의 재실행 결과로 표시하지 않는다.

```sh
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast
```

로그는 임시 경로에만 보관하고 커밋에 추가하지 않는다. 이번 증적 갱신에서는 최종 CLI 빌드와 18·19·20쪽 시각 대조를 수행했고 전체 회귀는 반복하지 않았다. 이 절은 증적 갱신 당시의 기록이며, 이후 상태는 아래 PR 생성 직전 최종 기록을 따른다.

## PR 생성 직전 최종 기록 (2026-09-08)

- 메인터너 보정 커밋: `64f33cfc1c96d893c7e081b987349994dbff2717`. 앞 절의 미커밋 작업 트리 검증은 이 커밋으로 보존한 보정 내용에 해당한다.
- 사용자 승인 후 최종 필수 lint를 순차 재실행했다. suite prepare/check, fmt/check, native Clippy, WASM32 Clippy, workspace build, workspace all-target Clippy가 모두 종료 코드 0으로 통과했다.
- native Clippy 25.79초, WASM32 Clippy 13.87초, workspace build 43.49초, workspace all-target Clippy 30.62초다. 대상 디렉터리는 `target/pr-review`다.
- 전체 회귀는 이미 완료한 8스레드 결과인 **9,241 통과, 0 실패, 46 건너뜀**, 388.355초를 사용하며 이번 lint 실행 중 반복하지 않았다.
- 오늘할일과 개별 리뷰, 최종 기준 PDF 7개, 코멘트용 PNG 12개를 같은 통합 PR에 포함한다. 임시 로그·중간 PNG/SVG/JSON 및 generated suite는 추가하지 않는다.
- 이 기록 작성 시점에는 최종 head의 원격 CI와 병합이 아직 수행되지 않았다. 로컬 검증 통과를 원격 CI 통과로 대체하지 않는다. 원 source PR/이슈의 코멘트·종료는 병합 후 절차로 남긴다.
