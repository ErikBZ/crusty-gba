pub fn calc_cycles_for_stm_ldm(cycles_per_entry: u32, entries: u32, load: bool, is_pc: bool) -> u32 {
    // TODO: Double check this. It may be wrong, but assuming it's right for now
    if load {
        if is_pc {
            // NOTE (n+1)S + 2N + 1I when PC is in register_list
            (cycles_per_entry * entries) + 4
        } else {
            // NOTE nS + 1N + 1I
            (cycles_per_entry * entries) + 2
        }
    } else {
        // NOTE: (n-1)S + 2N
        (cycles_per_entry * entries) + 1
    }
}
