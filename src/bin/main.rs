extern crate numeric_laplace;

use numeric_laplace::*;

fn main() {
    // creating ´Scale´ Object with resolution (´nodes_per_unit´) and info about the coordinate
    // system
    let scale = Scale {
        // don't overdo these 1 or 2 is reasonable 10 is noticeably slower but still feasible
        // you might need lower deltas to generate usable plots though
        nodes_per_unit: 5,
        x_offset: 10,
        y_offset: 25,
        invert_x: false,
        invert_y: true,
    };

    // basically describing the initial situation
    let yellow = FixedBox { x: -5, y: 5, width: 10, height: 10, potential: 100.0 };
    let blue_top = FixedBox { x: 30, y: 20, width: 5, height: 18, potential: 0.0 };
    let blue_bottom = FixedBox { x: 30, y: -2, width: 5, height: 18, potential: 0.0 };
    let fixed_boxes = vec![yellow, blue_top, blue_bottom];
    let mut watch_data = Vec::with_capacity(2000);

    // creating the underlying data structure of the grid using the info above
    let mut grid = Grid::new(50, 50, &fixed_boxes, &scale, None);

    // do the iterating over the grid use
    // grid.evaluate_multi_thread(0.001, 1.8, 15);
    // for higher ´nodes_per_unit´ values (like 20) or just faster evaluation in general
    grid.evaluate(0.001, 1.8, (32, 0), &mut watch_data);

    // plotting of the data
    grid.to_csv("out.csv").expect("Could not write grid file!");
    plot::plot_2d_color_map("out.csv");

    // this plots the values at the "watch" point but as it takes all iterations the resulting graph
    // is not all that useful run "test_over_relaxation" instead
    // (the number of iterations is limited there)
    watch_data_to_csv(&[String::from("run1")], watch_data, "watch.csv")
        .expect("Could not write watch file!");
    plot::plot_lines_by_column("watch.csv", "Potential At (32, 0) Over Iterations", 1.8, 0.0);
}
