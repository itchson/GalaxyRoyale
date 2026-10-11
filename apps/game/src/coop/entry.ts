/** Save barriers cannot resurrect a cancelled invitation or a departed screen. */
export async function enterAfterSave(
  save: () => Promise<void>,
  flush: () => Promise<void>,
  current: () => boolean,
  enter: () => Promise<void>,
): Promise<void> {
  const check = () => {
    if (!current()) throw new Error("Co-op request cancelled.");
  };
  check();
  await save();
  check();
  await flush();
  check();
  await enter();
}
