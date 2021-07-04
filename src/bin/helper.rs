use chips::{ConwaysLife, MooreNeighborhood};

fn main() {
    println!("[");
    for neighborhood in 0..512u16 {
        fn print_dead() {
            println!("    false,");
        }

        fn print_alive() {
            println!("    true,");
        }

        let hood: MooreNeighborhood = ((neighborhood & 0b0000000011111111) as u8).into();
        let status = (neighborhood & 1 << 8) != 0;

        let neighbors: [bool; 8] = hood.into();
        let alive_neighbors = neighbors.iter().filter(|&i| *i).count();

        if ConwaysLife::simulate_with_logic(status, hood) {
            print_alive();
        } else {
            print_dead();
        }
    }
    println!("]")
}
