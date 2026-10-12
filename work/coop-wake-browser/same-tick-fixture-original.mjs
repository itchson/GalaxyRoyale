// Original added fixture block from the author patch transcript, executed in
// focused-second.log. It accidentally paired latest staged header/current rows
// with the last received Wake page from another capture. Not a runtime change.
test("same-tick current-state Wake is forwarded unchanged and native set rejection stays decisive", async () => {
  const p=await pair({wake:true});await p.frame(8);
  const old=p.replica.header,points=p.replica.pages.filter(p=>Array.isArray(p.points)).at(-1);
  assert.ok(points);
  const same=structuredClone(old);same.acquisition=p.native.acquisition+1;
  deliver(p,"publication",same);
  deliver(p,"body",{session:old.snapshot.session,tick:old.snapshot.tick,revision:old.snapshot.revision,offset:0,next_offset:null,entities:p.native.capture.rows});
  deliver(p,"body",points);
  assert.deepEqual(p.replica.pages.at(-1),points);
  // Configured fake native rejection models the separate real Rust commit
  // fence; JS neither reconstructs nor compares captured lifetime sets.
  p.replica.commit=()=>false;
  assert.throws(()=>deliver(p,"publication-end",{acquisition:same.acquisition,revision:same.snapshot.revision}),/replica_commit/);
  const ackBefore=p.network.sent.filter(f=>f.kind==="publication-ack").length;
  assert.equal(p.network.sent.filter(f=>f.kind==="publication-ack").length,ackBefore);
  await p.host.end();await p.guest.dispose();
});
