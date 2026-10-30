# TASK002 contract conflict — paused

The first stage run passed TST-BEH-003 then failed TST-BEH-004 at tests/observation-delivery/tests/contracts.rs:197. Locked oracle asserts exact tick0 position[5,10] for P0observation[10,20], priorP25 and requested sigma5. The unchanged root KalmanFilter Cholesky update returned [4.999999999999999,9.999999999999998]. The scalar mathematical expectation is correct in real arithmetic; exact floating equality is incompatible with this existing operation ordering on the declared native target.

REQ020 preserves unchanged F/Q/Joseph/variance semantics; REQ031 and TST-ACC-001 require bitwise old-run equivalence. No estimator rewrite or output rounding is permitted to force an exact mathematical value. No test/catalog/lock/oracle or legacy numerical source changed. Main received exact failure and must decide a controlled contract revision before work continues. Original command/stdout/stderr/sourcehash remain task002-tests.json. Current production sourceSHA256 e262afab2ba90cd37fe76f49298487704640c500b2a4bbb2a6622f0fd7941d25.

The new replay implementation is not accepted or fully tested. Codecs remain diagnostic. No resource/native/final acceptance is claimed.
