# Actual asynchronous shader review

Engine specialist read-only review received12October2026 against Root1eca1c510325788624851a86527c2810b5e42df7. No extra tests or shader execution by this reviewer. No concrete blocker for the mapped PAN pipeline.

The composed columns and sign(det)/sqrt(abs(det)) adjugate form match the portable inverse transpose of diag(A,sqrt(abs(det))) after normalization. Basis-magnitude normalization removes an absolute tiny-quad floor; reflection/shear signs and full-sphere oct8/sign-not-zero decoding match Surface. Low-seven slot masking is correct. Legacy mask1 preserves emission; signed soft-emission decoding precedes the linear byte multiplier. Height uses the spare light component at0.35*radius; camera-relative cell/light coordinates agree.

Limit: shader f32 and relative determinant fallback1e-12 are narrower than the portable f64 extreme-basis oracle. Uniform tiny scales do not trigger that threshold; the registered starter basis is well-conditioned.

Exact Git blob pins: pixel.wgsl f907a21b860499e03c5eef11ff7d4ee33335ea20; lighting.rs3679cb1fb935dabfd83b21158871b74c5fdee15d; renderer.rs cfefe452004b5ff90eeaf102c567b89705dc3552; surface.rs80f6bff0f2b6cc1fac70d958d1c27981fea6fe23. Reviewer confirmed working bytes matched committed blobs.
