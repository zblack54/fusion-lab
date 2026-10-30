#![allow(dead_code)]
use fusion_lab::{ObservationId,Observation,ReplayModel,DeliveryPlan,Trace,ReplayResult,ReplaySnapshot,LedgerEntry,State};
fn read_only_api(identity:ObservationId,observation:&Observation,model:&ReplayModel,_plan:&DeliveryPlan,trace:&Trace,result:&ReplayResult,snapshot:&ReplaySnapshot,entry:&LedgerEntry){
let _=identity.sensor();let _=identity.sequence();let _=observation.id();let _=observation.sample_tick();let _=observation.planned_arrival_tick();let _:[f64;2]=observation.value();
let _:f64=model.assumed_position_sigma();let _:f64=model.assumed_velocity_sigma();let _:f64=model.acceleration_density();let _=trace.observations();let _=trace.duration_seconds();let _=trace.pending_capacity();let _=trace.drop_every(fusion_lab::Sensor::Position);
let _=result.snapshots();let _=result.ledger();let _=result.arrival_order();let _:State=snapshot.estimate();let _=snapshot.covariance();let _=snapshot.held_position();let _=snapshot.integrated_position();let _:Option<u16>=entry.queue_occupancy();let _=entry.sample_tick();let _=entry.planned_arrival_tick();let _=entry.decision_tick();let _=entry.outcome();
}
