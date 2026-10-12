# Controlled QA room-start refusal

The actual old ba70 child command/message functions returned ok:true with invite:null when a declared controller boundary had resolved a refused start with signaling_limit. Root retained the failing source-function assertion and old source. This is not a raw HTTP429 or live reproduction of the earlier failed lock case.

At ec15de3abdbbf0a64c3f9f035636e65364469483 the QA create command refuses missing invitations, preserves only the controlled signaling_limit enum or generic room_create_failed, and propagates those through the child, parent and strict report allowlists. Unknown error text stays excluded; successful private grant fields are unchanged. Normal game, service, native and package sources are unchanged.

Actual source-function checks: 19 negative/refusal cases,10 existing fifth-shot cases and16 existing WASM helper cases pass;2 supplied-artifact HTTP cases were excluded. Independent review and a new matching actual-browser refusal are pending. Historical21live captures keep their old source pins and generic failed result.
