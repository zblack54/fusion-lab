# Planar EKF design preflight (#24)

Draft following the wheel-motion model in #23. The filter sees signed wheel observations and independent position fixes. The simulator and evaluator own truth. The first state is (x, y, heading) in metres and radians, with explicit initial pose and 3×3 covariance. Bias, slip estimation, asynchronous replay and UI integration are outside the first slice.

## Prediction candidate

Use the wheel pair as controls. Let s=(r+l)/2, a=(r-l)/(2b), g(a)=sin(a)/a, φ=heading+a. Advance by s g(a) [cos φ, sin φ], with heading increment 2a. Reuse the accepted mean geometry where practical. Use the unwrapped local heading for derivatives, normalize the mean afterward, and compute small-angle limits without subtractive cancellation.

For F = ∂f/∂state, the heading column of the position block is s g(a) [−sin φ, cos φ]. The rest is the identity. For each wheel j, ∂s/∂j=1/2 and ∂a/∂j=±1/(2b). With u=[cos φ,sin φ] and v=[−sin φ,cos φ], its position derivative is (g/2)u + s (∂a/∂j)(g' u + g v), and its heading derivative is 2∂a/∂j. At a=0, g=1 and g'=0. Use F P Fᵀ + G R_w Gᵀ with wheel travel variances in m². Any additional process noise requires an explicit physical model.

## Position update candidate

Measure x,y with H selecting the position block. Solve the innovation system with Cholesky and use the Joseph covariance update. Validate finite, symmetric positive covariance at the public boundary. Define tolerances, covariance domain, and exact failure behavior in the contract. On error, preserve the previous state and covariance. Include tests for nonpositive inputs and atomic failure. A position fix does not directly observe heading. Motion can couple heading to observed position through the cross-covariance, while stationary fixes alone cannot recover initial heading.

## Verification plan

Derive reference predictions for straight, pivot, arc, reverse and near-zero turns. Check both F and G against independently implemented central differences away from the angle branch cut. At a wrap, compare angular perturbations on the circle. Add exact straight-limit derivatives and a hand-computed linear position update. Verify symmetric positive covariance over a curved trajectory and preserve state on failed prediction/update. Freeze tolerances, units, API and data ownership before implementation.

For evaluation, compare on curved and straight held-out trajectories. Give the equal-information comparator the same wheel observations and position fixes, initial pose and scheduled updates. A wheel-only dead-reckoning baseline is an information ablation. Include weak-observability and matched-simple-motion cases where an EKF may add little. Report position and wrapped heading error plus consistency across seeds. Prespecify seeds, metrics and tuning boundaries before collecting held-out results.

## Independent review and next decisions

For h=0, b=2, l=r=2, the independent oracle gives F=[[1,0,0],[0,1,2],[0,0,1]] and G=[[0.5,0.5],[-0.5,0.5],[-0.5,0.5]]. With P=diag(1,2,3) and Rw=diag(1,4), the predicted covariance is [[2.25,0.75,0.75],[0.75,15.25,7.25],[0.75,7.25,4.25]]. Unequal wheel variances distinguish the two columns. At small a use g'(a)=-a/3+a³/30-a⁵/840 plus higher-order terms. Freeze the branch threshold and tolerance before implementation.

Wheel travel variance is per interval. Allow exact zero wheel variance, but require positive-definite initial and position-fix covariance. Specify a symmetric representation, finite bounds, and error ordering. No silent jitter. The prediction domain must explicitly address noisy travel beyond the ideal input bound from #23. The equal-information comparator must use identical wheel and position inputs, schedule, initialization and no future data. Review the exact comparator and covariance interpretation before locking an executable contract.
