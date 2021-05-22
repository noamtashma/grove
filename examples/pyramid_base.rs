use itertools::Itertools;
use orchard::*;

use std::str::FromStr;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Error, ErrorKind, Read};
use std::time::Instant;

use trees::treap::*;
use trees::splay::*;
use trees::avl::*;
use example_data::SizedSummary;

type I = i32;

///////////////// obstacle and edge ///////////////////
// these are types that represent the obstacles and the obstacle's edges

/// One of the obstacles.
/// Represents the rectangle [xlow..xhigh) x [ylow..yhigh) with half-open 
/// ranges, as should be.
#[derive(PartialEq, Eq, Clone, Debug)]
struct Obstacle {
    xlow : I,
    xhigh : I,
    ylow : I,
    yhigh : I,
    cost : I,
}

impl Obstacle {
    fn edges(&self) -> (Edge, Edge) {
        (Edge {
            is_opening : true,
            xlow : self.xlow,
            xhigh : self.xhigh,
            y : self.ylow,
            cost : self.cost,
        },
        Edge {
            is_opening : false,
            xlow : self.xlow,
            xhigh : self.xhigh,
            y : self.yhigh,
            cost : self.cost,
        })
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct Edge {
    /// tells if this edge is an opening edge or a closing edge
    is_opening : bool,
    xlow : I,
    xhigh : I,
    y : I,
    cost : I,
}

impl Edge {
    // enlarges the obstacle `d` units to the negative directions.
    // might get out of bounds. 
    fn enlarge(&self, d : I) -> Edge {
        Edge {
            xlow : self.xlow - d,
            y : if self.is_opening { self.y - d } else { self.y },
            ..*self
        }
    }
}

impl PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Edge {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // it is only important for `y` to be compared first.
        // the rest doesn't matter, really.
        self.y.cmp(&other.y).then(
            (!self.is_opening).cmp(&!other.is_opening)
        ).then(
            self.xlow.cmp(&other.xlow)
        ).then(
            self.xhigh.cmp(&other.xhigh)
        ).then(
            self.cost.cmp(&other.cost)
        )
    }
}


//////////////////////////////// implementing the tree data //////////////////////////////
// here we implement the data that will sit in the tree: the values (Segment), summaries (SizeMinSummary)
// and the actions (AddAction).

// a segment of constant values
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
struct Segment {
    size : usize,
    val : I,
}

impl Segment {
    fn split_at_index(self, i : usize) -> (Segment, Segment) {
        (Segment {
            size : i, val : self.val, 
        }, Segment {
            size : self.size - i, val : self.val,
        })
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
struct SizeMinSummary {
    size : usize,
    min : Option<I>,
}

impl std::ops::Add for SizeMinSummary {
    type Output = Self;
    fn add(self, other : Self) -> Self {
        SizeMinSummary {
            min : match (self.min, other.min) {
                (Some(a), Some(b)) => Some(std::cmp::min(a,b)),
                (Some(a), _) => Some(a),
                (_, b) => b,
            },
            size : self.size + other.size,
        }
    }
}

impl std::default::Default for SizeMinSummary {
    fn default() -> Self {
        SizeMinSummary {
            size : 0,
            min : None,
        }
    }
}

impl SizedSummary for SizeMinSummary {
    fn size(self) -> usize {
        self.size
    }
}


#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct AddAction{
    pub add : I,
}

impl std::ops::Add for AddAction {
    type Output = Self;
    fn add(self, other : Self) -> Self {
        AddAction {
            add : self.add + other.add,
        }
    }
}

impl Default for AddAction {
    fn default() -> Self {
        AddAction { add : 0 }
    }
}

impl Action for AddAction {
    fn is_identity(self) -> bool
    {
		self == Default::default()
	}
}

impl Acts<SizeMinSummary> for AddAction {
    fn act_inplace(&self, summary : &mut SizeMinSummary) {
        summary.min = summary.min.map(|max : I| { max + self.add });
    }
}

impl Acts<Segment> for AddAction {
    fn act_inplace(&self, object : &mut Segment) {
        object.val += self.add;
    }
}


#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
struct MyData {}

impl Data for MyData {
    type Value = Segment;

    type Summary = SizeMinSummary;

    type Action = AddAction;

    fn to_summary(val : &Self::Value) -> Self::Summary {
        SizeMinSummary {
            size : val.size,
            min : if val.size == 0 { None } else { Some(val.val) },
        }
    }
}


//////////////////////////////////////////// algorithm //////////////////////////////////////////
// here the actual algorithms start

// splits a segment inside the tree
fn search_split<TR : ModifiableTreeRef<MyData>>(tree : TR, index : usize)
{
    let mut walker = tree.walker();
    // using an empty range so that we'll only end up at a node
    // if we actually need to split that node
    methods::search_subtree(&mut walker, index..index); 
    
    let left = walker.left_summary().size;
    let v2option = walker.with_value( |val| {
        let (v1, v2) = val.split_at_index(index - left);
        *val = v1;
        v2
    });

    if let Some(v2) = v2option {
        methods::next_empty(&mut walker).unwrap(); // not at an empty position
        walker.insert(v2).unwrap();
    }
}


/// solves the pyramid_base problem
fn solve(m : usize, n : usize, budget : I, obstacles : Vec<Obstacle>) -> usize {
    let (mut opening_edges, mut closing_edges) : (Vec<Edge>, Vec<Edge>) 
        = obstacles.iter().map(|x| x.edges()).unzip();

    opening_edges.sort_unstable();
    closing_edges.sort_unstable();

    // binary search the size of the pyramid, `k`
    let mut lo = 0;
    let mut hi = std::cmp::min(n, m);
    
    while hi > lo {
        let k = (lo + hi + 1) / 2;

        // this is the specific size, because squares with a large `x` coordinate
        // correspond after enlarging to pyramids that go out of the positive boundary
        let mut tree : Treap<MyData>
            = vec![Segment { size : m + 1 - k , val : 0 }].into_iter().collect();

        // the list of updates we do to our tree as we scan it from a lower `y` coordinate
        // to a higher `y` coordinate
        let event_iter 
            = opening_edges.iter().map(|e| e.enlarge((k-1) as I)).merge(
                closing_edges.iter().map(|e| e.enlarge((k-1) as I))
            );
        
        // prev_y starts at `0` instead of `I::MIN`, in order that we don't consider any squares
        // with negative `y` coordinate, which are out of bounds.
        let mut prev_y = 0;
        let mut possible : bool = false;
        // scan from low `y` coordinate to high `y` coordinate
        for edge in event_iter {
            // only check the values of the squares after all of the updates on the `y`
            // coordinate of `prev_y` have been applied, and `edge.y` is positive.
            if edge.y > prev_y {
                let temp_min = tree.subtree_summary().min.unwrap();
                if temp_min <= budget {
                    possible = true;
                    break;
                }
                // update is out of bounds: squares with a large `y` coordinate
                // correspond after enlarging to pyramids that go out of the positive boundary
                if edge.y >= n as I - k as I + 1 {
                    break;
                }
                prev_y = edge.y;
            }

            // clamping to zero if the `x` coordinate is negative
            let xlow = std::cmp::max(edge.xlow, 0) as usize;
            if edge.is_opening {
                search_split(&mut tree, xlow);
                search_split(&mut tree, edge.xhigh as usize);
            }
            
            // note that acting on edges that got out of bounds still works as it should,
            // adding the cost only on the part that is in bounds
            tree.slice(xlow..edge.xhigh as usize).act(AddAction {
                add : if edge.is_opening { edge.cost } else { - edge.cost }
            });

            // TODO: if there is a closing edge, try to reunite the segments.
        }
        
        // check one last time, because the squares with the highest possible `y` coordinate
        // will not get checked unless there is an `edge` which has a `y` coordinate which is out of bounds.
        let temp_min = tree.subtree_summary().min.unwrap();
        if temp_min <= budget {
            possible = true;
        }

        if possible {
            lo = k;
        } else {
            hi = k-1;
        }
    }

    lo
}

///////////////////////////////////////////// input handling ////////////////////////////////////////////

// runs the solution on a given input
fn run_from<R: Read>(io: R) -> usize {
    let br = BufReader::new(io);
    
    let mut words_iter = br.lines()
        .flat_map(|row|
            row.unwrap()
            .split_whitespace()
            .map(|x : &str| x.parse().unwrap())
            .collect::<Vec<_>>()
        );

    let m = words_iter.next().unwrap();
    let n = words_iter.next().unwrap();
    let budget = words_iter.next().unwrap();
    let p = words_iter.next().unwrap();
    print!(" p={: <6}", p);
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    if p > 17_000 { // skip the third test case
        return 0;
    }
    let mut obstacles = vec![];
    for _ in 0..p {
        let e = "format not met";
        let obstacle = Obstacle {
            xlow : words_iter.next().expect(e) - 1,
            ylow : words_iter.next().expect(e) - 1,
            xhigh : words_iter.next().expect(e),
            yhigh : words_iter.next().expect(e),
            cost : words_iter.next().expect(e),
        };
        obstacles.push(obstacle);
    }
    
    let start = Instant::now();
    let res = solve(m as usize, n as usize, budget, obstacles);
    let duration = Instant::now().duration_since(start);
    print!(" {: <12} ", format!("{:?}", duration));
    res
}

// run the tests in the test directory.
// you need to manually put the tests in the folder.
fn check_all_tests() -> Result<(), Error> {
    let current_dir = std::path::PathBuf::from_str("../orchard/pyramid_base_test_files").unwrap();
    println!(
        "Testing files from {:?}:",
        current_dir
    );

    for entry in fs::read_dir(current_dir.clone())? {
        let entry = entry?;
        let path = entry.path();

        let (name, filetype) = if let Some(pair) =
            path
            .file_name()
            .and_then(|x| x.to_str())
            .and_then(|x| x.split_once('.'))
        {
            pair
        } else { continue };

        if filetype != "in" {
            continue;
        }
        run_on_file(name)?;
    }

    Ok(())
}

// run the solution of a specific file,
// and print some metadata
fn run_on_file(name : &str) -> Result<(), Error> {
    let current_dir = std::path::PathBuf::from_str("../orchard/pyramid_base_test_files").unwrap();
    print!("testing {: >8}.in:", name);
    let mut file_path = current_dir.clone();
    file_path.push(format!("{}.in", name));
    std::io::Write::flush(&mut std::io::stdout())?;

    let computed_res = run_from(File::open(file_path)?);
    print!("{: >7}: ", computed_res);

    let mut file_path = current_dir.clone();
    file_path.push(format!("{}.out", name));
    let mut content = String::from("");
    File::open(file_path)?.read_to_string(&mut content)?;
    let real_res : usize = content.trim().parse()
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    if computed_res == real_res {
        println!("Ok!");
    } else {
        println!("Failed!");
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    check_all_tests()?;
    //let res = run_from(File::open("../orchard/pyramid_base_test_files/pbs10a.in")?);
    //let res = run_from(std::io::stdin());
    //println!("{}", res);
    Ok(())
}