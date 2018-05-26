extern crate rand;

use rand::prelude::*;
use rand::ChaChaRng;
use std::fs::File;
use std::io::Write;

#[derive(Debug)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

/// A simple struct for defining the elements in the model that have fixed potentials
///
/// # Fields
///
/// * `point` - coordinates of the top left corner of the box
/// * `width` - width of the box
/// * `height` - height of the box
/// * `potential` - fixed potential across the box
#[derive(Debug)]
pub struct FixedBox {
    pub point: Point,
    pub width: usize,
    pub height: usize,
    pub potential: f64,
}

impl FixedBox {
    /// adds the box in form of fixed nodes to the provided `nodes` and `fixed_nodes_indices` vectors
    pub fn gen_fixed_box(
        self,
        width: usize,
        height: usize,
        nodes: &mut Vec<f64>,
        fixed_nodes_indices: &mut Vec<usize>, ) {
        //check constraints
        if self.point.x >  width-1
            || self.point.y > height-1
            || self.point.x +  self.width > width-1
            || self.point.y + self.height > height-1 {
            panic!("A fixed box lays outside of the grid.\n{:#?}", self)
        }

        for i in 0..self.height {
            for j in 0..self.width {
                let index = width*(i+self.point.y)+j+self.point.x;
                fixed_nodes_indices.push(index);
                nodes[index] = self.potential;
            }
        }
    }
}

/// Part of the Grid struct it contains info for plotting
///
///  # Fields
///
/// * `nodes_per_unit` - grid resolution
/// * `lowest`- lowest fixed potential (for color scale)
/// * `highest` - highest fixed potential (for color scale)
/// * `x_offset` - offset to the x axis (internal coordinates are unsigned so this is required to
/// reflect the coordinates of the task.
/// * `y_offset` - offset to the y axis
/// * `invert_x`- if this is true the x axis will go from right to left (data is not filliped)
/// * `invert_y`- if this is true the y axis will go from top to bottom (data is not filliped)
pub struct Scale {
    pub nodes_per_unit: usize,
    pub lowest: f64,
    pub highest: f64,
    pub x_offset: usize,
    pub y_offset: usize,
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
pub struct Grid {
    pub nodes: Vec<f64>,
    pub width: usize,
    dynamic_nodes_indices:Vec<usize>,
    scale: Scale,
}

impl Grid {
    pub fn new(width: usize, height: usize, fixed_elements: Vec<FixedBox>,
               scale: Scale) -> Grid {
        let width = width * scale.nodes_per_unit;
        let height= height * scale.nodes_per_unit;
        let mut nodes: Vec<f64> = Vec::with_capacity(width*height);
        let mut  fixed_nodes_indices: Vec<usize> = Vec::new();
        gen_fixed_borders(width, height, &mut nodes, &mut fixed_nodes_indices);
        fixed_elements.into_iter().for_each(|x| {
            x.gen_fixed_box(width, height, &mut nodes, &mut fixed_nodes_indices);
        });
        //"inverting" the `fixed_nodes_indices` vector
        let dynamic_nodes_indices: Vec<usize> = (0..width*height).filter(|x| {
            !fixed_nodes_indices.contains(x)
        }).collect();

        //filling the nodes with noise
        let mut random = ChaChaRng::from_entropy();
        for &i in dynamic_nodes_indices.iter() {
            nodes[i] = random.gen_range(0.0, 100.0);
        }

        Grid {
            nodes,
            width,
            dynamic_nodes_indices,
            scale,
        }
    }

    pub fn evaluate(&mut self, accepted_delta: f64, over_relaxation: f64, watch: Point)
        -> (usize, Vec<f64>) {
        let watch = watch.y * self.width + watch.x;
        if self.nodes.len() <= watch {panic!("Watch is outside of the grid");}
        let mut watch_data = Vec::with_capacity(200);
        watch_data.push(self.nodes[watch]);

        let mut max_delta= accepted_delta + 1.0;
        let mut iterations = 0;
        while max_delta > accepted_delta {
            iterations += 1;
            println!("{} iterations, max delta = {}", iterations, max_delta);
            max_delta = 0.0;
            for &i in self.dynamic_nodes_indices.iter() {
                let top = self.nodes[i-self.width];
                let right = self.nodes[i-1];
                let left = self.nodes[i+1];
                let bottom = self.nodes[i+self.width];

                let new_value = (top +  right + left + bottom) / 4.0;

                let mut delta = (self.nodes[i] - new_value);
                self.nodes[i] = self.nodes[i] - over_relaxation*delta;
                if delta.abs() > max_delta {max_delta = delta.abs()};
            }
            watch_data.push(self.nodes[watch]);
        }
        (iterations, watch_data)
    }


    /// generates a csv file at the specified path containing the nodes
    /// these files then can be opened in a spread sheet for plotting.
    pub fn to_csv(&self, path: &str) -> std::io::Result<()> {
        let mut csv = File::create(path)?;
        let mut file_string = String::with_capacity(self.nodes.len() * 20);
        file_string.push_str(&format!("{}, ", self.nodes[0]));
        self.nodes.iter().enumerate().skip(1).for_each(|(i, n)| {
            if i%self.width == 0 {
                //avoid adding a ', ' at end of line (out of vanity)
                file_string.pop();
                file_string.pop();
                file_string.push('\n');
            }
            file_string.push_str(&format!("{}, ", n));
        });
        csv.write_all(file_string.as_bytes())?;
        Ok(())
    }
}

fn gen_fixed_borders(
    width: usize,
    height: usize,
    nodes: &mut Vec<f64>,
    fixed_nodes_indices: &mut Vec<usize>) {
    nodes.resize(width*height, 0.0);
    (0..width).for_each(|x| {
        //top border
        fixed_nodes_indices.push(x);
        //left border
        fixed_nodes_indices.push(x*width);
        //right border
        fixed_nodes_indices.push((x+1)*width-1);
    });
    //bottom border
    let capacity = width*height;
    ((capacity-width)..capacity).for_each(|x| fixed_nodes_indices.push(x));
    //setting every point of the border to a potential of 0
    fixed_nodes_indices.iter().for_each(|&x| nodes[x] = 0.0);
}

pub fn watch_data_to_csv(watch_data: Vec<f64>, path: &str) -> std::io::Result<()> {
    let mut csv = File::create(path)?;
    let mut file_string = String::with_capacity(watch_data.len() * 20);
    file_string.push_str(&format!("{}", watch_data[0]));
    watch_data.iter().skip(1).for_each(|i| {
        file_string.push_str(&format!(", {}", i));
    });
    csv.write_all(file_string.as_bytes())?;
    Ok(())
}