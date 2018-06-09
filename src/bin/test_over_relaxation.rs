extern crate numeric_laplace;

use numeric_laplace::*;

fn main() {
    let scale = Scale {
        // don't overdo these 1 or 2 is reasonable 10 is noticeably slower but still feasible
        // you might need lower deltas to generate usable plots though
        nodes_per_unit: 5,
        x_offset: 10,
        y_offset: 25,
        invert_x: false,
        invert_y: true,
    };

    let yellow = FixedBox { x: -5, y: 5, width: 10, height: 10, potential: 100.0 };
    let blue_top = FixedBox { x: 30, y: 20, width: 5, height: 18, potential: 0.0 };
    let blue_bottom = FixedBox { x: 30, y: -2, width: 5, height: 18, potential: 0.0 };

    let fixed_boxes = vec![yellow, blue_top, blue_bottom];
    let mut watch_data = Vec::with_capacity(201*9);
    let mut watch_headers = Vec::with_capacity(9);

    let mut grid;
    let mut over_relaxation = 1.9;
    // just an arbitrary value to generate the same grid for each over relaxation value to make the
    // runs actually comparable
    let seed = [4; 32];
    let mut i = 0;
    // evaluating the grid with different values for ´over_relaxation´
    loop {
        watch_headers.push(format!("{}", over_relaxation));
        grid = Grid::new(50, 50, &fixed_boxes, &scale, Some(seed));
        grid.evaluate_for(over_relaxation, (32, 0), &mut watch_data, 200);
        over_relaxation += 0.01;
        i += 1;
        if i >= 10 {break;}
    }

    watch_data_to_csv(&watch_headers, watch_data, "watch.csv")
        .expect("Could not write watch file!");
    plot::plot_lines_by_column("watch.csv", "Potential At (32, 0) Over Iterations",
                               1.9, 0.01);
}
