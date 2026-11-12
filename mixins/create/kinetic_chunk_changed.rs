// SPDX-License-Identifier: MIT
// The pinned engine emits these server lifecycle notifications. Do not read
// unloaded chunks: their block entities can still be present at this point.
fn kinetic_chunk_changed() {
    crate::game::create_port_kinetic_runtime::chunks_changed();
}
