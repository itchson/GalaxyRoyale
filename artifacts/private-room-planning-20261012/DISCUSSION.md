# Actual asynchronous implementation discussion

Engine and Browser coordinated private approval gating while preserving existing pending→RTC-ready participant states. Host receives join-request(peer,expires_at); approval atomically yields peer-joined(peer) to host and join-approved to guest. Explicit approval is required for SDP, ICE and heartbeat readiness. The bounded initial wait is20seconds for approval plus20seconds for negotiation.

QA inspected d2acfd2 and identified baseline automatic-pending negotiation/native admission as the gap being addressed. QA also identified a real design hazard: an older same-room heartbeat/poll/admin reply can roll back joining_locked, and dropping an old rotation reply can leave an obsolete code if a newer lock reply contains no code. Engine/Browser agreed to return room_revision+room_code+joining_locked as one complete policy and adopt newer revisions coherently. Root accepted the coordinated contract and requested meaningful reorder/failed-preflight tests.

These are actual written contributions received from the three dispatched agents. No live meeting, new candidate test pass or implemented field is claimed by this record. Source candidates and final review remain pending.
