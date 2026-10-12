// Actual source SHA256 744bd717bde86b8d199f31c3f4f639ee6dfc6a8792abc79200852595eb3a66b3
/** A room lookup code, never a participant/admin credential. */
export const MAX_ROOM_CODE_LENGTH = 64;
export function normalizeRoomCode(value) {
    if (typeof value !== "string" || value.length > MAX_ROOM_CODE_LENGTH)
        return null;
    // Deliberately ASCII-only: trim() would silently accept NBSP and BOM.
    const compact = value.replace(/[\t\n\v\f\r -]/g, "");
    if (!/^#?[0-9a-f]{8}$/i.test(compact))
        return null;
    const hex = compact.replace(/^#/, "").toUpperCase();
    return `#${hex.slice(0, 4)}-${hex.slice(4)}`;
}
