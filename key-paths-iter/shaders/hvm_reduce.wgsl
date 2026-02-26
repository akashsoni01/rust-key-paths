// HVM2 Interaction Net Reduction Shader
// Struct layout must match Rust: NetNode (kind, port0, port1, port2), RedexPair (left, right).

struct NetNode {
    kind: u32,
    port0: u32,
    port1: u32,
    port2: u32,
}

struct RedexPair {
    left: u32,
    right: u32,
}

struct Metadata {
    node_count: u32,
    pair_count: u32,
    steps: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read_write> nodes: array<NetNode>;
@group(0) @binding(1) var<storage, read> pairs: array<RedexPair>;
@group(0) @binding(2) var<storage, read_write> metadata: Metadata;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;

    if (idx >= metadata.pair_count) {
        return;
    }

    let pair = pairs[idx];
    let left_idx = pair.left;
    let right_idx = pair.right;

    if (left_idx >= metadata.node_count || right_idx >= metadata.node_count) {
        return;
    }

    let left_node = nodes[left_idx];
    let right_node = nodes[right_idx];

    // HVM2 reduction rules (simplified)
    // Dup-Dup → annihilate (mark as Era)
    if (left_node.kind == 2u && right_node.kind == 2u) {
        nodes[left_idx].kind = 0u;
        nodes[right_idx].kind = 0u;
    }

    // Con-Era → annihilate
    if (left_node.kind == 1u && right_node.kind == 0u) {
        nodes[left_idx].kind = 0u;
    }
    if (left_node.kind == 0u && right_node.kind == 1u) {
        nodes[right_idx].kind = 0u;
    }

    // Step counter (single workgroup semantics; for multi-workgroup use atomic)
    metadata.steps += 1u;
}
