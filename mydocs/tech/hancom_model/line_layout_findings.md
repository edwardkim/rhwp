---
kind: reference
status: active
canonical: mydocs/tech/hancom_model/README.md
last_verified: 2026-07-19
---

# HNC Line Layout Findings

This document is the canonical HNC line-layout **model**: it states the
per-glyph advance function, the break-candidate selector, the endpoint
finalizer, and the surrounding paragraph-shape geometry as model roles, with
the empirical findings (authoritative PDF output and lineseg roundtrip) that
constrain each part.

It separates model behavior that is empirically established from implementation
details that remain unknown. Cross-references:
[`unified_layout_semantics.md`](unified_layout_semantics.md),
[`endpoint_line_closure_semantics.md`](endpoint_line_closure_semantics.md),
[`object_placement_semantics.md`](object_placement_semantics.md),
[`flow_pagination_semantics.md`](flow_pagination_semantics.md), and the index
[`README.md`](README.md).

## Verified Descriptor Semantics

The reference fixture is `TJ5:WRAP `, followed by U+AC00 units and terminal
U+000D. The table-cell available width is `38072`.

| Source length | U+AC00 count | Emitted source intervals |
| ---: | ---: | --- |
| 36 | 26 | `[0,36]` |
| 37 | 27 | `[0,37]` |
| 38 | 28 | `[0,38]` |
| 39 | 29 | `[0,39]` |
| 40 | 30 | `[0,9] [9,40]` |
| 41 | 31 | `[0,9] [9,41]` |
| 42 | 32 | `[0,9] [9,42]` |
| 43 | 33 | `[0,9] [9,43]` |
| 44 | 34 | `[0,9] [9,44]` |
| 45 | 35 | `[0,9] [9,43] [43,45]` |
| 46 | 36 | `[0,9] [9,43] [43,46]` |
| 47 | 37 | `[0,9] [9,43] [43,47]` |
| 48 | 38 | `[0,9] [9,43] [43,48]` |

Deletion reproduces every insertion state in reverse. In particular:

```text
40 -> 39: [0,9] [9,40] -> [0,39]
45 -> 44: [0,9] [9,43] [43,45] -> [0,9] [9,44]
```

The final deletion restores source length 36 as `[0,36]`.

## Known Constraints

For this font and style, one U+AC00 unit has nominal advance `1100`:

```text
34 * 1100 = 37400 <= 38072
35 * 1100 = 38500 > 38072
```

This explains the second wrap threshold in this fixture. It does not define
the general horizontal advance function.

At source length 40, the model changes from `[0,39]` to `[0,9] [9,40]`. It does
not retain the longest fitting prefix on the first line. The source-space
boundary at position 9 therefore participates in endpoint selection with a
different status from an arbitrary internal Hangul boundary.

## Formal Boundary

Let `A_m(i,j)` be the effective advance for source interval `[i,j)` in
flow frame `F_m`, and let `B_m(i,j)` be its break admissibility or priority.

```text
C(i,m) = { j | i < j <= n, A_m(i,j) <= W_m, B_m(i,j) < infinity }
```

Endpoint selection decomposes into a specialized candidate selector `Q_m` and
a wrapper finalization `R_m`:

```text
b_{k+1} = R_m(b_k, overflow_k, Q_m, state_k)
```

For hyphenation mode, with candidate offsets `h_r` emitted in HNC order:

```text
c_r = segment_origin + h_r + 1
Q_m = first c_r such that P_m(c_r) + M(c_r) <= W_m
```

Both an accepted sequence (`15`, then `13`) and a no-candidate result (`-1`)
are observed. `R_m` preserves `Q_m` in the accepted table case and overrides
the specialized selector in the keep-word case. Its complete protected-object,
control, and page-transition predicates remain to be fully enumerated.

The descriptor emitter runs after `R_m`; it is the output oracle and is not
used to infer candidate ordering.

## Edit Invariant

For document state `D`, style state `S`, frame sequence `F`, and the canonical
HNC layout function `Lambda`:

```text
E^-1(E(D)) = D
    =>
Lambda(E^-1(E(D)), S, F) = Lambda(D, S, F)
```

This invariant holds for the table-cell sequence above. The model contains
no stored/imported/generated geometry mode: layout depends only on document
and style state, not on how the geometry was produced.

## Model Overview

The model has two required internal paths:

1. The function that computes effective horizontal advance for a candidate
   source interval.
2. The function that classifies break candidates and selects the final source
   endpoint.

The evidence base contains live inputs and outputs, reversible edits, a
no-candidate hyphenation run, and accepted candidate enumeration in a table
flow frame, all cross-checked against the authoritative PDF and the lineseg
roundtrip. The complete wrapper predicates for protected objects, controls,
and page transitions are separate extension work. rhwp must not replace them
with overflow guards, provenance flags, or longest-fit rules.

## Model Roles

### Paragraph flow-directive decoder

The paragraph flow-directive decoder (`ResolveParagraphFlowDirective`) is not a
page-break-only query. It reads the paragraph-header break byte and normalizes
it into an internal break vector.

The paragraph-header break bits are those already represented by rhwp:

```text
0x01  section break
0x02  multicolumn break
0x04  page break
0x08  column break
```

The precedence and internal bit-vector construction are:

```text
raw & 0x01 -> 0x100                    section
raw & 0x02 -> 0x800 | (raw&0x04?0x200:0) multicolumn, optionally page
raw & 0x08 -> 0x400                    column
raw & 0x04 -> 0x200                    page
paragraph style page-break-before -> 0x200
```

The resolver then applies context masks. One capability query clears the
`0x100|0x200` page/section axis when unavailable. A second capability query
clears the `0x400|0x800` column/multicolumn axis when unavailable. A plain
`0x400` column directive is additionally cleared when `columnIndex + 1 >=
columnCount`.

The normalized vector is then dispatched:

```text
0x100       -> section family
0x800       -> column-family branch
0xA00       -> page-family branch
0x400       -> column-family branch
0x200       -> page-family branch
0x82        -> terminal/default-owner branch
otherwise   -> automatic branch
```

The GUI independently produces `0x200` for Ctrl+Enter and `0x400` for
Ctrl+Shift+Enter. The decoder supplies the section, multicolumn, normalization,
and dispatch relations; it does not by itself define the geometry produced
inside each transition family.

### Cursor predecessor

The backward-cursor step (`MeasureBackwardCursorStep`) receives a source offset
and an address-state byte, reads the preceding source units through the native
accessor, and returns the number of units to move backward. Its script-range
branches group Korean/Jamo and control ranges, so endpoint resolution cannot be
modeled as integer `i - 1`. Together with the endpoint output pair
`(offset,state)`, this establishes the native cursor domain used by the
canonical model.

The horizontal layout path in the formatter is:

```text
formatLine():
  while (!(formatter.overflow_flag & 1)) {
    accumulateSourceAdvance()
    advanceCursor()
  }
```

The line-record builder consumes the formatter result and creates internal line
records. It copies interval pairs from the formatter's interval buffer, marks
the first and last records with `0x20000` and `0x40000`, and writes the
endpoint position, slot, and available width. It is downstream of line
selection. The line records are separate from the paragraph layout header,
which stores endpoint position/state as a `(offset,state)` pair. The byte is an
address mode, not a width.

The address model is a node/position/mode tuple. Forward continuation follows a
node's successor link only while the next node is flagged as a logical
continuation and only when the mode equals `1`, reducing the flattened position
by each node length; every other value is resolved in the current node. The
constructors build `(node_length,0)` for local addressing and
`(sum_of_chain_lengths,1)` for flattened-chain addressing. The
identity-preserving inverse maps `(root,flattened_position,1)` to
`(resolved_node,local_position,0)`. The backward link is proved by summing
preceding node lengths while returning a local cursor to the flattened domain.
Thus the mode is not a width, and consistent conversion of the full
node/position/mode tuple does not change source identity or geometry. A census
of provenances found exactly one chain-mode (`1`) constructor in the
flow/owner transition path and many local-mode (`0`) constructors in the ODF
serialization routine. This does not exclude a cached or indirect local-mode
layout input, but it separates the direct provenances.

The cached/indirect case is confirmed: an `Active-Active` paragraph produced
223 mode-`0` character accesses over positions `0..210`. Its sole node had
length 271, flags 0, and no eligible continuation successor. The
high-confidence origin loads a cached header cursor, preserves its mode, and
scans to the next endpoint. An alternate producer stored in the formatter's
header-cursor slots remains a candidate but is no longer the only known
layout-side provenance. Two other header-writer sites store `(-1,0)` as an
invalidation sentinel; those zeros are not local geometric cursors.

Two paired list-position initializers close the downstream representation
boundary. One reads the header start, the other reads the header end; each
canonicalizes through the identity-preserving inverse and stores the returned
concrete node plus local offset in its list-position object. An independent
path sums the logical chain, constructs `(total_length-1,1)`, and
canonicalizes it to the final physical node. This is high-confidence proof that
mode `0` can be a normalized representation of the same logical source
location, not a separate geometry policy.

The continuation producer is localized. The producer queries a note/anchor
predicate at each current node's `length-1` cursor with node-local mode `0`;
only on success does it load the successor node and set the successor's
continuation bit. The predicate queries for note anchors (class `0x11`) and
header/footer anchors (class `0x10`). Both are track-change entries. The lookup
resolves the anchor control ID to a track-change table entry; in this layout
caller either match requires the entry's HWPX `hide` field to be nonzero. The
lookup accepts the complete node/position/mode tuple and canonicalizes mode `1`
before resolving the control. This rules out ordinary line wrapping and generic
tracked-paragraph merging as the bit producer; the chain is an out-of-band
continuation over a currently hidden tracked-change anchor.

The downstream geometry edge is closed. After the formatting loop, the producer
calls the line-record builder with `(formatter, root_node, line_index, 0)`.
That builder creates the line records, copies endpoint and available-width
state, and commits the line header. Thus the continuation changes the logical
source interval seen by the line-record builder; it is not an archival or paint
annotation. The producer relation is high-confidence; a single live run that
records the hide match, writer, line-record call, two physical nodes, and
identity-preserving canonicalization together remains the final cross-check.

The note-only relaxation has a closed request path. The flag owner is the
document object. A context word saves the document's track-change flags with
mask `0x11`, conditionally sets a context bit for one traversal call, and
restores the saved word. The callee translates that bit to a request flag; a
promoter raises such a request further. The downstream range scanner observes
those as two state bits: `bypass_note_track_change_hide_gate` and
`flattened-address canonicalization`.

The two source bits have narrower behavioral provenance. Bit `0x01` is set
while processing the internal `em%%`/`rm%%` master-page control family. Bit
`0x10` is synchronized from a track-change-manager slot that searches the
document's track-change shape table for a post-reserved entry whose inherited
reference count is nonzero. The type discriminator maps internal `0x10..0x13`
to HWPX `Insert`, `Delete`, `CharShape`, and `ParaShape`. These support the
roles `master_page_control_mode` and `has_referenced_track_change_entry`, not
generic view-mode or recording-toggle roles. The combined flag is a scoped
source-to-layout traversal request. It can change the eligible note-owner
endpoint and therefore can indirectly change committed geometry by changing
included logical content. It contributes no character advance of its own.

The separate track-change-manager view word has two exact bits. Its writers
issue a `0x400` invalidation unless only bit `0x01` changes. Bit `0x01` is set
immediately after creating the modification password, making it
password/protection state. In the hide branch, bit `0x02` clear hides Delete
and bit `0x02` set hides Insert, proving final-content and original-content
polarity respectively. A view-option action family for track change registers
in contiguous fixed-size records; its exact mutations prove that `0x04` set is
plain Original/Final without changes and memo, `0x08` set is
non-inline/balloon presentation, `0x10` enables format/shape changes, and
`0x20` enables insert/delete changes. This same hide branch reads a
track-change-author `mark` field; the HWPX reader proves it is the serialized
`TrackChangeAuthor.mark` Boolean. Author enumeration gathers only names with
`mark == 1`, while the select-all-authors path writes `1` to every author,
proving that `1` means selected/included reviewer. This is a view-driven
inclusion and invalidation path, not a width source. Its next edge is exact:
the document holds an embedded document list whose active-state word receives
the manager's `0x400` notification. The receiver follows the active-list owner
chain first. For the common marker-list, document-list, and root-list family,
the resolved target is a list-embedded state and the active state is the
resolved list itself. Other implementations supply the target through a virtual
slot.

The concrete consumer preserves its receiver, tests the receiver's active-state
`0x400` bit, and on set constructs a list scan, finds section-definition
controls, clears two ten-entry 16-bit cache arrays, and
repopulates/compares them. Because `0x400 & 0x600` is nonzero, the same
dispatch also traverses `foot`, `head`, and `secd` controls and clears the
associated cache block before the dirty word is reduced to bit `0x02`. This is
a real list-scan and section-dependent layout recomputation path. The view bits
remain inclusion policy rather than advance values, but they can change
committed geometry by changing the content and anchors admitted to layout.

The effective-format resolver inside that scan probes control class `0x13`
(tracked `ParaShape`). Under original-view polarity it resolves the tracked
prior paragraph shape; otherwise it returns the base paragraph shape. The scan
loop consumes the result's key to rebuild section-dependent caches. Independent
callers of the same resolver include line-record construction, layout sizing,
and spacing distribution. This separates two claims: the manager view word is
not geometry, but the effective `ParaShape` selected by that view is a direct
true-geometry input.

The shape-to-geometry join is concrete rather than inferred from HWPX field
names. The effective `ParaShape` object has this verified core map, named by
role:

| ParaShape role | Meaning | Direct layout consumer |
| --- | --- | --- |
| left / right margin | left / right margin | added to / subtracted from the frame's left / right edge before line slots are built |
| indent | indent | a negative indent is combined with the left margin and admitted as a line/tab-position candidate |
| spacing before / after | spacing before / after | both converted and included in the vertical coordinate; spacing-before is also added when establishing the first layout position |
| line-spacing type / value | line-spacing type / value | selects the line-spacing formula and writes the resulting line leading/height state |
| border offsets left/right/top/bottom | paragraph-border offsets | produces four border-coordinate adjustments; paragraph-border connection can suppress the top/bottom terms |
| border-fill id | border-fill id | resolves current and adjacent border fills when deciding whether paragraph borders connect |
| tab-definition id | tab-definition id | resolves the tab definition and passes the effective shape into the tab/line-position candidate builder |

The `Metric` cells are packed, not bare HWP-unit integers. Consumers first
arithmetic-shift the raw value right by one. When bit 0 is clear the result is
an absolute value; when bit 0 is set, the consumer scales the shifted value
through `MulDiv(shifted, axisBase, 100)`. The selected `axisBase` is
consumer-specific: the negative-indent path requests axis `0`, while the
left/right margin and paragraph-spacing paths request axis `1`. An
implementation that copies the raw dword as a coordinate doubles the value and
mistakes relative metrics for absolute ones.

The packed policy fields also have formatter consumers. `attr1` contains
alignment in bits `2..4`, Latin word-break policy in bits `5..6`, non-Latin
keep-word policy in bit `7`, snap-to-grid in bit `8`, condense in bits
`9..15`, widow/orphan in bit `16`, keep-with-next in bit `17`, keep-lines in
bit `18`, page-break-before in bit `19`, vertical alignment in bits `20..21`,
and heading type in bits `23..24`. Alignment is consumed by the alignment/
spacing-distribution path; the word-break bits by the break selector; condense
by the advance accumulator; vertical alignment by the line-spacing path; and
snap-to-grid by the grid helper, which returns the formatter increment used to
quantize the frame and line positions. These are true geometry inputs even
though none is itself a glyph advance.

The Latin word-break field has exactly three defined format values:
`0=KEEP_WORD`, `1=HYPHENATION`, and `2=BREAK_WORD`, encoded as native masks
`0x00`, `0x20`, and `0x40`. Value `3` / mask `0x60` is unassigned. The wrapper
agrees with the third name: `0x40` bypasses the ordinary word selector, retains
the direct overflow cursor, and conditionally sends it through the endpoint
finalizer. During this audit, RHWP's two projections were corrected: HWP5
`attr1` bits `5..6` now populate `break_latin_word`, and HWPX `breakLatinWord`
now populates those `attr1` bits. Previously either conversion could preserve
XML while rendering or reserializing with `KEEP_WORD`, changing true line
geometry.

The `breakSetting` placement map is exact. The HWPX reader copies the four
`breakSetting` bytes to `attr1` masks `0x10000`, `0x20000`, `0x40000`, and
`0x80000` in that order. The layout path consumes the first three as
widow/orphan, keep-with-next, and keep-lines constraints; page-break-before is
decoded independently by the paragraph flow-directive path. `attr2` bits `0..1`
hold the `lineWrap` class, bits `4/5` hold East-Asian English/number
auto-spacing, and bits `6/7` hold `suppressLineNumbers`/`checked`-class
top-level ParaPr state. The published HWP 5.0 table 40 reserves `attr2` bits
`2..3`. A dedicated getter for them exists, but the HWPX mapper does not set
them and no type-correct formatter consumer has been joined. They therefore
remain opaque reserved state, not a geometry input. Numbering/bullet and
heading-level fields remain structurally mapped without a direct line-geometry
claim here.

Paragraph-border connection is joined to true vertical geometry. The HWPX
reader maps the `border` connect flag to `attr1` bit `28` (`0x10000000`). The
only direct tests of that bit are in the border-connection resolver, which
resolves the previous and next effective ParaShapes, requires the current and
adjacent shapes to be connectable, and caches `previousConnected` /
`nextConnected` in its border-layout context. Depending on the surrounding
border mode, it additionally rejects an empty neighbor border or requires
matching border style and offsets.

The border-adjustment producer always admits `offsetLeft` and `offsetRight`
when their border edges participate. It admits `offsetTop` only when
`previousConnected == 0` and `offsetBottom` only when `nextConnected == 0`.
This is not only paint state. In line-record construction, the third
adjustment (top) is added to the formatted vertical position and written to the
emitted line record's vertical-position field. In paragraph vertical layout,
the bottom adjustment is added directly to the running vertical coordinate.
Paragraph-border connection therefore suppresses duplicate top/bottom spacing
at a connected paragraph boundary and can change line origin, paragraph extent,
and page acceptance. It does not change left/right frame width in this path, so
it is not a horizontal line-break-width term.

The negative search boundary is also exact: there are no other direct consumers
of `attr1` bit `28` on the effective shape. This makes the vertical-only
classification high confidence.

`attr1` bit `29` (`0x20000000`) has a different geometry class. The HWPX reader
maps a second `border` flag to that bit; its exact direct consumer decodes and
subtracts left/right paragraph margins from a four-coordinate paragraph-border
adjustment and changes the vertical border extensions. Its only direct caller
is the paragraph-border rectangle builder; the observed callers carry the
result into drawing, transformed paint bounds, and rectangle expansion. No path
from this bit to line-slot bounds or endpoint selection exists. It is therefore
classified as paragraph-border **PaintGeometry**, not true line-break geometry,
until a different type-correct consumer is proved.

`attr1` bit `30` (`0x40000000`) is the published HWP 5.0 field named
`문단 꼬리 모양` (paragraph tail shape), but its placement class is bounded more
tightly than its name. Its getter returns `(attr1 >> 30) & 1` and is shared by
the ParaShape context/wrapper classes. There are no direct consumers on an
effective ParaShape. A whole-model high-bit search over direct `TEST`/`AND`,
`BT`, and load-then-shift forms found the remaining same-offset hits rejected
by object provenance: they test range-operation flags, extract fields from a
global transition table, or operate on vector/storage extents or unrelated
object flags. The bit-30 writes in the archive/record conversion family belong
to serialization, not formatting. Return-sensitive calls through the shared
getter slot are nine-coordinate graphics operations or destructor/parameter-set
interfaces, not ParaShape geometry.

The current HWPX ParaShape mapper does not set bit `30`, and the OWPML
`ParaShapeType` has no corresponding XML field. HWP5 round-trip still preserves
it as part of the raw `attr1` dword. Bit `30` is therefore
**OpaqueCompatibilityState**: it is neither TrueGeometry, PaintGeometry, nor a
flow directive on current evidence. The visible meaning of the legacy name
remains unknown.

Packed `attr1` bit `31` (`0x80000000`) is a separate, unnamed compatibility
bit. It has no getter in the primary ParaShape dispatch and no direct or
resolved-effective-shape consumer. Its positive evidence is confined to the
`secd` conversion family. Two export paths fold a boolean at a
ParaShape-compatible block into packed bit 31 before archive/OWPML output; the
OWPML path does not emit bit 31 into any OWPML member. The reverse path is
asymmetric: the OWPML import maps the available ParaShape fields but never
creates bit 31 or its mirror, and the HWP import slot is a zero-return stub
because native input already owns the compatibility block. A whole-model
packed-load/mirror-write scan found no path that interprets bit 31 as a runtime
scalar. Bit 31 is therefore **OpaqueCompatibilityState**, with narrower
evidence than bit 30: a native archive round-trip bit with no HWPX producer,
public semantic name, formatter consumer, paint consumer, or flow effect.

The dirty-state `0x400` must not be conflated with the flow-directive `0x400`.
The latter is a normalized column-break class read from paragraph/control flow
state and dispatched to the column-transition family. The former lives at the
resolved list's active-state word, is ORed by a track-change view setter,
consumed by the list-scan consumer, and then cleared. Their identical numeric
value is cross-domain reuse, not a semantic alias. Apparent consumers in the
find-and-replace object are rejected: register and type provenance prove that
their `0x400` object is the find-and-replace object, not any of the three list
types.

### Horizontal advance

The normal-text width cache is populated by calling the active drawing
backend's text-extent method (`GetTextExtentExPointW`) with the UTF-16 run, run
length, and output array. The backend pipeline is:

```text
GetTextExtentExPointW
  -> script/language/font-fallback segmentation
    -> per-segment font selection
      -> modern glyph and Hangul advance calculation
      -> legacy HChar advance calculation
    -> cumulative dx, fit count, SIZE, rotation transform
```

The modern arm handles Hangul syllables U+AC00..U+D7A3, Jamo composition and
fallback, indexed glyph widths, font ratio scaling, and per-code-unit output.
The legacy arm performs the corresponding conversion and scaling. The final
stage turns individual values into the cumulative `dx` contract.

Back in the formatter, the cache-fill routine runs a per-source-position pass
that applies spaces, Unicode categories, zero-width space, character ratio,
character spacing, and Korean/Latin/number pair adjustments before storing the
final per-position values.

The advance accumulator reads those values from the cache, then applies the
remaining paragraph semantics before accumulating the value:

```text
glyph advance
+ boundary adjustment
+ character/language spacing
+ grid or fixed-pitch quantization
+ control/object advance when present
```

The result is accumulated into the formatter's cumulative position and copied
into the per-source-position array used by break selection. Therefore rhwp's
canonical advance cannot be represented as `font.glyph_width(codepoint)` alone.

#### .hft width corrections

Font-side width corrections are applied per glyph when the character-shape
style requests weight or vertical-position adjustment. From the raw glyph
advance `raw`:

```text
bold:          advance += (raw + 10) / 20
extra-weight:  advance += (raw + 8)  / 16
super/sub:     advance  = advance * 16 / 25
```

These are integer operations on the per-glyph advance and are part of the
produced-width sum, not applied post-hoc to a line total.

#### Cumulative-pen differencing

The backend returns cumulative pen positions (`dx` array), not per-glyph
advances. The per-glyph advance for position `k` is the difference
`dx[k] - dx[k-1]` (with `dx[-1] = 0`). Language letter-spacing and the
produced-width sum are computed over these differenced advances, and the
produced width of a source interval is the sum of its per-glyph advances.

#### Fixed-point matrix slack distribution

When a fixed-point (justified) matrix applies, the residual slack between the
produced width and the target width is distributed across the interval's
glyph/space slots. The distributed slack is part of the committed advance and
must be reproduced; it is not a paint-only adjustment.

#### Character-grid geometry

The character-grid path is additionally controlled by HWPX
`applyCharSpacingToCharGrid`, a compatibility flag. Both the advance accumulator
and the measuring scan test this exact field when the grid pitch is nonzero.
Clear uses ordinary grid-cell rounding and the dedicated U+001F control/grid
arm. Set instead applies the language-specific character-spacing adjustment
inside the grid branch. This is a direct advance choice and therefore true
horizontal grid geometry.

For an already-resolved grid pitch, the two arms are:

```text
non-grid: directMetric + languageSpacing
grid:     ceil(directMetric / CharGrid) * CharGrid
```

The grid arm therefore snaps each item advance up to the next multiple of the
grid pitch (`ceil(raw / pitch) * pitch`).

#### Vertical object-baseline geometry

A separate compatibility switch, `adjustBaselineOfObjectToBottom`, controls
vertical line geometry. Both the measuring scan and the line-spacing path retain
ordinary-text primary/secondary vertical maxima separately from the maximum
contributed by all units. If an inline control/object dominates the line, the
set bit changes the final metric to
`all_max + text_primary_max - text_secondary_max`. The line-spacing path then
uses that adjusted value in line-spacing and baseline derivation. This is
true vertical line geometry and can affect pagination; it is not a horizontal
advance or paint-only flag.

A further switch, `extendVertLimitToPageMargins`, is object-placement geometry.
It applies a scoped gate to a signed object value: for the eligible placement
mode, clear clamps a negative value to zero and set preserves it. The returned
value is added to a transformed bound delta before storing the placed
coordinate, to available vertical extent before placement dispatch, and to a
remaining-span calculation. This is true vertical object geometry with possible
flow consequences, not line glyph advance or paint-only state.

#### U+001E / U+001F control metric

The dedicated U+001E/U+001F control metric is behaviorally resolved for
U+001F. Its style record reads `CharShape.Height`, `CharShape.SizeHangul`, and
`CharShape.RatioHangul`. In integer form:

```text
h = Height
if SizeHangul != 100: h = MulDiv(h, SizeHangul, 100)
q = trunc_toward_zero(h / 16)                    # U+001F arm
if RatioHangul != 100: q = MulDiv(q, RatioHangul, 100)
direct_control_advance = q * 4
```

The editor action `InsertFixedWidthSpace` inserts one U+001F source unit: the
HWP position changed from `(0, 0, 16)` to `(0, 0, 17)`, and `DeleteBack`
restored it exactly to `(0, 0, 16)`, with the status bar showing one character
while the unit was present. This proves insertion and reversibility of the
fixed-width space, not a new advance value; the metric proof is the fixture
measurement below.

Character spacing is applied afterward. Paired single-slot measurements prove
that changing `RatioLatin` or `SizeLatin` alone does not change this metric,
even when the fixed-width space inherits its CharShape from an adjacent Latin
seed.

The downstream spacing selector is distinct. Changing `SpacingHangul` alone did
not affect U+001F, while setting `SpacingLatin=0` removed the `-52` adjustment
and changed committed geometry from `272` to `324`. The current codepoint is
classified through `CharLanguageLookup(codepoint,1)` and the result selects a
per-language spacing slot; `CharLanguageLookup(U+001F,1)=1` selects the
`SpacingLatin` slot. The neighboring seed is not an input to this non-grid
selector.

The separate character-grid arm is joined to native document properties.
Setting `HSecDef.CharGrid=1000` and `HParaShape.SnapToGrid=1` produced grid
pitch `1000`; the grid quantization bit was clear. U+001F's direct control
metric remained `324`, but the grid branch rounded it up to one 1000-unit cell.
An enclosing separator span, which also contained the following existing
separator, advanced by exactly two cells (`22000 -> 24000`). This confirms the
ordinary character-grid branch.

#### 원고지 (manuscript-paper) grid geometry

The grid-quantization bit is the SectionDef manuscript-paper writing rule,
exposed by the COM API as `HSecDef.WongojiFormat`. With `WongojiFormat=1`, the
same grid configuration set the manuscript-grid formatter flag. The manuscript
aligner was then called twice:

```text
codepoint   current before   item before   current after   item after
U+001F              16000           324           16000         1000
U+0020              17000           648           17000         1000
```

The aligner receives `gridOrPitch=1000` and aligns cumulative position and item
advance separately for the observed character class. Therefore the two grid
arms are behaviorally distinct even when an already-aligned U+001F occupies one
cell in both. The manuscript trace also contains an unaligned separator
`13324 -> 15000`, demonstrating cumulative-position repair (em-center offset
into the manuscript cell).

A controlled class matrix names the **manuscript independent-cell character
predicate**. It returns zero for U+0027, U+002C, U+002E, ASCII digits, ASCII
lowercase, `U+00C0..U+024F`, `U+1E00..U+1EFF`, `U+FB00..U+FB06`, and
`U+FF60..U+FFEF`; it returns one for every other 16-bit code unit. Result one
rounds current cumulative position and item advance independently. Result zero
instead fits the item relative to the current cell, as shown by lowercase `a`
after period changing `648 -> 676` at current `17324`, so the combined endpoint
becomes exactly `18000`.

The predicate's direct exception is narrower than its surrounding caller:
`previous/mode U+002E + current U+201D`. U+201D still classifies as one but
uses the result-zero path (`current 21324` retained, item `648 -> 676`). U+2019
also classifies as one and has no direct exception, so it independently aligns
current `22324 -> 23000` and item `360 -> 1000`. A later redistribution changes
the local class result to zero for both U+2019 and U+201D after a period.
Period plus U+201D shares one 1000-unit cell (punctuation cell-sharing): the
28-unit residual is split evenly, changing the period `324 -> 338`, the quote
`676 -> 662`, and running current `21324 -> 21338`. Period plus U+2019 instead
occupies two cells: backward alignment changes the period `324 -> 1000`, and
U+2019 remains independently aligned at `1000`.

### Break selection

The advance accumulator compares the accumulated next position against the
current horizontal interval. On overflow it sets the overflow flag and calls
the break selector, which delegates to the candidate scanner.

There is a preceding line-end overflow exception. The measurement reads
compatibility bits that define:

```text
TrailingAlignmentExempt(u) =
  (doNotAlignLastPeriod && u in {'.', ',', '?', '!'}) ||
  (doNotAlignLastForbidden && LineStartForbidden(u))
```

When the preceding extent fits but the current unit's full advance would cross
the interval, a true result skips setting the overflow flag and consumes the
unit. The measuring scan likewise skips the ordinary stop helper. The matched
unit can therefore remain at the current line end and protrude past the nominal
interval (hanging / 금칙 protrusion) rather than forcing an earlier endpoint. The
alignment pass also keeps such units in the trailing suffix excluded from
ordinary line-end alignment. This is true horizontal endpoint/alignment geometry
and is separate from the finalizer's forbidden-boundary rollback.

The break selector handles protected/object boundaries and delegates candidate
selection to the candidate scanner. The candidate scanner performs all of the
following:

1. scans source positions around the overflow point;
2. recognizes spaces, U+3000, U+2000..U+200B, CR, LF, TAB, and U+001F;
3. consults a word-resolution predicate for HNC-specific unbreakable
   characters and object markers;
4. obtains candidate offsets from the candidate-offset generator for the
   scanned segment;
5. measures a candidate through the run object's candidate-measure method,
   then scales it;
6. accepts a candidate only when

```text
per_source_advance[candidate - source_start]
  + candidate_measure
  <= available_width
```

7. writes the selected source position and state byte to its output pair.

The word-resolution predicate (`RequiresWordResolutionAtCursor`) returns
nonzero to enter the word-like candidate branch and zero to remain on the
ordinary/control backward scan. Letters and nonbreaking characters can both
return nonzero, while Korean units return zero in some language/run contexts.
It is therefore neither a unary Unicode predicate nor the whole `break_allowed`
relation.

The candidate-offset generator receives the UTF-16 segment, its length/offset,
and returns a candidate offset or `-1`. It maps source characters to internal
classes and delegates part of the scan to a sub-scanner. Naming the internal
class labels and the concrete candidate-measure implementation remains
extension work.

## Dynamic Boundary Findings

The reference baseline paragraph is:

```text
text      = "TJ5:WRAP \r"
length    = 10
width     = 38072
interval  = [0,10]
```

The descriptor oracle is the reliable source of truth. Advance/break events
are only accepted when the target descriptor advances to the expected next
length after each command; accepted input without a changed target descriptor
is insufficient evidence.

### Live advance values

The table-cell paragraph `TB3:TAB ...` was edited from source length 45 to 44
and restored to 45. The selected intervals were exactly reversible:

```text
45: [0,8) [8,42) [42,45)
44: [0,8) [8,42) [42,44)
45: [0,8) [8,42) [42,45)
```

The backend text-extent method produced these cumulative `dx` arrays for the
target:

```text
"TB3:TAB "
  [584, 1236, 1808, 2072, 2638, 3380, 4024, 4412]

34-code-unit Korean run
  [1100, 2200, ..., 36300, 37400]

"쪽끝"
  [1100, 2200]
```

The formatter completed the middle interval with `currentAdvance = 37400`
against available width `38072`. The direct backend call for the deleted
paragraph's 36-code-unit suffix returned `38888`, which does not fit. This
confirms that the formatter consumes the backend's per-source cumulative
advance and selects endpoints before the over-width suffix is committed.

During the same edit, the word-resolution predicate was called while scanning
the target paragraph. Every observed Korean code unit and U+0020 returned `0`
in this context, so those positions were not protected from the ordinary
backward scan.

Inserting U+0020 at source index 28 made the paragraph length 46. The break
selector received overflow probe position 38 and selected
`(position = 29, state = 1)`. The resulting intervals were `[0,29)` and
`[29,46)`: the boundary is immediately after the space, and the first interval
owns the space at index 28. Its formatter advance was 37020, which fits width
38072. Deleting that space restored the original text; the selector selected
positions 8 and 42, and the intervals returned exactly to
`[0,8) [8,42) [42,45)`.

Therefore a reversible space edit is a deterministic transition over source
positions:

```text
insert U+0020 at 28: [0,8) [8,42) [42,45) -> [0,29) [29,46)
delete U+0020 at 28: [0,29) [29,46) -> [0,8) [8,42) [42,45)
```

The candidate-offset generator and the candidate scanner were not called for
this Hangul path. The universal operation is the break selector; it performs
ordinary backward scanning itself and conditionally delegates to the candidate
scanner. The specialized selector and candidate generator must not be modeled
as the general line-break entry.

The delegation was subsequently observed with width 38072, overflow position
77, and state 1 returning `(position = 72, state = 1)`. Other source states
returned positions 73, 74, 75, and 17. That probe did not capture the owning
paragraph, so those positions are code-path evidence only.

The condition for the candidate generator is exact:

```text
run_class = run_flags & 0x60
run_class == 0x20  => use candidate generator and measure each candidate
run_class != 0x20  => return the selector's direct positional result
```

The native Latin paragraph was changed through the paragraph dialog to
hyphenation mode. The active run object changed from flags `0x180` to `0x1a0`,
establishing the dynamic style mapping:

```text
0x180 & 0x60 = 0x00  keep-word branch
0x1a0 & 0x60 = 0x20  hyphenation-candidate branch
```

With `run_class = 0x20`, the candidate scanner called the candidate generator
at overflow positions 101, 122, 211, and 302. The generator received the UTF-16
segment covering the concatenated `Supercalifragilisticexpialidocious` source
and delegated to its sub-scanner. Both returned `-1` for each observed segment,
meaning this input contains no candidate accepted by the generator.

The resulting fallback sequence was:

```text
overflow  selector  wrapper
101       32        32
122       32        122
211       122       211
302       211       302
```

The emitted intervals were unchanged:

```text
[32,122) [122,211) [211,302) [302,309)
```

This proves the conditional call and its no-candidate result. It does not
establish the accepted-candidate return contract or candidate ordering.

### Accepted candidate in a table flow frame

A table-cell paragraph containing `internationalization` inherited run flags
`0x1a0`. Its default available width was 40932, so no break was required. The
native current-column resize command reduced the flow-frame width and exercised
accepted candidates.

At width 9236, overflow position 19 produced:

```text
sub-scanner result       7
generator result        15
selector endpoint       16
wrapper endpoint        16
```

At width 8104, overflow position 17 caused repeated enumeration:

```text
generator results     15, 13
selector endpoint     14
wrapper endpoint      14
```

Let `s` be the source origin of the UTF-16 segment passed to the generator, and
let `h_r` be its `r`th returned candidate offset. The mapping is:

```text
c_r = s + h_r + 1
```

Let `P_m(c_r)` be the formatter's cumulative per-source advance to `c_r` in
flow frame `F_m`, and let `M(c_r)` be the candidate measure obtained through the
run object's candidate-measure method and scaled. The selector accepts the
first enumerated candidate satisfying:

```text
P_m(c_r) + M(c_r) <= W_m
```

Thus the specialized selector is:

```text
Q_m(s) = first c_r in HNC candidate order
         such that P_m(c_r) + M(c_r) <= W_m
```

If no candidate satisfies the inequality, it returns its direct positional
fallback. The break selector then applies protected-boundary and overflow-state
semantics to `Q_m`; its result, not `Q_m` alone, is the final line endpoint.

At the settled cell width 1440, the descriptor oracle recorded source length 21
and intervals `[3,5) [5,7) [7,10) [10,12) [12,15) [15,18) [18,21)`. The first
interval was emitted before the settled probe attached, so its start is not
inferred here.

### Native Latin wrapper result

The `hancom-latin-candidate.hwp` paragraph was edited by deleting and
reinserting its final `D`. Its available width was 42520, and the transition
was exactly reversible:

```text
309: [32,122) [122,211) [211,302) [302,309)
308: [32,122) [122,211) [211,302) [302,308)
309: [32,122) [122,211) [211,302) [302,309)
```

On both layout passes, the break selector received overflow position 302 and
state 1. The candidate scanner classified the active run object as:

```text
run_flags = 0x180
run_class = run_flags & 0x60 = 0
```

The selector returned `(position = 211, state = 1)`, but the wrapper returned
`(position = 302, state = 1)`. The emitted terminal descriptor began at 302.
Consequently, rhwp must represent selector output and finalized endpoint as
separate values. It must not publish the candidate scanner's return directly as
the line boundary:

```text
candidate_endpoint = 211
selected_endpoint  = 302
emitted_interval   = [302, source_length)
```

Changing the native paragraph-shape bits from `0x180` to `0x1a0` produced an
unstable HWP that terminated the application after rendering. That file is not
evidence for the `run_class == 0x20` branch. The accepted evidence is the
style change performed by the application itself, which produced flags `0x1a0`
and dynamically invoked the candidate generator in the stable native document.

### HWP3-converted mixed-script closure

The converted `hwp3-sample16-hwp5-2022.hwp` fixture contains a long paragraph
mixing Korean, Latin product names, punctuation, and spaces. A reversible GUI
edit inserted `x` immediately before `S/W`, waited for the inserted state to
finish formatting, then deleted it and waited for the restored state.

The descriptor sequence was:

```text
original: [0,54) [54,107) [107,159) [159,211)
inserted: [0,54) [54,107) [107,160) [160,212)
restored: [0,54) [54,107) [107,159) [159,211)
```

The first two endpoints were stable because the insertion occurred after
source position 107. Every affected downstream endpoint advanced by exactly one
source unit and returned exactly after deletion. The break selector finalized
the same transition:

```text
state       line starts        selected endpoints
original    0,54,107,159,211   54,107,159,211,261
inserted    0,54,107,160,212   54,107,160,212,262
restored    0,54,107,159,211   54,107,159,211,261
```

The terminal wrapper call selects two source units beyond the last descriptor
shown above because the capture excerpt stops before the remaining paragraph
descriptors; it is retained as wrapper evidence, not presented as a visible
line boundary in isolation.

No candidate-scanner event occurred during either state. This establishes that
the mixed Korean/Latin edit remained on the break selector's ordinary endpoint
path: the presence of Latin text does not itself select the specialized word
candidate branch. The formatter's run for the affected inserted line began with
`xS/W...`; its first advances were `672, 684, 788, 1296`, while the restored
run began with `S/W...` and `684, 788, 1296`. Endpoint movement therefore
followed the changed cumulative source sequence without changing the
surrounding flow-frame width or descriptor ownership.

### U+2000 ordinary-endpoint semantics

The same restored paragraph was used to sweep an explicitly open Unicode-space
case. U+2000 EN QUAD was inserted at source position 211, between the existing
spaces after `IP,` and the following Korean text. The target descriptor oracle
contained U+2000, so this is a real document edit.

The complete interval transition was:

```text
original: [0,54) [54,107) [107,159) [159,211) [211,261) [261,271)
U+2000:   [0,54) [54,107) [107,159) [159,212) [212,262) [262,272)
restored: [0,54) [54,107) [107,159) [159,211) [211,261) [261,271)
```

U+2000 is therefore owned by the preceding interval: insertion changes
`[159,211)` to `[159,212)`, and the successor interval begins at 212. The
formatter exposed the inserted run with U+2000 at its boundary and assigned the
backend/cache advance `1300` in this style. The break selector selected
endpoint 212 directly; the candidate scanner did not run. This confirms U+2000
as an ordinary-scan stopping unit whose source position belongs to the
completed line in this context.

The attempted U+2007, U+200B, and U+3000 cases remain open: their edits were
not reflected in the target descriptor before restoration, so they are rejected
as semantic evidence. A final redraw verified the original text and all six
original intervals after the sweep.

### U+001F cache/geometry separation

A later capture inserted U+001F at source position 210 through native
`InsertFixedWidthSpace` in two fresh processes. The drawing-cache entry was
zero, but the formatter advance at the next content position was `43084`;
exact deletion restored `42812`. Thus U+001F contributed `272` units to true
line geometry in this fixture even though it had no drawing-cache width.

The matching branch handles U+001E/U+001F outside the object-control mask.
U+001F selects a signed divide-by-16 result from `CharShape.Height`,
conditionally applies `CharShape.SizeHangul` and `CharShape.RatioHangul`, and
multiplies the result by four (the integer form above).

The baseline spacing branch is resolved for this fixture. The direct U+001F
metric `324` is quartered to `81`; the imported helper call is
`MulDiv(81, -16, 100) = -13`; the caller shifts that result left by two
to obtain `-52`. Therefore:

```text
direct control metric       324
character spacing (-16%)    -52
committed geometry delta    272
```

This exactly accounts for the independent `43084 - 42812 = 272` committed
advance measurement.

A fresh U+2007 comparison sharpened the distinction: its cache entry was
`1300`, but inserted and restored committed advance were both `42812` between
the two existing spaces. A follow-up capture resolved that separator run
directly:

```text
U+0020   raw 648 + spacing -104 + boundary 0 = 544
U+2007   no space-cache helper; hasAdvance=0; boundary 0 = 0
U+0020   raw 648 + spacing -104 + boundary 0 = 544
run      41724 -> 42812                          = 1088
```

Thus the result is not adjacent-space collapse: both ordinary spaces remain
independently additive. U+2007's cache entry `1300` is not consumed by this
separator line-advance path. The separator loop admits U+2000..U+200B but calls
the cached-advance accessor only for U+0020 and U+3000. This is a scoped fixture
observation, not a context-free Unicode-width rule.

```text
                         cache input   committed geometry delta
native U+001F                 0                 +272
native U+001F, CharGrid=1000  0                +1000
native U+001F, Wongoji grid   0                +1000
COM U+2007                 1300                    0
```

## rhwp Representation Requirement

The engine needs one canonical layout state for every paragraph context,
including body text, table cells, footnotes, endnotes, and text-bearing
objects:

```text
cumulative_advance[cursor | line_start, slot, style]
break_relation[cursor | layout_context]
flow_interval[line]
selected_endpoint[line]
```

The first two terms are deliberately contextual. The native source address is
the pair `(offset,state)`, not an integer position. The measured cumulative
advance array is indexed from the current source origin and is valid only for
the active slot and style state. Likewise, the wrapper does not read one
context-free break-class array. It combines the cursor unit, address state, run
mode, adjacent units, and control/object queries while scanning.

The four canonical observables have this status:

```text
cumulative_advance:
  formatter-committed per-source values after cache width, style scaling,
  boundary/spacing policy, fixed-space handling, and control/object advance

break_relation:
  contextual resolver predicates plus cursor predecessor/successor operations;
  not a scalar character class

flow_interval:
  descriptor [slot_left, slot_left + slot_width), owner-relative

selected_endpoint:
  ResolveLineEndpoint output (offset,state), committed as descriptor source_end
```

The emitted line descriptor records the horizontal slot (left, width) and the
source interval endpoints (start, end). The line descriptor is distinct from
the paragraph layout header, which holds the wrapper state byte. For example,
an observed 47-unit body paragraph emitted:

```text
line 0: source [0,40),  slot [0,48188)
line 1: source [40,47), slot [0,48188)
```

This descriptor proves the committed line interval and flow slot. It does not
by itself expose the candidate preference order that produced source endpoint
40; that order remains a responsibility of the contextual endpoint resolver.
It also has no state-byte field; the wrapper state is stored separately in the
paragraph layout header.

The endpoint-priority capture resolves the ordinary scan kernel. At candidate
cursor `c`, the model inspects `p = pred_D(c)`. Header/footer (`0x10`) and
footnote/endnote (`0x11`) controls with a nonzero payload are crossed by moving
to `p`. A word-resolution unit delegates to the mode selector. SPACE, U+3000,
U+2000..U+200B, CR, LF, TAB, and U+001F stop the scan and select `c`, so the
complete preceding source span belongs to the line.

The native TAB action produced eight-unit spans:

```text
[43,51): pred(51)=43, selected=51
[29,37): pred(37)=29, selected=37
```

The selector's source-step mask is `0x00ffdbfe`. Any selected control code
below U+0020 advances by eight source units; the mask includes `0x0B`, the HWP
code used by table, shape, picture, equation, and related drawing controls. The
source cursor therefore crosses those controls atomically. Object width is not
encoded by this source span: the line producer resolves the `0x0B` object and
separately tests bit 0 at the object's inline-flow field. Set enters full
object layout; clear follows the generic zero-filled atomic-control path. The
role is `ParticipatesInInlineFlow`, consistent with HWP's common-object
`treat_as_char` bit 0.

The object-control advance mask is narrower than that source-span result. The
control-metric branch tests `0x00e7d80e`, resolves a control, and invokes its
paragraph-advance method. The mask-selected codes are:

```text
0x01, 0x02, 0x03, 0x0B, 0x0C, 0x0E, 0x0F,
0x10, 0x11, 0x12, 0x15, 0x16, 0x17
```

The same function has a separate non-object branch for U+001E and U+001F after
the mask test. That branch derives the style metric described in the U+001F
section above; it does not invoke the object paragraph-advance method.

For the TAC table fixture, the table control's paragraph-advance method
returned `14120` in three formatter callers, including the reversible
insert/delete edit. The fixture fields give the exact identity:

```text
14120 = table width 13554 + left outer margin 283 + right outer margin 283
```

The implementation gates on the object's inline-flow bit, obtains the primary
placement record, and returns the primary or secondary placement extent
according to a context resolver. Fragment zero is embedded in the object; later
fragments are read from the placement array. This proves that table advance is
selected placement geometry rather than a direct read of a source width field.

Table controls and generic shape-object controls share the same
placement-extent calculation. The concrete `Picture` and `ShapeComponent`
primary methods instead return a constant, so their same-numbered slot is not
the paragraph-control slot. A picture's paragraph advance must be associated
with its owning shape object; direct picture-component width dispatch is not a
canonical operation. Which shape-object instance owns a particular picture
component still requires a dynamic body-picture correlation.

The context selector returns an integer orientation class in `0..7`, not a
boolean. One path extracts `(layout_context.flags >> 1) & 7` after
owner-capability and native-control kind checks. The placement resolver then
returns the primary placement extent for class zero and the secondary placement
extent for every nonzero class. This resolves the switch equation while leaving
the semantic class names open.

A direct GUI run resolved the body-picture case. In
`pic-in-table-with-toggle.hwp`, the two TAC pictures resolved as generated
shape objects whose paragraph-advance method is the shared placement-extent
calculation. The ordinary layout context returned orientation class zero. The
selected extents were `16942` and `12257`, exactly matching the respective
picture widths because both horizontal outer margins were zero. Two reversible
edit cycles adjacent to the body picture (U+314C insert/delete and `x`
insert/delete) reproduced the same shape objects, orientation class zero, and
advances. This proves that paragraph picture advance belongs to the generated
shape-object wrapper, not the concrete picture-component method.

A second controlled GUI pass changed the containing table cell to
`세로쓰기(영문 세움)`. Insertion and deletion of `x` both produced orientation
class `2` for the vertical-writing owner. The table fragment placement remained
`primary extent = 42520` and `secondary extent = 39354`; two distinct
generated-picture wrappers returned selected extents `1154` and `7880`. The
deletion pass
reproduced all four values. This maps class `2` to upright-English vertical
cell writing in this fixture and exercises the nonzero-class selector branch.
The table placement tuple and picture-wrapper extents are separate objects and
must not be conflated. Classes `1` and `3..7` remain unmapped.

In the non-TAC floating fixture, no floating drawing object entered this
virtual advance path. The only resolved controls were `SecDef` and `ColDef`,
whose shared implementation always returns zero.

The paired nonbreaking-space cases produced:

```text
SPACE 28 < NBSP 44 < overflow 65 => selected 29
NBSP 28 < SPACE 44 < overflow 65 => selected 45
```

The explicit soft line break was U+000A at source 25; the successor line began
at 26 before width-overflow resolution.

The same state must drive paint, hit testing, caret movement, editing, and
pagination. No field or branch may select behavior based on whether geometry
was stored, imported, generated, or edited. Reversible edits must reproduce the
same arrays and endpoints, not merely a visually similar line count.
