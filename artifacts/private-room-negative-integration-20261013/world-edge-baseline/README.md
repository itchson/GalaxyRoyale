# Actual integrated-baseline world-edge failure

Root independently ran the complete native capture regression in isolated codex/world-edge-contract from 58bb2ec (runtime unchanged from 332ddd5). No generated entity was removed. The exact targeted cargo test exited 101: zero passed, one failed; the first positive-X edge rejected actual generated geometry as invalid_entity. This is a reproduced baseline defect, not a fix or passing release test. Remaining edges in the loop were not reached after the first failure. No outbound projectile failure or live RTC result is inferred.
