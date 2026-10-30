use fusion_lab::{ObservationId,Observation,ReplayModel,DeliveryPlan,Trace,ReplayResult,ReplaySnapshot,LedgerEntry};
fn identity(owned:ObservationId){let _=ObservationId{..owned};}
fn observation(owned:Observation){let _=Observation{..owned};}
fn model(owned:ReplayModel){let _=ReplayModel{..owned};}
fn plan(owned:DeliveryPlan){let _=DeliveryPlan{..owned};}
fn trace(owned:Trace){let _=Trace{..owned};}
fn result(owned:ReplayResult){let _=ReplayResult{..owned};}
fn snapshot(owned:ReplaySnapshot){let _=ReplaySnapshot{..owned};}
fn ledger(owned:LedgerEntry){let _=LedgerEntry{..owned};}
