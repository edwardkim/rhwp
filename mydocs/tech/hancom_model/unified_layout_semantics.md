---
kind: reference
status: active
canonical: mydocs/tech/hancom_model/README.md
last_verified: 2026-07-19
---

# HNC Unified Layout Semantics

This document composes three formal modules without duplicating their internal
rules:

- `EP`: [endpoint and line closure](endpoint_line_closure_semantics.md)
- `OP`: [object/control placement](object_placement_semantics.md)
- `FL`: [flow-owner and pagination](flow_pagination_semantics.md)

The modules remain authoritative for lane-local equations, evidence, and
countermodels. This document defines their shared universe, product state,
interfaces, cross-lane transition system, theorem dependencies, and projection
boundary.

## 1. Completeness Claim And Non-Claim

The unified model is **mathematically complete relative to current evidence**
in this precise sense:

1. every observable currently named by `EP`, `OP`, or `FL` has a typed domain;
2. every documented proved/static/observed assertion embeds into a provenance
   judgment below;
3. every cross-lane handoff is a typed partial morphism;
4. every currently undecidable behavior is an explicit partial function,
   unknown parameter, compatible countermodel, or stage proof obligation;
5. every accepted future trace must satisfy the stated safety, causality, and
   restoration invariants; and
6. no missing layout relation is completed by default, visual similarity,
   pointer equality, or an implementation convention.

This is **not** a complete semantics of the full HNC layout engine. It does not
claim totality, global determinism, global confluence, semantic names for
opaque fields, or accepted verdicts for the 25 stages. No stage is currently
semantically proved.

## 2. Shared Universe

All named identity sets are disjoint sorts, even when the layout engine uses
equal integers or pointers:

```text
FixtureId, RunId, RevisionId
CallSerial, EventSerial, DescriptorSerial, CommitSerial, RejectionSerial
AdmissionSerial, DocumentOpenGeneration

SourceId, ParagraphId, SectionKey, PageKey, FrameKey, OwnerKey
ObjectKey, AnchorKey, ControlKey, TableKey, RowKey, CellKey, FragmentKey
RunKey, StyleKey, MatrixKey, SlotKey, RoleIdentity
CrossLaneSourceKey, LineContext, OpaqueBits, LayoutDescriptorEmission
SelectionBoundary = CommitBoundary | ComposeBoundary | IntermediateBoundary

TypedKey = SectionKey + PageKey + FrameKey + OwnerKey + ObjectKey
         + AnchorKey + ControlKey + TableKey + RowKey + CellKey
         + FragmentKey + RunKey + StyleKey + MatrixKey + SlotKey

U16          = {0,...,65535}
Byte         = {0,...,255}
Nat          = {0,1,...}
HU           = signed HWPUNIT
SourcePos(T) = { i : Nat | 0 <= i <= |T| }
Cursor(T)    = SourcePos(T) x Byte
Span(T)      = { [a,b) | a,b in SourcePos(T), a <= b }
```

The current semantic instance is scoped to a single fixed layout-engine
revision. Imported layout claims are undefined for any other revision until
re-derived; a claim keyed to one revision does not transfer to another by
default.

The common source and geometry records are:

```text
SourceState(T) = {
  source: SourceId,
  paragraph: ParagraphId,
  text: T : U16*,
  revision: RevisionId,
  legal_boundaries: Set<SourcePos(T)>,
  layout_controls: Set<(ControlKey,Span(T))>
}

Slot = { key: SlotKey, left: HU, width: HU }

Endpoint(T) = {
  cursor: Cursor(T),
  source_start: SourcePos(T),
  slot: Slot,
  line_context: LineContext
}

Descriptor(T) = {
  serial: DescriptorSerial,
  paragraph: ParagraphId,
  source_start: SourcePos(T),
  source_end: SourcePos(T),
  slot: Slot,
  flags: OpaqueBits,
  endpoint_state: Option<Byte>
}

DescriptorCommit(T) = {
  descriptor: Descriptor(T),
  frame: FrameKey,
  owner: OwnerKey,
  page: PageKey,
  transition_serial: Option<CallSerial>,
  commit_serial: CommitSerial,
  emission_serial: DescriptorSerial
}
```

The exact descriptor source/slot fields are imported from `EP` and `FL`.
`OwnerKey`, `FrameKey`, `PageKey`, `ObjectKey`, `AnchorKey`, and OP's opaque
`Relation30` never coerce into one another.

### 2.1 Run-local witnesses and normalized identities

Let `Ptr_run` be the layout engine's pointer sort for one run. Normalization is
partial:

```text
normalize : RunId x Ptr_run x WitnessedRole -> Option<TypedKey>
```

It is undefined when the role comes only from paint, dimensions, type-name
text, temporal proximity, or cross-run pointer equality. A normalized identity
is parameterized by fixture, revision, and its accepted role lineage.

## 3. Product State

The global machine state is a product, not a flattened runtime struct:

```text
Sigma(T) = Src(T) x EPState(T) x OPState(T) x FLState(T)
           x CommitState(T) x ProvState
```

with:

```text
EPState(T) = {
  phase: Idle | Pred | Select | Generate | Measure | Scale | Finalize | Closed,
  probe: Option<Cursor(T)>,
  candidate: Option<Candidate>,
  endpoint: Option<Endpoint(T)>,
  call_stack: Stack<CallSerial>,
  unknown: ThetaEP
}

OPVerdict = V_sourceSpan x V_dispatch x V_endpointEffect

OPState(T) = {
  source_control: Option<SourceControl>,
  runtime_object: Option<RunLocalObject>,
  dispatch: Option<Dispatch>,
  placement: Option<PlacementObservation>,
  verdict: OPVerdict,
  anchor: Option<AnchorKey>,
  call_stack: Stack<CallSerial>,
  unknown: ThetaOP
}

CandidateOwnerFrame = { frame: FrameKey, owner: OwnerKey }
AcceptedFlowWitness = { candidate: CandidateOwnerFrame,
                        admissionSerial: AdmissionSerial,
                        sourceLineage: CrossLaneSourceKey,
                        slot: Slot }
RejectedFlowWitness = { candidate: CandidateOwnerFrame,
                        rejectionSerial: RejectionSerial,
                        sourceLineage: CrossLaneSourceKey,
                        slot: Slot }
CommittedFlowWitness = { accepted: AcceptedFlowWitness, page: PageKey,
                         transitionSerial: Option<CallSerial>,
                         commitSerial: CommitSerial }

SuccessorEvidence = {
  fromFrame: FrameKey,
  toFrame: FrameKey,
  toPage: PageKey,
  toOwner: OwnerKey,
  selectionBoundary: SelectionBoundary,
  rejectionSerial: RejectionSerial,
  transitionSerial: CallSerial,
  commitSerial: CommitSerial
}

FLState(T) = {
  cursor: FlowCursor,
  candidate_owner_frame: Option<CandidateOwnerFrame>,
  accepted_flow: Option<AcceptedFlowWitness>,
  rejected_flow: Option<RejectedFlowWitness>,
  committed_flow: Option<CommittedFlowWitness>,
  successor: Option<SuccessorEvidence>,
  graph: PartialFrameGraph,
  candidate: Option<VerticalCandidate>,
  reservation: ReservationState,
  fragments: PartialFragmentGraph,
  unknown: ThetaFL
}

CommitState(T) = {
  descriptors: OrderedList<DescriptorCommit(T)>,
  owner_commits: OrderedList<OwnerCommit>,
  page_commits: OrderedList<CommittedFlowWitness>,
  rejection_serial: Option<RejectionSerial>
}

ProvState = {
  fixture: FixtureId,
  run: RunId,
  revision: RevisionId,
  evidence: Map<EventSerial,EvidenceItem>,
  accepted_dependencies: Set<RoleIdentity>
}
```

`ThetaEP`, `ThetaOP`, and `ThetaFL` are the unknown-parameter families defined
by their lane modules. Product construction does not instantiate them.

`EvidenceItem` is a support judgment (assertion plus its supporting evidence)
scoped to the fixture, run, and revision it was observed under. Evidence is not
merged across differing scopes (see section 4).

## 4. Evidence And Provenance Lattice

The support lattice is:

```text
                 Both
                /    \
          Static      Dynamic
                \    /
               Observed
                  |
                Unknown
```

Formally:

```text
Unknown <= Observed <= Static <= Both
Unknown <= Observed <= Dynamic <= Both
Static and Dynamic are incomparable
```

Format facts are an orthogonal bit, not a stronger proof:

```text
Evidence = Support x P({FormatFact}) x Scope x Polarity
Scope    = (FixtureId?,RunId?,RevisionId?,StageId?)
Polarity = Positive | NegativeRule | RejectedRun
```

Tag embeddings are:

```text
EP PROVED       -> Dynamic (or Both when explicitly paired with static flow)
EP STATIC       -> Static
EP OBSERVED     -> Observed
EP HYPOTHESIS   -> Unknown

OP [D:S]        -> Dynamic scoped to S
OP [S:S]        -> Static scoped to S
OP [O:S]        -> Observed scoped to S
OP [N:S]        -> NegativeRule scoped to S
OP [U]          -> Unknown

FL [PD]         -> Dynamic
FL [PS]         -> Static
FL [PS+PD]      -> Both
FL [FF]         -> FormatFact
FL [HYP]/[OPEN] -> Unknown
FL [REQ:S]      -> proof obligation, not evidence
```

Evidence joins are permitted only for compatible scopes. A static fact from one
revision and a dynamic fact from another do not yield `Both`. An accepted stage
manifest is a separate gate: a local historical `Dynamic` lemma does not
manufacture an accepted dependency token.

## 5. Small-Step Transition System

Write `Sigma --label[e]--> Sigma'` for one transition justified by evidence
`e`. A transition mutates its owning component and the explicitly named
interface fields only.

The system has two causal partial orders. They are not collapsed into one
total chronology:

```text
TextFlow:
  TextSource
    < EPClosure
    < CandidateOwnerFrame
    < (AcceptedFlow
       or RejectedFlow < SuccessorEvidence < EPRecomposition < AcceptedFlow')
    < CommittedFlow
    < LayoutDescriptorEmission
    < DescriptorCommit

ObjectFlow:
  SourceControl
    < PartialResolve
    < Dispatch
    < OptionalSharedPlacement
    < CandidateOwnerFrame
    < (AcceptedFlow
       or RejectedFlow < SuccessorEvidence < AcceptedFlow')
    < CommittedOwnerPage
    < LayoutDescriptorEmission
    < DescriptorCommit
```

The orders may share source, frame, transition, and descriptor events only
through a defined interface morphism. In ObjectFlow, OP source resolution,
dispatch, and any actual placement observation precede the joined flow
rejection and page commitment. `OptionalSharedPlacement` may be absent even
when dispatch is `Shared`.

### 5.1 Source and control resolution

```text
resolve_text_source : SourceState(T) x Cursor(T) ⇀ TextSource
resolve_source_control : SourceState(T) x Cursor(T) ⇀ SourceControl

legal_boundary(Src,c)   resolve_source_control(Src,c) = sc
------------------------------------------------------------- SRC-RESOLVE
Sigma --src(sc)--> Sigma[OP.source_control := sc,
                         EP.probe := c]
```

Undefined resolution rejects the corresponding transition; it does not mean
`Null` or `BYPASS`. Ordinary text uses `resolve_text_source` and does not
synthesize a `SourceControl`. For layout controls, OP's filtered object
resolver is separately partial. Endpoint predecessor/stop rules remain in `EP`;
OP dispatch rules remain in `OP`.

### 5.2 Horizontal closure

The lane-local closure is imported as a partial judgment:

```text
EP.close(Src, line_context, probe) = endpoint
------------------------------------------------------------- EP-CLOSE
Sigma --horizontal(endpoint)--> Sigma[EP.endpoint := endpoint,
                                      EP.phase := Closed]
```

`EP.close` may traverse predicate, selector, generator, measure, scale, and
finalizer substeps, or bypass some of them. Its output is not yet a committed
descriptor and has no page-owner meaning.

### 5.3 Vertical candidate and owner admission

The endpoint-to-flow interface constructs a typed candidate:

```text
mu_EP_FL(endpoint, FL.cursor, owner_constraints)
  = (vertical_candidate,candidate_owner_frame)
--------------------------------------------------------------------- CANDIDATE
Sigma --candidate(vertical_candidate,candidate_owner_frame)-->
  Sigma[FL.candidate := vertical_candidate,
        FL.candidate_owner_frame := candidate_owner_frame]

FL.admit(candidate_owner_frame,vertical_candidate)
  = Accepted(accepted_witness)
------------------------------------------------------------- ADMIT
Sigma --admit(accepted_witness)-->
  Sigma[FL.accepted_flow := accepted_witness]
```

`mu_EP_FL` is partial until slot, source lineage, line-local context, and owner
roles are accepted. `FL.admit` is partial: a candidate frame/owner is not an
accepted witness, and an accepted witness is not a committed page witness.
Horizontal fit does not imply vertical admission.

### 5.4 Rejection and successor selection

```text
FL.admit(candidate_owner_frame,candidate)
  = Rejected(rejected_witness)
------------------------------------------------------------- REJECT
Sigma --reject(rejected_witness)-->
  Sigma[FL.rejected_flow := rejected_witness,
        Commit.rejection_serial := rejected_witness.rejectionSerial]

FL.select_successor(FL.cursor,directive,FL.graph,window)
  = successor_evidence
---------------------------------------------------------------- SUCCESSOR
Sigma --successor(successor_evidence)-->
  Sigma[FL.successor := successor_evidence,
        FL.candidate_owner_frame :=
          CandidateOwnerFrame(successor_evidence.toFrame,
                              successor_evidence.toOwner)]
```

The successor relation is partial and admits the FL H2a/H2b/H2c countermodels.
Its evidence carries successor frame, page, owner, and the selection boundary
(`CommitBoundary`, `ComposeBoundary`, or `IntermediateBoundary`).
`T-RESET-400` supplies none of those values.

### 5.5 Successor recomposition

Let `rho_FL_EP` provide the successor slot and accepted context to EP:

```text
rho_FL_EP(FL.cursor,successor_evidence,graph) = successor_line_context
EP.recompose(endpoint,successor_line_context) = endpoint'
---------------------------------------------------------------- RECOMPOSE
Sigma --recompose(endpoint')--> Sigma[EP.endpoint := endpoint']
```

`EP.recompose` is a sum of the compatible `F-CARRY`, `F-REFINALIZE`, and
`F-REPLAY` relations. It is undefined when XL-01 cannot discriminate them.

### 5.6 Object dispatch and placement

```text
OP.resolve : SourceControl x CallSerial ⇀ Option<RunLocalObject>

Dispatch = Mask(code)
         | Null(code,outwardResults)
         | Zero(object,target)
         | Specialized(object,target,outwardResults)
         | Shared(object,target=sharedExtentEntry,relation30,outwardResults)

OP.dispatch(source_control,OP.resolve) = dispatch
------------------------------------------------------------- OP-DISPATCH
Sigma --dispatch(dispatch)--> Sigma[OP.dispatch := dispatch]

outer3(Mask(_)) = BYPASS
outer3(Specialized(...)) = SPECIALIZED
outer3(Shared(...)) = SHARED
outer3(Null(...)) and outer3(Zero(...)) are undefined
```

`Mask` requires no outward/resolution event. `Null` requires an admitted
outward call whose partial resolver returned `None`. `Zero`, `Specialized`, and
`Shared` require a joined concrete object and actual outward target/result, as
defined by OP. `resolve` being undefined is not any dispatch constructor. The
shared target `sharedExtentEntry` denotes the single shared extent-selection
entry point (a fixed dispatch target in the modeled instance).

Shared dispatch is necessary but insufficient for placement:

```text
OP.dispatch = Shared(object,sharedExtentEntry,relation30,results)
OP.selectExtent(object,placementCall,class)
  = (primaryExtent,secondaryExtent,result)
placementParent(placementCall) = joinedSharedExtentCall
placementObject(placementCall) = object
result = if class=0 then primaryExtent else secondaryExtent
------------------------------------------------------------- OP-PLACE-SHARED
Sigma --placement(placement)--> Sigma[OP.placement := placement]
```

`OP.selectExtent` is the shared extent selector: given the object, the
placement call, and the numeric class, it yields the primary and secondary
extent fields and picks between them by class. If the partial extent-selector
call, nesting frame, object identity, numeric class, or selector equation is
absent, `OP.placement` remains `None` even under Shared. For `SPECIALIZED` and
`BYPASS`, shared placement is not synthesized. OP's `Relation30`, placement
record, anchor, and owner remain different sorts. The three OP verdict
coordinates remain independently recorded in `OP.verdict`.

### 5.7 Placement/page commitment

The object-to-flow morphism is defined only by a complete causal join:

```text
mu_OP_FL(OP.source_control,OP.dispatch,OP.placement?,OP.anchor?,FL.window)
  = placement_owner_evidence
FL.accepted_flow = placement_owner_evidence.acceptedFlow
FL.commit_owner_page(placement_owner_evidence) = committed_witness
---------------------------------------------------------------- PAGE-COMMIT-OBJECT
Sigma --page_commit(committed_witness)-->
  Sigma[FL.committed_flow := committed_witness,
        Commit.page_commits += committed_witness]
```

No rule exists from `Relation30` or `AnchorKey` directly to `OwnerKey`.

Ordinary text and non-object controls use a flow-only commitment rule; they do
not synthesize an OP dispatch:

```text
FL.accepted_flow = flow_evidence
FL.accepted_flow_window(FL.cursor,flow_evidence,window)
FL.commit_owner_page(flow_evidence) = committed_witness
---------------------------------------------------------------- PAGE-COMMIT-FLOW
Sigma --page_commit(committed_witness)-->
  Sigma[FL.committed_flow := committed_witness,
        Commit.page_commits += committed_witness]
```

### 5.8 Descriptor emission

```text
EP.endpoint = endpoint
FL.committed_flow = committed
committed.accepted.slot = endpoint.slot
layout_descriptor_emission(endpoint,committed,emissionSerial) = descriptor
dc = DescriptorCommit(descriptor,
      committed.accepted.candidate.frame,
      committed.accepted.candidate.owner,
      committed.page,
      committed.transitionSerial,
      committed.commitSerial,
      emissionSerial)
---------------------------------------------------------------- EMIT
Sigma --descriptor_commit(dc)-->
  Sigma[Commit.descriptors += dc]
```

`layout_descriptor_emission` is an independent observed witness, not a value
derived from endpoint or geometry equality. EMIT therefore requires an accepted
and committed flow/page witness plus the layout descriptor emission. For a
rejected current frame, emission may occur only for the accepted successor
context. Missing current-frame emission is admissible when rejection,
successor, transition/commit, and destination emission serials form one chain.

## 6. Interface Morphisms

Each interface is partial, type preserving, and provenance preserving.

```text
mu_EP_FL : EP.EndpointEvidence x FL.RoleEvidence
           -> Option<FL.VerticalCandidate>

rho_FL_EP : FL.SuccessorEvidence
            -> Option<EP.SuccessorLineContext>

mu_OP_FL : OP.DispatchOptionalPlacementEvidence x FL.FlowWindow
           -> Option<FL.PlacementOwnerEvidence>

mu_OP_SOURCE : OP.SourceControlEvidence x OP.DispatchEvidence
               -> Option<OP.DispatchOptionalPlacementEvidence>

mu_EP_ANCHOR_FL : EP.AnchorEvidence x FL.FlowWindow
                  -> Option<FL.AnchorReservationEvidence>

mu_DESCRIPTOR : EP.EndpointEvidence x FL.CommittedFlowWitness
                x LayoutDescriptorEmission
                -> Option<DescriptorCommit>

mu_CANONICAL : XL04aEvidence x XL02aEvidence x XL02bEvidence
               x XL02cEvidence x XL03Evidence x FL06Evidence
               -> Option<CanonicalClosureEvidence>
```

They are defined only when run, revision, paragraph/source lineage,
parent/child serials, fixture identity, and dependency role identities agree.
For any dynamic or cross-lane join, every input must share the same fixture,
opened-document identity, and document-open generation; evidence observed under
differing scopes cannot be joined. Each morphism carries its input provenance;
it cannot raise the evidence grade or widen its scope.

The stage realizations are:

```text
XL-01  realizes mu_EP_FL followed by rho_FL_EP and mu_DESCRIPTOR
OP-03a realizes OP-local mu_OP_SOURCE and the prerequisite portion of mu_OP_FL
XL-02  realizes mu_OP_FL and page commitment
XL-03  realizes mu_EP_ANCHOR_FL plus FL reservation/commit
XL-04a is the internal endpoint/flow recurrence substage with exact lineage
        EP-03 + EP-07 + FL-03 + XL-01
XL-04  realizes mu_CANONICAL after XL-04a, XL-02a/b/c, XL-03, and FL-06
       provide accepted endpoint, object, table, and anchor identity lineages
```

`XL-04a` is not a unified roadmap ID and does not add a stage to the
proof-obligation matrix. It names the currently executable line-local
recurrence substage. Canonical `XL-04` remains the final closure theorem.

## 7. Partiality And Unknown Parameters

The unified unknown family is the tagged disjoint union:

```text
Theta = InEP(ThetaEP) + InOP(ThetaOP) + InFL(ThetaFL) + ThetaInterface
```

`ThetaInterface` includes:

```text
paragraphHeaderAddressModeMapping
endpointToVerticalExtent
successorReplayBoundary
objectEndpointTransform
anchorReservationCoupling
placementOwnerTransfer
lineLocalAdjustmentTransfer
firstEvenOddSelectionBoundary
rowSpanFragmentMapping
rowSpanProgressMeasure
overheightAdmissionPolicy
overheightProgressMeasure
```

For hidden tracked-change anchors, the view-policy boundary is now narrower. In
the track-change manager's view-policy field, bit `0x02` selects
original-content polarity when set and final-content polarity when clear; it
causes Insert or Delete entries, respectively, to acquire `hide`. Bit `0x01` is
track-change password/protection state and is excluded from layout invalidation
when changed alone. The action dispatch for track-change viewing proves `0x04`
set is plain Original/Final without changes and memo, `0x08` set is
non-inline/balloon presentation, `0x10` enables format/shape changes, and
`0x20` enables insert/delete changes. A reviewer's `mark == 1` selects/includes
that reviewer. None is an advance value, although the view policy can change
committed geometry by changing the included logical content and eligible hidden
anchor.

The invalidation receiver is also list-resolution sensitive. The
document-list dirty-notification receiver first climbs from the current list to
the active list owner. The shared marker-list, document-list, and root-list
path maps that list to a resolved-owner slot (`list + owner-offset`), then ORs
the notification into the resolved owner's dirty word (`(*(owner-slot))
+ dirty-word-offset`). The list constructor helper proves the resolved-owner
slot points back to the list itself for this family. Thus a track-change
`0x400` notification reaches the resolved list's own dirty word, not
necessarily the document's immediately embedded list. The concrete dirty-state
dispatcher preserves the list receiver, tests the resolved dirty word against
`0x400`, and calls the cache-rebuild routine. That path resets and rebuilds
list-scan caches against section definitions; because the same bit is also in
mask `0x600`, it invalidates header/footer/section-definition cache state as
well. Several earlier same-offset candidate receivers resolve to the
find-and-replace family and remain rejected. Static producer-to-consumer
identity is high confidence; the passive relationship awaits one same-list live
cross-check.

The `0x400` scan's format resolver probes tracked-change class `0x13 =
ParaShape`, selects the tracked prior shape for original-view polarity or the
base shape otherwise, and supplies that effective shape to the section-cache
refresh. The same resolver feeds line-record construction and downstream
sizing/spacing routines. In the formal model, revision view therefore changes
`effectiveParaShape`; it never substitutes for an advance or coordinate by
itself.

For the modeled instance, the geometry-bearing portion of that value is
modeled as follows. Offsets are field positions within the native paragraph
shape record, not HWPX record offsets:

```text
EffectiveParaShape = {
  attr1:                 u32 @ attr1-field,
  attr2:                 u32 @ attr2-field,
  marginLeft:         Metric @ marginLeft-field,
  marginRight:        Metric @ marginRight-field,
  indent:             Metric @ indent-field,
  spacingBefore:      Metric @ spacingBefore-field,
  spacingAfter:       Metric @ spacingAfter-field,
  lineSpacingType:      u4 @ (lineSpacingType-field & 0x0f),
  lineSpacingValue:  Metric @ lineSpacingValue-field,
  borderOffsets:       i16[4] @ borderOffsets-field,  # left,right,top,bottom
  borderFillId:          u16 @ borderFillId-field,
  tabDefinitionId:       u16 @ tabDefinitionId-field
}

attr1.alignment          = bits(2..4)
attr1.breakLatinWord     = bits(5..6)
attr1.keepNonLatinWord   = bit(7)
attr1.snapToGrid         = bit(8)
attr1.condense           = bits(9..15)
attr1.widowOrphan        = bit(16)
attr1.keepWithNext       = bit(17)
attr1.keepLines          = bit(18)
attr1.pageBreakBefore    = bit(19)
attr1.verticalAlignment  = bits(20..21)
attr1.headingType        = bits(23..24)
attr1.borderConnect      = bit(28)
attr1.borderIgnoreMargin = bit(29)
attr1.paragraphTailShapeOpaque = bit(30)

attr2.lineWrap           = bits(0..1)
attr2.reservedOpaque     = bits(2..3)
attr2.autoSpaceEaEng     = bit(4)
attr2.autoSpaceEaNum     = bit(5)
attr2.suppressLineNums   = bit(6)
attr2.checked            = bit(7)
```

`attr2.reservedOpaque` is deliberately not assigned layout semantics. HWP 5.0
table 40 labels bits `2..3` reserved, the current HWPX reader leaves them
untouched, and the current class only proves that one paragraph-shape accessor
slot can read them. Same-slot calls on larger UI/control objects are type
mismatches; no formatter consumer of an exact paragraph shape has been proved.
These bits cannot enter `TrueGeometry` unless that missing consumer join is
captured.

`Metric` is a tagged native scalar. `raw >> 1` is its signed payload; low bit
zero selects an absolute value, while low bit one scales the payload by the
consumer-selected paragraph basis through `MulDiv(payload, basis, 100)`. The
indent consumer uses basis axis `0`; the observed margin and paragraph spacing
consumers use axis `1`. The model must decode this tag before any coordinate
arithmetic.

The corresponding native consumers partition true geometry into three effects.
The margin consumer applies left/right margins to horizontal frame bounds; the
tab/line-position consumer uses negative indent plus left margin as a
tab/line-position candidate. The vertical-placement consumer applies
spacing-before/after to vertical placement, while the line-metrics consumer
computes line height/leading from line-spacing type/value and vertical
alignment. Alignment, word-break, condense, tabs, and section grid are consumed
by their respective dedicated consumers. Therefore true geometry is not limited
to character advance: it also includes available line width, candidate
endpoints, vertical origin, line height, and grid quantization.

`attr1.borderConnect` is a true vertical-geometry policy, not merely a border
paint hint. The border-connection predicate compares the current effective
ParaShape with its previous and next effective neighbors and computes two
connection predicates. The border-offset resolver then omits
`borderOffsets.top` when the previous border is connected and omits
`borderOffsets.bottom` when the next border is connected. The top term is added
by line-record construction to the emitted line record's vertical-position
field, the observed line vertical position. The bottom term is returned through
the line-metrics finalizer and added by the paragraph-advance accumulator to
the running vertical coordinate (`formatter.runningVertical`). Thus:

```text
TopBorderGeometry(a) = 0                         if Connected(prev(a), a)
                     = borderOffsets(a).top      otherwise

BottomBorderGeometry(a) = 0                      if Connected(a, next(a))
                        = borderOffsets(a).bottom otherwise
```

The exact connection predicate also considers border presence and, in the
stricter mode, border-style and offset equality. The bit does not suppress
left/right offsets and has no proved path to horizontal frame width or endpoint
selection. Its semantic class is `TrueVerticalGeometry`: it can change line
origin, paragraph extent, and pagination without changing an inline advance.

`attr1.borderIgnoreMargin` remains in a different domain. Its exact direct
consumer changes the paragraph-border rectangle by decoded left/right margins
and vertical border extensions. The caller chain reaches paint/bounds
operations, while no type-correct path to line-slot construction or endpoint
selection is established. The model therefore assigns it to `PaintGeometry`,
not `TrueGeometry`, pending contrary evidence.

`attr1.paragraphTailShapeOpaque` preserves HWP 5.0 bit `30`, whose published
name is `문단 꼬리 모양` (paragraph tail shape). Native identity is exact: the
paragraph-shape accessor for this slot returns `(attr1 >> 30) & 1`. Semantic
admission is deliberately absent. A whole-image high-bit data-flow scan,
direct-call scan, effective-ParaShape virtual-call scan, and return-sensitive
slot-call classification found no type-correct formatter, flow, or paint
consumer. Current HWPX mapping also has no producer for the bit. Therefore:

```text
Geometry(attr1.paragraphTailShapeOpaque) = none  # modeled instance
SerializationDomain = HWP5 packed attr1
HWPXMapping = absent
```

This is a scoped negative result, not a claim that every HNC version or legacy
file mode ignores the field. A consumer must be joined to the exact ParaShape
getter or field before the bit can leave `OpaqueCompatibilityState`.

`attr1.reserved31Opaque` preserves the unnamed high bit only where a native
ParaShape-compatible archive block already carries it. The export path
normalizes a boolean mirror field into packed bit 31. The current OWPML
importer at the same slot maps the public ParaShape members without setting bit
31, while the native importer slot is a no-op stub. The primary ParaShape
interface exposes no bit-31 getter, and whole-image direct-mask, shift,
mirror-write, and effective-shape searches establish no type-correct consumer.
Therefore:

```text
Geometry(attr1.reserved31Opaque) = none  # modeled instance
SerializationDomain = native secd ParaShape-compatible archive block
HWPXMapping = absent
PublicSemanticName = unknown
```

This field must not be inferred from the published bit-30 paragraph-tail name.
The two adjacent bits have different interface evidence: bit 30 has a primary
ParaShape getter, while bit 31 does not.

The HWPX reader proves the four pagination names by mapping the ordered
`breakSetting` bytes directly to native `attr1` masks `0x10000`, `0x20000`,
`0x40000`, and `0x80000`. The first three feed the proved widow/orphan,
keep-with-next, and keep-lines placement constraints; page-break-before feeds
the paragraph flow-directive decoder. They are page geometry, not `attr2`
extensions.

The model keeps the two observed `0x400` values in separate nominal domains:

```text
FlowDirective.ColumnBreak       = 0x400
ListDirty.TrackChangeViewLayout = 0x400
```

`FlowDirective.ColumnBreak` participates in break normalization and owner
transition. `ListDirty.TrackChangeViewLayout` lives in resolved-list state,
selects effective revision content/ParaShape, invalidates caches, and is
consumed and cleared. Equality of their integer encodings permits no cast or
behavioral inference between them.

Important lane-local unknowns remain unchanged:

- EP: exact outer-caller and live multi-node identity preservation for the
  now-observed node-local mode; the continuation membership writer is already
  localized to the hidden-track-change `0x10/0x11` anchor gate; dynamic
  confirmation of the now-traced document track-change `& 0x11`
  master-page/track-change-table sources that scope the note-only
  track-change-`hide` bypass request; remaining mode/policy fields,
  candidate-receiver runtime class, numeric-pipeline output meaning, selector
  fallback, configurable-set provenance/behavior, and successor-frame mode
  preservation;
- OP: family masks, resolver domains, specialized result meanings, numeric
  class meanings, primary/secondary extent field meanings, `Relation30`, TAC
  sensitivity, endpoint transform, owner transfer;
- FL: successor ownership H2a/H2b/H2c, eager/lazy frame construction,
  first/even/odd control selection boundary, header/footer reservation,
  fragment/header materialization, row-span mapping/progress, over-height-row
  admission/progress, and endpoint replay amount.

An unknown parameter may be constrained by evidence without becoming total.
Undefined input yields rejection or `UNRESOLVED`, never a default branch.

## 8. Safety, Non-Interference, And Restoration

Every accepted global execution satisfies:

1. **Revision scope:** all evidence and signatures name the modeled revision or
   a separately re-derived semantic instance.
2. **Typed non-aliasing:** object, relation30, placement, anchor, owner, frame,
   page, descriptor, and source identities do not coerce.
3. **Run locality:** raw pointers never cross `RunId` boundaries.
4. **Legal source boundaries:** endpoint and descriptor positions do not split
   acknowledged native atomic spans.
5. **Source continuity:** committed intervals are ordered and contiguous except
   for a separately proved control-span transition.
6. **Horizontal/vertical separation:** horizontal fit does not imply owner
   admission; vertical rejection does not retroactively change the current
   endpoint without `rho_FL_EP` evidence.
7. **Dispatch separation:** `BYPASS`, `SPECIALIZED`, and `SHARED` do not invent
   one another's events; shared placement exists only under SHARED.
8. **Owner separation:** neither anchor identity nor `Relation30` proves
   ownership.
9. **Causal commitment:** destination descriptors after rejection name the
   accepted successor frame/slot and joined transition/commit serials.
10. **Conditional reservation monotonicity:** only when an accepted
    `ReserveBody` relation selects reservation semantics and its before/after
    page geometry is otherwise identical, the reserved body interval is a
    subset of the unreserved interval. Overlay and unresolved selection models
    carry no monotonicity claim.
11. **Fragment progress:** table continuation strictly advances source coverage
    or vertical progress and never duplicates source ownership.
12. **Lane non-interference:** an EP-local step changes only `EPState`; OP-local
    only `OPState`; FL-local only `FLState`. Product components change together
    only through a named interface morphism.
13. **Input isolation:** parallel evidence from different runs, fixtures, or
    revisions cannot be merged into one transition.
14. **Restoration:** for one declared perturbation,
    `normalize(S0)=normalize(S0r)` and `normalize(S0)!=normalize(S1)`, including
    source, descriptor, slot, endpoint state, owner/frame/page, and applicable
    placement/fragment relations.

Non-interference is a model safety rule, not a claim that the layout engine's
internal memory regions are disjoint.

## 9. Determinism And Confluence Boundaries

Only these scoped claims are imported:

- OP dispatch constructors are exclusive under stable source/call/object
  identity.
- When the partial shared extent-selector call, nesting/object identity,
  numeric class, and selector equation are all witnessed, OP extent selection is
  deterministic: class zero selects the primary extent, otherwise the secondary
  extent. Shared dispatch alone entails no placement.
- FL `T-RESET-400` uniquely commits directive zero and resolves opaque flow axis
  zero under its exact premise.
- EP's historical candidate conversion and numeric fit equation have fixed
  arithmetic forms for their scoped evidence.

No global determinism theorem follows. In particular, unknown policy fields,
successor selection, owner construction, native allocation, and replay amount
can make `Sigma` branch.

There is no global confluence theorem. A conditional commuting square is
allowed only for two transitions whose read/write sets are disjoint and whose
provenance scopes match:

```text
Sigma --a--> Sigma1 --b--> Sigma12
  |                         ^
  b                         a
  v                         |
Sigma2 ---------------------+
```

This is an algebraic property of the product model. It does not assert a native
scheduler confluence. Transitions sharing source revision, endpoint, owner,
frame, slot, or endpoint address mode are not assumed to commute.

## 10. Countermodel Compatibility

A global countermodel is a tuple `(mEP,mOP,mFL)` whose interface values agree.
The `mFL` coordinate itself contains successor selection, frame construction,
first/even/odd control selection, reservation, header materialization, row-span
progress, and over-height-row progress choices. Compatible examples include:

```text
(F-CARRY,      SHARED,      H2a, eager frames, reservation, header reuse)
(F-REFINALIZE, SPECIALIZED, H2b, lazy frames,  overlay,     header clone)
(F-REPLAY,     BYPASS,      H2c, eager frames, late reserve, fragment-local)
```

These rows are examples of compatibility, not evidence that the combinations
occur. A combination is rejected when it violates an interface type or proved
theorem. Incompatible examples are:

- terminal raw `0x400` becoming committed `0x200`, contradicting
  `T-RESET-400`;
- `SPECIALIZED` using the shared target/placement equation;
- `BYPASS` containing outward resolution events;
- anchor or `Relation30` coerced into `OwnerKey`;
- `F-CARRY` with successor endpoint child events or a mismatched destination
  endpoint;
- repeated-header presentation duplicating source ownership;
- any model requiring a descriptor in a rejected current frame without native
  emission evidence.

Additional compatible unresolved families are:

```text
PageClassSelection:
  PC-CONSTRUCT   select First/Odd/Even controls during page construction
  PC-COMPOSE     select them at owner composition
  PC-FALLBACK    resolve an absent class through an opaque fallback relation

RowSpanProgress:
  RS-SOURCE      progress is strict growth of covered source-cell span
  RS-VERTICAL    source coverage may hold while fragment vertical progress grows
  RS-MIXED       either measure may advance, with complete final source coverage

OverheightProgress:
  OH-FORCE       admit one oversize row/fragment to guarantee source progress
  OH-FRAGMENT    split into positive-height fragments with strict coverage growth
  OH-DEFER       defer once to a successor frame, then require a different
                 accepted state or reject as zero progress
```

These are parameter families, not native names. A model that repeats identical
source coverage and identical vertical state is incompatible with the strict
progress invariant. First/even/odd selection cannot be inferred from final
paint; it requires the selected source control, page class, owner/frame, and
selection boundary in one causal witness.

Because current native matrices are unaccepted, multiple compatible global
countermodels remain. Mathematical completeness requires listing them and their
discriminators, not choosing one.

## 11. Theorem Dependency Graph

Imported historical facts are rooted in their preserved revision scope, not in a
future acceptance verdict:

```text
Preserved static/dynamic evidence (modeled revision)
  |
  +-> EP L1..L6 (scoped output separation, candidate arithmetic,
  |              stop ownership, fit form, finalizer identity/restoration)
  |
  +-> OP O1..O7 (dispatch exclusivity, shared selector, causal join,
  |              restoration schema, endpoint independence, non-aliasing,
  |              acquisition non-semantics)
  |
  `-> FL T-RESET-400

future acceptance gate ---> authorizes future capture on one fresh
                            semantic instance only
```

The acceptance gate checks that a future run matches the modeled revision; it
does not retroactively prove or re-prove historical lane facts.

Future cross-lane theorems require accepted stages:

```text
EP-01 -> EP-02 -> EP-03 ---------------------------+
                    +-> EP-04                       |
                    +-> EP-06 -> EP-05              |
EP-04 + EP-05 + EP-06 -> EP-07 --------------------+-> XL-01
FL-01 + FL-03 ---------------------------------------+-> XL-01
EP-06 ------------------------------------------------> XL-03

OP-00 -> applicable OP-01 -> OP-02 ----------------+-> OP-03a
FL-01 ---------> FL-03 ---------------------------+
OP-03a + FL-01 + FL-03 ---------------------------> XL-02a/b
accepted XL-02a/b ---------------------------------> XL-02c

OP-03b acquisition --qualified family--> restart OP-01 discovery

FL-01 -> FL-03 -> FL-04 --------------------------> XL-03
FL-01 + FL-03 + OP-00 -> FL-05 -> FL-06 ----------+
EP-03 + EP-07 + FL-03 + XL-01 --------------------> XL-04a

XL-04a + XL-02a/b/c + XL-03 + FL-06
  + accepted table/object/anchor identities ----------> XL-04
```

An arrow means logical dependency, not that the antecedent is currently
accepted. OP-03b is detached acquisition: qualification restarts the acquired
family at OP-01 and has no arrow into OP-02, OP-03a, or XL-02 by itself.
XL-04a has exactly the endpoint/flow executable lineage shown above. Canonical
XL-04 additionally requires the accepted object, table, and anchor branches;
XL-04a alone cannot claim their composition.

## 12. Implementation Projection Boundary

Let `Serialized` denote facts actually represented by HWP/HWPX, such as UTF-16
source, controls, TAC, section/column geometry, page-control references, table
policy, styles, and object attributes. Let `Derived` denote runtime-only
closure state: call serials, runtime pointers, endpoint candidates, cumulative
arrays, placement records, owner/frame/page allocations, rejection windows, and
committed descriptors.

```text
parse      : HWP_or_HWPX -> Serialized
interpret  : Serialized x Theta -> PartialLayout
promote    : AcceptedEvidence -> ProvenRelation
project    : Serialized x ProvenRelation -> ImplementationConstraint
```

`project` is undefined for `Observed`, `Unknown`, rejected evidence, or an
unaccepted stage dependency. It must never:

- store runtime pointers, code addresses, call serials, or paint coordinates as
  format semantics;
- serialize a computed endpoint partition as if HNC stored it;
- equate imported/generated/stored provenance with a native layout branch;
- encode numeric placement classes or opaque axis zeros with guessed names;
- type an anchor or `Relation30` as ownership; or
- choose carry/re-finalize/replay without XL-01 evidence.

The current implementation domains are [model](../../../src/model),
[HWPX parser](../../../src/parser/hwpx), and
[HWPX serializer](../../../src/serializer/hwpx). This formal document changes no
implementation and authorizes none. A future code change must cite an accepted
relation, state its projection, and preserve round-trip format facts separately
from derived layout behavior.

## 13. Proof-Obligation Matrix

| Stage | Relation or interface to close | Required evidence and non-claim |
| --- | --- | --- |
| `G-00` | Future revision-scoped semantic instance and reviewed observation surface | Fresh semantic instance with opened-document identity, revision tuple, signatures, and probe hashes; proves no layout semantics and does not re-prove historical facts. |
| `EP-01` | Active target of abstract `EP.candidate_receiver` | Two fresh identical A1 target signatures and exact descriptor restoration; discriminate the observed receiver from alternative receiver dispatchs. |
| `EP-02` | One complete receiver/production/fit instance | Joined candidate, selected receiver, pipeline inputs/output, cumulative value, width, selector, wrapper, state, and descriptor row; do not name or generalize the produced quantity from one row. |
| `EP-03` | Width-controlled measure discrimination | A1/A2 paired widths with source/style/matrix/spacing/owner/frame/cumulative revision fixed; narrow, do not guess, measure/scale hypotheses. |
| `EP-04` | Wrapper override ownership | Join selector `211`, second predicate, cumulative values, optional finalizer, wrapper/descriptor `302`; do not substitute selector for wrapper. |
| `EP-05` | Mode and finalizer branch family | Native `0x00/0x20/0x40` modes and six two-sided boundary pairs with reviewed output argument; `0x40` is format/static `BREAK_WORD`, while reserved `0x60` remains unnamed. |
| `EP-06` | Ordinary/control stop and anchor relation | Sixteen fresh cases with atomic span, predecessor, payload, descriptor, settled/included owner tuple; do not type anchor as owner. |
| `EP-07` | Endpoint state propagation | Revision required: two discovery and two confirmation runs joining wrapper state to paragraph-header state and descriptor position. |
| `OP-00` | Known-positive shared calibration | Source -> resolver -> actual shared-extent entry -> shared extent -> pointer-free endpoint recurrence; calibration is not general family semantics. |
| `OP-01` | Five-way novel-family dispatch product | Close source-span, dispatch, and endpoint-effect coordinates independently; a same-offset extent-selector call is not placement. |
| `OP-02` | Owner/mode/TAC matrix and numeric class subsets | Accepted OP-00 plus applicable OP-01 family lineage; eighteen variants, paired triples and repeats; do not name numeric classes or assert TAC equivalence. |
| `OP-03a` | OP-local `mu_OP_SOURCE` and prerequisite `mu_OP_FL` chain | Exact per-fixture OP-02 mode plus FL roles; OP source/object/outward/optional actual placement/anchor/frame/page chain; no EP dependency or role coercion. |
| `OP-03b` | Detached acquisition qualification only | Genuine visible/selectable unknown object, payload/control identity, save/reopen; proves no dispatch or ownership and restarts that family at OP-01 discovery. |
| `FL-01` | Neutral page/frame/column role identities | Terminal/nonterminal/page/natural controls and complete directive/predicate/commit/owner windows; axis values remain opaque until discriminated. |
| `FL-02` | `select_successor`, H2a/H2b/H2c | Successor frame/page/owner and exact Commit/Compose/Intermediate selection boundary, exact intermediate write, first successor compose, and reopen recurrence; reset alone is insufficient. |
| `FL-03` | Frame graph and `rho_FL_EP` slot roles | Seven section/column variants, ordered construction, successor frame/page/owner roles, descriptor slot joins, exact geometry and restoration. |
| `FL-04` | Page-control selection and conditional body reservation | First/even/odd presence/absence, explicit selection boundary, and isolated height pairs joined to body limits; paint equality is insufficient and monotonicity applies only to proved reservation semantics. |
| `FL-05` | Repeated-header materialization | Source table -> object dispatch -> reviewed fragment lookup -> placement -> frame/page chain; distinguish reuse/clone/presentation. |
| `FL-06` | Fragment conservation and progress | Complete source-cell coverage, source order, no loss/duplication, strict progress for row-span/over-height cases. |
| `XL-01` | `mu_EP_FL`, `rho_FL_EP`, successor closure | Six equal/unequal natural/explicit cases twice; select carry/re-finalize/replay only from complete rejection-to-descriptor chains. |
| `XL-02a` | TAC `mu_OP_FL` and page commitment | Accepted TAC outer mode, applicable optional placement plus anchor/frame/page chain and recurrence; Shared alone does not prove placement and no inner-owner substitution is allowed. |
| `XL-02b` | Non-TAC exclusion and page commitment | Forty-eight paired variants with applicable outer mode and owner chain; specialized/bypass never synthesize shared records. |
| `XL-02c` | Nested outer/inner owner separation | Accepted outer mode plus separately mapped inner source/owner across 24 variants; no outer/inner role coercion. |
| `XL-03` | Anchor-selection-reservation interface | EP-06 anchor, FL-03 frame, FL-04 selection/body limit, destination descriptor across ten pairs; pointer/paint-only evidence rejects. |
| `XL-04` | Canonical final closure through `mu_CANONICAL` | First close internal XL-04a from EP-03 + EP-07 + FL-03 + XL-01 using four style/width fixtures across core/Bidi/shaping and 24 fresh runs with no stored endpoint partition. Canonical XL-04 then requires accepted XL-04a + XL-02a/b/c + XL-03 + FL-06 and their table/object/anchor identities. |

No row is discharged merely because its inputs are prepared.

## 14. Exact Authorities

The module equations and local evidence are authoritative at:

- [EP formal semantics](endpoint_line_closure_semantics.md)
- [OP formal semantics](object_placement_semantics.md)
- [FL formal semantics](flow_pagination_semantics.md)

The composed module index is at [README](README.md).
