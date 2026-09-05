# 18 — pageCount 1..220 라우팅 표

`info.pageCount` 값마다 첫 경로와 금지를 고정한다. 추측하지 않는다.

| pageCount | 밴드 | 기본 경로 | 금지 | truncated 고지 |
| --- | --- | --- | --- | --- |
| 1 | tiny | info→export-text --json | 사실만 물으면 search부터 | no |
| 2 | tiny | info→export-text --json | 사실만 물으면 search부터 | no |
| 3 | tiny | info→export-text --json | 사실만 물으면 search부터 | no |
| 4 | small | info→explain | export-text 무제한 | no |
| 5 | small | info→explain | export-text 무제한 | no |
| 6 | small | info→explain | export-text 무제한 | no |
| 7 | small | info→explain | export-text 무제한 | no |
| 8 | small | info→explain | export-text 무제한 | no |
| 9 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 10 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 11 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 12 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 13 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 14 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 15 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 16 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 17 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 18 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 19 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 20 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 21 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 22 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 23 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 24 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 25 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 26 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 27 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 28 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 29 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 30 | medium | info→explain→digest --max-chars | export-text 무제한; 전쪽 png | yes |
| 31 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 32 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 33 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 34 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 35 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 36 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 37 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 38 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 39 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 40 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 41 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 42 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 43 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 44 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 45 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 46 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 47 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 48 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 49 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 50 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 51 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 52 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 53 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 54 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 55 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 56 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 57 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 58 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 59 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 60 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 61 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 62 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 63 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 64 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 65 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 66 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 67 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 68 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 69 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 70 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 71 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 72 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 73 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 74 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 75 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 76 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 77 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 78 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 79 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 80 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 81 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 82 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 83 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 84 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 85 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 86 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 87 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 88 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 89 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 90 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 91 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 92 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 93 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 94 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 95 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 96 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 97 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 98 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 99 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 100 | large | info→digest --max-chars 800 | excerpt를 전체로 읽기 | yes |
| 101 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 102 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 103 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 104 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 105 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 106 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 107 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 108 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 109 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 110 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 111 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 112 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 113 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 114 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 115 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 116 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 117 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 118 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 119 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 120 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 121 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 122 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 123 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 124 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 125 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 126 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 127 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 128 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 129 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 130 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 131 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 132 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 133 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 134 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 135 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 136 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 137 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 138 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 139 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 140 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 141 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 142 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 143 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 144 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 145 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 146 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 147 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 148 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 149 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 150 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 151 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 152 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 153 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 154 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 155 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 156 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 157 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 158 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 159 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 160 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 161 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 162 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 163 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 164 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 165 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 166 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 167 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 168 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 169 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 170 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 171 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 172 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 173 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 174 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 175 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 176 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 177 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 178 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 179 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 180 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 181 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 182 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 183 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 184 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 185 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 186 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 187 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 188 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 189 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 190 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 191 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 192 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 193 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 194 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 195 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 196 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 197 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 198 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 199 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 200 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 201 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 202 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 203 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 204 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 205 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 206 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 207 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 208 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 209 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 210 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 211 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 212 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 213 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 214 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 215 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 216 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 217 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 218 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 219 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |
| 220 | huge | info→digest --max-chars 600→search --limit 20 | digest --pages 0..last | yes |

220쪽을 넘는 문서도 `huge` 와 같다. pageCount를 모르면 이 표를 쓰지 말고 info부터 한다.
