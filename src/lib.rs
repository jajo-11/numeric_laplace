extern crate rand;

mod thread_pool;
pub mod plot;

use rand::prelude::*;
use rand::ChaChaRng;
use std::fs::File;
use std::io::Write;

/// A simple struct for defining the elements in the model that have fixed potentials
///
/// # Fields
///
/// * `x` - coordinate of the top left corner of the box
/// * `y` - coordinate of the top left corner of the box
/// * `width` - width of the box
/// * `height` - height of the box
/// * `potential` - fixed potential across the box
#[derive(Debug)]
pub struct FixedBox {
    pub x: isize,
    pub y: isize,
    pub width: usize,
    pub height: usize,
    pub potential: f64,
}

/// Part of the Grid struct it contains info on the coordinate system the grid was defined with
/// mainly used to translate between internal and external coordinates
///
///  # Fields
///
/// * `nodes_per_unit` - grid resolution
/// * `x_offset` - offset to the x axis (internal coordinates are unsigned so this is required to
/// reflect the coordinates of the task.
/// * `y_offset` - offset to the y axis
/// * `invert_x`- if this is true the x axis will go from right to left (data is not filliped)
/// * `invert_y`- if this is true the y axis will go from top to bottom (data is not filliped)
pub struct Scale {
    pub nodes_per_unit: usize,
    pub x_offset: isize,
    pub y_offset: isize,
    pub invert_x: bool,
    pub invert_y: bool,
}

/// Storing the grid with all its nodes
///
///  # Fields
///
/// * `nodes` - the literal nodes of the Grid as an array
/// * `width` - each set of `width` elements of the `nodes` array form one row of nodes in the x
/// direction
/// * `dynamic_nodes_indices` - all the indices of nodes that can actually change. Each cycle of the
/// calculation will iterate of these.
/// * `scale` - info for plots
pub struct Grid<'s> {
    pub nodes: Vec<f64>,
    pub width: usize,
    dynamic_nodes_indices: Vec<usize>,
    pub scale: &'s Scale,
}

impl<'s> Grid<'s> {
    pub fn new(width: usize, height: usize, fixed_elements: &Vec<FixedBox>, scale: &'s Scale,
               seed: Option<[u8; 32]>) -> Grid<'s> {
        let width = width * scale.nodes_per_unit;
        let height= height * scale.nodes_per_unit;

        let mut nodes = vec![0.0; width*height];
        let mut  fixed_nodes_indices = Vec::with_capacity(width*height);

        // adding all indices of the border to the ´fixed_nodes_indices´ array
        let bottom_border_start_index = width * (height-1);
        for i in 0..width {
            //top border
            fixed_nodes_indices.push(i);
            //left border
            fixed_nodes_indices.push(i*width);
            //right border
            fixed_nodes_indices.push(i*width+width-1);
            //bottom border
            fixed_nodes_indices.push(bottom_border_start_index + i);
        }

        // adding all indices of the fixed boxes to the ´fixed_nodes_indices´ array
        for fixed_box in fixed_elements.iter() {
            //TODO check constraints
            let top_left_index = convert_coordinates(fixed_box.x, fixed_box.y, scale, width);
            for i in 0..fixed_box.height * scale.nodes_per_unit {
                for j in 0..fixed_box.width * scale.nodes_per_unit {
                    let index = top_left_index + j + i * width;
                    fixed_nodes_indices.push(index);
                    nodes[index] = fixed_box.potential;
                }
            }
        }

        //"inverting" the `fixed_nodes_indices` vector
        // this is the fastest method  i came up with if done wrong this will take a long time
        // (i.e. using .contains())
        fixed_nodes_indices.sort_unstable();
        fixed_nodes_indices.dedup();
        let mut dynamic_nodes_indices =
            Vec::with_capacity(nodes.len()-fixed_nodes_indices.len());
        let mut j = 0;
        for i in 0..nodes.len() {
            if i != fixed_nodes_indices[j] {
                dynamic_nodes_indices.push(i);
            } else { j += 1; }
        }

        //filling the nodes with noise
        let mut random = if let Some(seed) = seed {
            ChaChaRng::from_seed(seed)
        } else {
            ChaChaRng::from_entropy()
        };
        for &i in dynamic_nodes_indices.iter() {
            nodes[i] = random.gen_range(0.0, 100.0);
        }

        Grid { nodes, width, dynamic_nodes_indices, scale}
    }

    /// does the main work described in the task
    /// the function takes a ´accepted_delta´ which is used to determine when to stop iterating
    /// basically the function checks on every iteration what the biggest delta was and if it is
    /// below ´accepted_delta´ the functions returns
    pub fn evaluate(&mut self, accepted_delta: f64, over_relaxation: f64, watch: (isize, isize),
                    watch_data: &mut Vec<f64>) {
        let watch = convert_coordinates(watch.0, watch.1, self.scale, self.width);
        if self.nodes.len() <= watch { panic!("Watch is outside of the grid"); }
        watch_data.push(self.nodes[watch]);

        let mut max_delta= accepted_delta + 1.0;
        let mut iterations = 0;
        while max_delta > accepted_delta {
            iterations += 1;
            max_delta = 0.0;
            for &i in self.dynamic_nodes_indices.iter() {
                //summing up values to top, left, bottom and right and dividing by 4
                let mut new_value = self.nodes[i-self.width];
                new_value += self.nodes[i-1];
                new_value += self.nodes[i+1];
                new_value += self.nodes[i+self.width];
                new_value /= 4.0;

                let mut delta = self.nodes[i] - new_value;
                self.nodes[i] -= over_relaxation*delta;
                //checking if delta is new high
                if delta.abs() > max_delta {max_delta = delta.abs()};
            }
            watch_data.push(self.nodes[watch]);
            print!("\r{} iterations, max delta = {}", iterations, max_delta);
            std::io::stdout().flush().expect("Could not flush stdout!");
        }
        watch_data.push(std::f64::NEG_INFINITY);
        print!("\n");
    }

    // is the exact same as evaluate just with a fixed iteration count
    pub fn evaluate_for(&mut self, over_relaxation: f64, watch: (isize, isize),
                    watch_data: &mut Vec<f64>, iterations: usize) {
        let watch = convert_coordinates(watch.0, watch.1, self.scale, self.width);
        if self.nodes.len() <= watch { panic!("Watch is outside of the grid"); }
        watch_data.push(self.nodes[watch]);

        for _i in 0..iterations {
            for &j in self.dynamic_nodes_indices.iter() {
                //summing up values to top, left, bottom and right and dividing by 4
                let mut new_value = self.nodes[j-self.width];
                new_value += self.nodes[j-1];
                new_value += self.nodes[j+1];
                new_value += self.nodes[j+self.width];
                new_value /= 4.0;

                let mut delta = self.nodes[j] - new_value;
                self.nodes[j] -= over_relaxation*delta;
            }
            watch_data.push(self.nodes[watch]);
        }
        watch_data.push(std::f64::NEG_INFINITY);
    }

    /// does the exact same ting ´evaluate()´ does just on multiple threads at once
    /// the `threads´ argument takes the number of slave threads you want so the optimal number
    /// should be the number of threads your cpu supports - 1 (for the master thread)
    pub fn evaluate_multi_thread(&mut self, accepted_delta: f64, over_relaxation: f64,
                                 threads: usize) {
        let pool = thread_pool::ThreadPool::new(threads, self, over_relaxation);
        pool.evaluate(accepted_delta, self.dynamic_nodes_indices.len());
    }

    /// generates a csv file at the specified path containing the nodes
    /// these files then can be opened in a spread sheet program like excel for plotting.
    /// The 100 in the string literals is a filler
    /// the first row and column are filled with axis info
    pub fn to_csv(&self, path: &str) -> std::io::Result<()> {
        let mut csv = File::create(path)?;
        let mut file_string = String::with_capacity(self.nodes.len() * 20);

        //create x coordinate labels
        if self.scale.invert_x {
            file_string.push_str(
                &format!("100,{}", (self.width - 1) as f64 / self.scale.nodes_per_unit as f64
                    - self.scale.x_offset as f64));
            for x in 1..self.width {
                file_string.push_str(&format!(",{}", (self.width - x) as f64
                    / self.scale.nodes_per_unit as f64 - self.scale.x_offset as f64));
            }
        } else {
            file_string.push_str(
                &format!("100,{}", -self.scale.x_offset));
            for x in 1..self.width {
                file_string.push_str(&format!(",{}", x as f64 / self.scale.nodes_per_unit as
                    f64 - self.scale.x_offset as f64));
            }
        }

        file_string.push('\n');
        let height = self.nodes.len() / self.width;


        // write all the data of the grid to the file + y labels
        if self.scale.invert_y {
            for y in 0..height {
                file_string.push_str(&format!("{}", (height - y) as f64
                    / self.scale.nodes_per_unit as f64 - self.scale.y_offset as f64));
                for x in 0..self.width {
                    file_string.push_str(&format!(",{}", self.nodes[y*self.width+x]));
                }
                file_string.push('\n');
            }
        } else {
            for y in 0..height {
                file_string.push_str(&format!("{}", y as f64 / self.scale.nodes_per_unit as
                    f64 - self.scale.y_offset as f64));
                for x in 0..self.width {
                    file_string.push_str(&format!(",{}", self.nodes[y*self.width+x]));
                }
                file_string.push('\n');
            }
        }
        csv.write_all(file_string.as_bytes())?;
        Ok(())
    }
}

/// Takes coordinates and transforms them to the index representing the point in the grid
// TODO check for errors (out of bounds)
fn convert_coordinates(x: isize, y: isize, scale: &Scale, width: usize) -> usize {
    let x = if scale.invert_x { (-x + scale.x_offset) as usize * scale.nodes_per_unit }
        else { (x + scale.x_offset) as usize * scale.nodes_per_unit };
    let y = if scale.invert_y { (-y + scale.y_offset) as usize * scale.nodes_per_unit }
        else { (y + scale.y_offset) as usize * scale.nodes_per_unit };
    x+y*width
}

/// Literally what the name says; used to plot the watch data
pub fn watch_data_to_csv(headers: &[String], watch_data: Vec<f64>, path: &str)
    -> std::io::Result<()> {
    let mut csv = File::create(path)?;
    let mut file_string = String::with_capacity(watch_data.len() * 20);

    let mut headers = headers.iter();
    file_string.push_str(headers.next().expect("No headers supplied").as_str());
    for &i in (&watch_data)[0..watch_data.len() - 1].iter() {
        if i == std::f64::NEG_INFINITY {
            file_string.push('\n');
            if let Some(header) = headers.next() {
                file_string.push_str(header.as_str());
            }
        } else {
            file_string.push_str(&format!(",{}", i));
        }
    }
    csv.write_all(file_string.as_bytes())?;
    Ok(())
}