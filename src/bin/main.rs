extern crate numeric_laplace;

use numeric_laplace::*;

fn main() {
    let scale = Scale {
        // don't overdo these 1 or 2 is reasonable 10 is still fast
        // but needs lower deltas to function properly
        nodes_per_unit: 10,
        lowest: 0.0,
        highest: 100.0,
        x_offset: 10,
        y_offset: 25,
        invert_x: false,
        invert_y: true,
    };
    let yellow = FixedBox::new(Point::new(-5, 5, &scale), 10, 10, 100.0, &scale);
    let blue_top = FixedBox::new(Point::new(30, 20, &scale), 5, 18, 0.0, &scale);
    let blue_bottom = FixedBox::new(Point::new(30, -2, &scale), 5, 18, 0.0, &scale);

    let mut grid = Grid::new(50, 50, vec![yellow, blue_top, blue_bottom],
    scale);

    let watch_point = Point::new(32, 0, &grid.scale);
    let (iterations, watch_data) = grid.evaluate(0.01, 1.8, watch_point);
    println!("Done in {} iterations", iterations);
    grid.to_csv("out.csv").expect("Could not write grid file!");
    watch_data_to_csv(watch_data, "watch.csv").expect("Could not write watch file!");
}
