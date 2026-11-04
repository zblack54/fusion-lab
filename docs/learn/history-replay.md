# Correcting with late observations

Status: approved, locked design for [issue #22](https://github.com/zblack54/fusion-lab/issues/22).
The constructor slice is verified; correction behavior and final feature acceptance
remain outstanding. The existing
[timestamped replay](observation-delivery.md) still rejects stale observations.

An observation has a sampling time and an availability time. Updating the current
filter as if an old measurement described the present changes its meaning. The
proposed alternative restores an earlier checkpoint, applies measurements in
sampling order, and predicts forward using only information available now.

For example, take a velocity observation [2, -2] m/s sampled at tick0 and arriving
at tick1, with no other observations. Start from zero estimated state and zero
baselines, per-axis position variance25, velocity variance4 and position/velocity
cross covariance0. With zero acceleration density and measurement variance4,
the scalar Kalman gain is4/(4+4)=1/2. Correcting at the sample gives
velocity [1, -1] m/s. Predicting one10ms interval gives current position
[0.01, -0.01] m. Per axis, the current covariance is Ppp=25.0002,
Ppv=0.02 and Pvv=2. These are independent mathematical expectations; floating
tests use a stated component tolerance rather than assuming exact decimal bits.

The online tick0 estimate remains what the system knew at tick0. Internal
checkpoints can be revised; already emitted online snapshots cannot. Replacing
old online estimates with hindsight would give a misleading comparison with
the stale-drop policy. Truth remains with the evaluator and never enters the
correction API.

The proposed history window is at most500ticks (five seconds). A late observation
at age exactly the window remains eligible; one beyond it is explicitly dropped.
The initial prior supplies the anchor before sample0. A batch merges late and
on-time arrivals once in sample/position/velocity order. It chooses the replay
branch before doing current-tick work, so that work is not silently duplicated.

History size and process memory are different quantities. The draft must count
committed checkpoints, candidate transaction storage, reserved capacity, the
complete source trace, online outputs and verification buffers. A finite history
alone does not establish a process memory limit. The resource plan will measure
declared stress profiles with actual binary, source and machine provenance.

The zero-window comparison also preserves existing raw-baseline arithmetic.
An independent boundary check found that extreme finite direct observations can
overflow raw integration while the filter stays finite. [Issue #79](https://github.com/zblack54/fusion-lab/issues/79)
tracks that separate policy decision. The draft does not promise finite raw
baselines for every finite floating-point payload or silently clip those values.

Both policies use identical transport admission: scripted losses, waiting queue
capacity, drain-before-admit timing and horizon decisions. Only their numerical
use of available late information differs. Changing an unarrived payload must
not affect an earlier online prefix; adding an already sampled future waiter
can legitimately change queue occupancy, so it is not a valid equivalent test.

Follow the [issue](https://github.com/zblack54/fusion-lab/issues/22) for the reviewed
API, executable contract, commands and eventual implementation evidence. Actual
asynchronous channels, slow consumers and shutdown remain under
[parent #6](https://github.com/zblack54/fusion-lab/issues/6).

Exercise: with the same late velocity observation, predict the raw-velocity
integration baseline at tick1. It is [0.02, -0.02] m because it integrates the
measurement, whereas the filter integrates its posterior velocity. Explain why
neither correction permits rewriting the online tick0 snapshot.


## Current implementation progress

The first slice validates an integer history window from0 through500ticks and
returns `HistoryError::InvalidWindow` with the original request for larger values.
Five exact contract selectors passed on each installed alias, all seven gates
passed, and independent review confirmed only the constructor changed. Both local
aliases currently resolve to Rust1.99.0; the approved design also has hosted
Windows/Linux evidence on1.99.0 and1.99.1.

The correction engine remains under construction. Its full38selectors, resource
qualification, example and complete conformance review are required before
acceptance. Do not use the diagnostic replay boundary as completed functionality.
