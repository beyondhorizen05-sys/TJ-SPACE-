# SYSTEM 30 — Performance & Streaming Budget

## D1 Scope
Owns frame-budget allocation, streaming priority, LOD pressure response, Performance Mode, and F1.3 frame targets. It does not own rendering assets, world geometry, navigation, panels, interaction, telemetry or alerts.

## D2 Data model
FTJFrameBudgetAllocation, FTJPerformanceCost, FTJStreamingCandidate, FTJPerformanceReport, registered systems, districts, LOD state and performance-mode state.

## D3 Interface
AllocateFrameBudget(budgetMs); ReportFrameCost(system,ms); PrioritizeStreaming(cameraPose,budget); DemoteLOD(actorClass,level); PromoteLOD(actorClass,level); EnterPerformanceMode(); ExitPerformanceMode(); GetPerformanceReport(). Registration helpers establish system and district inputs.

## D4 Implementation
Budgets use priority weights: Critical 35%, High 30%, Normal 25%, Cosmetic 10%, normalized across registered systems. Streaming ranks districts by screen importance and camera distance. LOD state moves toward larger levels under pressure and toward smaller levels when headroom returns. Performance Mode is an explicit state transition consumed by the rendering integration: it requests baked-lighting fallback while preserving the established visual style. Target frame times are 16.6667 ms desktop and 11.1111 ms VR.

## D5 Integration
Systems 13–29 only. Critical registrations are reserved for Systems 28 interaction, 27 panels and 22 alerts. System 13 rendering foundation supplies the visual baseline; System 14 supplies district/world geometry inputs; Systems 15–25 and 26–29 provide runtime costs and visual/accessibility constraints.

## D6 Failure modes
Invalid budgets/costs are rejected; unknown systems are safely registered at Normal priority; empty streaming inputs return an empty result; invalid LOD values are rejected; repeated Performance Mode transitions are idempotent. The allocator preserves a dedicated Critical priority class so operator-critical workloads are not classified as cosmetic.

## D7 Tests
1. Allocate 16.6667 ms and verify allocation totals.
2. Allocate 11.1111 ms for VR.
3. Report costs and verify per-system accounting.
4. Register critical systems and verify their Critical allocation.
5. Verify cosmetic systems receive the smallest priority share.
6. Rank districts by camera distance and screen importance.
7. Demote and promote an actor class.
8. Enter/exit Performance Mode.
9. Verify report contains all costs and allocations.
10. Reject negative/NaN budgets and costs.

## D8 Acceptance
- [x] Desktop 60-FPS target
- [x] VR 90-FPS target
- [x] Frame budget allocation
- [x] Per-system reporting
- [x] Streaming prioritization
- [x] LOD demotion/promotion
- [x] Performance Mode state
- [x] Baked-lighting fallback contract
- [x] Style preservation contract
- [x] Critical priority class
- [x] Cosmetic-first degradation policy
- [x] Performance report
- [x] Systems 13–29 integration only
- [x] No System 31+ references
