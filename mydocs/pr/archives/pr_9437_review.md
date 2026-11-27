# PR #9437 리뷰 — 세 OS의 정본 보호 글꼴 공급과 하한

**보호 문서 23개 전쪽 검증과 코드 head CI를 통과했다.** macOS, `ubuntu-ted`, `win10-ted`에서 Native와 fresh WASM 각각 2,023쪽을 독립 한컴 Print PDF와 대조했다. 총 12,138개 측정이 완료됐으며 raw 90% 미만은 없었다. [PR #9437](https://github.com/edwardkim/rhwp/pull/9437)

| 검토 항목 | 확인 결과 |
| --- | --- |
| 코드 head와 대상 | `7e0c3c7c76b36adbd1d3100fa918a6a79ff6fdc1` → `devel` (`3cf4ceb5f1ebca8612906a7a9324ea82a3f724fc`) |
| 코드 CI | [CI](https://github.com/edwardkim/rhwp/actions/runs/45905442777) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/45905442840) · [Canvas Render Diff](https://github.com/edwardkim/rhwp/actions/runs/45905442646) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/45905442608) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/45905442588) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/45905442589) 성공. 필수 `CI Impact Policy` 성공. |
| 기록 head | 기록 커밋의 required check와 병합 가능 상태를 확인한 뒤 병합한다. |

## 글꼴 공급과 하한 판정

정본 보호 공급 목록에 `_MAC.otf` 제외 규칙을 등록했다. 검사기는 입력 파일과 심볼릭 링크가 가리키는 파일의 이름에 같은 규칙을 적용한다. 확장자 대소문자 무시, 링크 이름 변경, 일반 OTF, 허용된 `_MAC.TTF`를 검사했다. 두 편람의 WASM 공급 배열에는 #9306에서 확인한 `HANBatang.ttf`, `HANBatangB.ttf`를 같은 순서로 추가했다. 두 형식의 200쪽은 세 OS에서 모두 100.00%였다. 전체 공급은 331개 프로그램, 1,690개 슬롯이다. [공급 매니페스트](../../verification/protected-font-supply/manifest.json) · [#9306 공급 확인](../assets/issue_7114/protected-mac-20261127/README.md)

| 실측 환경 | Native 최저 | fresh WASM 최저 | 측정 |
| --- | ---: | ---: | ---: |
| macOS | 97.20% | 98.86% | 4,046 |
| Ubuntu (`ubuntu-ted`) | 98.86% | 98.86% | 4,046 |
| Windows (`win10-ted`) | 97.27% | 98.86% | 4,046 |

기존 공식 하한과 세 OS 실측 중 낮은 값을 쪽별로 채택했다. 조정한 하한은 11개 문서의 Native 75행과 WASM 118행이다. 시행령 Native 13쪽은 100.00%에서 97.20%, 39쪽은 100.00%에서 99.97%로 조정했다. 입력·기준 PDF·96 DPI·2px 조건으로 4,046행 전체를 확인했다. Ubuntu 네 호스트의 공급을 확인했고, 같은 환경의 전쪽 측정은 `ubuntu-ted`에서 대표로 실행했다. [세 OS 측정 조건](../assets/visual-protected-baseline/updates/cross-os-20261127/conditions.json) · [기존 하한·실측·새 하한](../assets/visual-protected-baseline/updates/cross-os-20261127/page-floors.tsv) · [현행 정본](../assets/visual-protected-baseline/final.tsv)

## 리베이스와 실행 경로 검증

세 OS 실측은 `e41e1e9c81240481433aa12f6d86531df489d9cd`의 검증 도구와 출력으로 수행했다. 해당 소스는 `e46501863d71706304aafdc3d3f6ca35c0b2b1f1`의 Rust와 같다. #9436을 반영한 `f37b89739d9336b75232e7642e1c50f707f387df`에서 Mac Native와 fresh WASM 전쪽을 다시 출력했다. 4,046쪽의 PNG SHA-256, RGB 픽셀, raw 점수가 이전 Mac 출력과 모두 같았다. 최신 Studio에서는 중첩 칸의 선택과 편집, 묶음 속 글상자, 수식 뒤 병합과 실행 취소·재실행을 실제 입력으로 확인했다. 그 뒤 `devel`에 들어온 작업 기록 커밋과 최종 하한 갱신은 출력 코드와 보호 입력을 바꾸지 않았다. [#9436](https://github.com/edwardkim/rhwp/pull/9436) · [리베이스 전후 전쪽 비교](../assets/visual-protected-baseline/updates/cross-os-20261127/rebase-print-parity.tsv)

Windows의 Chrome 종료 뒤 임시 프로필 잠금은 재시도 후 `EPERM`일 때 경로를 로그에 남기도록 처리했다. Windows 편람 HWPX는 완료 묶음을 유지해 재개했고, 최종 쪽수와 전체 TSV를 검증했다. Mac·Windows의 저장소 Python 환경과 Ubuntu 네 호스트에서 작업용 `PYTHONPATH` 없이 Brotli 압축·해제 왕복을 확인했다. 로컬 정책·도구 검사는 Python 34건과 Node 1건이 통과했다. 편람 HWPX Native·WASM의 정본 게이트는 각각 383쪽을 통과했다.
