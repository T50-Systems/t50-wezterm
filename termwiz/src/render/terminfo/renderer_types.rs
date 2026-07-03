pub struct TerminfoRenderer {
    caps: Capabilities,
    current_attr: CellAttributes,
    pending_attr: Option<CellAttributes>,
    /* TODO: we should record cursor position, shape and color here
     * so that we can optimize updating them on screen. */
}
