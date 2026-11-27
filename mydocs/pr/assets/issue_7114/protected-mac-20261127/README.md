# #9306 Mac 보호 검증

최신 devel과 PR head를 합친 트리의 Native Skia와 fresh WASM Canvas를 비교했다. 23개 문서·46개 조건의 4,046쪽이 승인된 보호 하한을 모두 충족했다.

sample16은 같은 Mac 공급에서 devel·후보 384쪽의 PNG 해시가 모두 같았다. 사용자가 승인한 수식 6쪽 여섯 행에 실측 하한을 적용했다. 편람 HWP와 HWPX의 WASM은 함초롬바탕을 다시 공급해 200쪽이 99.53%에서 100.00%로 복원됐다. sample10은 Native·WASM 각 763쪽이 하한을 충족했다.

[페이지별 결과](protected-page-comparison.tsv) · [입력·글꼴·바이너리 해시](validation.json) · [sample16 승인 근거](../sample16-mac-protection-20261127/README.md)
