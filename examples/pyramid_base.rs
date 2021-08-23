//! This is an example solution to the pyramid base question from IOI 2008.
//!
//! In order to run pyramid_base, you will need to download the pyramid
//! base test files from [here], and save
//! them in a new folder named "pyramid_base_test_files", in the package's directory.
//!
//! For each tree type (currently treap, splay, and avl) the code will so this:
//! * It will look for the test files in "package/pyramid_base_test_files/".
//! * It will sort them by difficulty based on their name. run the solution on them (with the specific tree type).
//!   Tests with `p > 30_000` will be skipped by immediately returning 0 (they're a bit too slow).
//! * It will look for the correct answer in the file with the same name (input `file.in` corresponds to output
//!   in `file.out`), and compare it to the one computed by the algorithm.
//! * For every test case, it will print a line containing the file name, `p`,
//!   how long was the computation, and the answer.
//!
//! [here]: https://ioinformatics.org/page/ioi-2008/34

use grove::*;
use itertools::Itertools;

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Error, ErrorKind, Read};
use std::str::FromStr;
use std::time::Instant;

use example_data::{AddAction, SizedSummary};
use trees::avl::*;
use trees::splay::*;
use trees::treap::*;

type I = i32;

///////////////// obstacle and edge ///////////////////
// these are types that represent the obstacles and the obstacle's edges

/// One of the obstacles.
/// Represents the rectangle [xlow..xhigh) x [ylow..yhigh) with half-open
/// ranges, as should be.
#[derive(PartialEq, Eq, Clone, Debug)]
struct Obstacle {
    xlow: I,
    xhigh: I,
    ylow: I,
    yhigh: I,
    cost: I,
}

impl Obstacle {
    fn edges(&self) -> (Edge, Edge) {
        (
            Edge {
                is_opening: true,
                xlow: self.xlow,
                xhigh: self.xhigh,
                y: self.ylow,
                cost: self.cost,
            },
            Edge {
                is_opening: false,
                xlow: self.xlow,
                xhigh: self.xhigh,
                y: self.yhigh,
                cost: self.cost,
            },
        )
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct Edge {
    /// tells if this edge is an opening edge or a closing edge
    is_opening: bool,
    xlow: I,
    xhigh: I,
    y: I,
    cost: I,
}

impl Edge {
    // enlarges the obstacle `d` units to the negative directions.
    // might get out of bounds.
    fn enlarge(&self, d: I) -> Edge {
        Edge {
            xlow: self.xlow - d,
            y: if self.is_opening { self.y - d } else { self.y },
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
        self.y
            .cmp(&other.y)
            .then((!self.is_opening).cmp(&!other.is_opening))
            .then(self.xlow.cmp(&other.xlow))
            .then(self.xhigh.cmp(&other.xhigh))
            .then(self.cost.cmp(&other.cost))
    }
}

//////////////////////////////// implementing the tree data //////////////////////////////
// here we implement the data that will sit in the tree: the values (Segment), summaries (SizeMinSummary)
// and the actions (AddAction).

// a segment of constant values
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
struct Segment {
    size: usize,
    val: I,
}

impl Segment {
    fn split_at_index(self, i: usize) -> (Segment, Segment) {
        (
            Segment {
                size: i,
                val: self.val,
            },
            Segment {
                size: self.size - i,
                val: self.val,
            },
        )
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
struct SizeMinSummary {
    size: usize,
    min: Option<I>,
}

impl std::ops::Add for SizeMinSummary {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        SizeMinSummary {
            min: match (self.min, other.min) {
                (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
                (Some(a), _) => Some(a),
                (_, b) => b,
            },
            size: self.size + other.size,
        }
    }
}

impl std::default::Default for SizeMinSummary {
    fn default() -> Self {
        SizeMinSummary { size: 0, min: None }
    }
}

impl SizedSummary for SizeMinSummary {
    fn size(self) -> usize {
        self.size
    }
}

impl ToSummary<SizeMinSummary> for Segment {
    fn to_summary(&self) -> SizeMinSummary {
        SizeMinSummary {
            size: self.size,
            min: if self.size == 0 { None } else { Some(self.val) },
        }
    }
}

impl Acts<SizeMinSummary> for AddAction {
    fn act_inplace(&self, summary: &mut SizeMinSummary) {
        summary.min = summary.min.map(|max: I| max + self.add);
    }
}

impl Acts<Segment> for AddAction {
    fn act_inplace(&self, object: &mut Segment) {
        object.val += self.add;
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
struct MyData {}

impl Data for MyData {
    type Value = Segment;

    type Summary = SizeMinSummary;

    type Action = AddAction;
}

//////////////////////////////////////////// algorithm //////////////////////////////////////////
// here the actual algorithms start

// splits a segment inside the tree
fn search_split<TR: ModifiableTreeRef<MyData>>(tree: TR, index: usize) {
    let mut walker = tree.walker();
    // using an empty range so that we'll only end up at a node
    // if we actually need to split that node
    walker.search_subtree(index..index);

    let left = walker.left_summary().size;
    let v2option = walker.with_value(|val| {
        let (v1, v2) = val.split_at_index(index - left);
        *val = v1;
        v2
    });

    if let Some(v2) = v2option {
        walker.next_empty().unwrap(); // not at an empty position
        walker.insert(v2).unwrap();
    }
}

/// solves the pyramid_base problem
fn solve<T>(m: usize, n: usize, budget: I, obstacles: Vec<Obstacle>) -> usize
where
    T: SomeTree<MyData>,
    T: std::iter::FromIterator<Segment>,
    for<'b> &'b mut T: ModifiableTreeRef<MyData>,
    T: std::iter::FromIterator<Segment>,
{
    let (mut opening_edges, mut closing_edges): (Vec<Edge>, Vec<Edge>) =
        obstacles.iter().map(|x| x.edges()).unzip();

    opening_edges.sort_unstable();
    closing_edges.sort_unstable();

    // binary search the size of the pyramid, `k`
    let mut lo = 0;
    let mut hi = std::cmp::min(n, m);

    while hi > lo {
        let k = (lo + hi + 1) / 2;

        // this is the specific size, because squares with a large `x` coordinate
        // correspond after enlarging to pyramids that go out of the positive boundary
        let mut tree: T = vec![Segment {
            size: m + 1 - k,
            val: 0,
        }]
        .into_iter()
        .collect();

        // the list of updates we do to our tree as we scan it from a lower `y` coordinate
        // to a higher `y` coordinate
        let event_iter = opening_edges
            .iter()
            .map(|e| e.enlarge((k - 1) as I))
            .merge(closing_edges.iter().map(|e| e.enlarge((k - 1) as I)));

        // prev_y starts at `0` instead of `I::MIN`, in order that we don't consider any squares
        // with negative `y` coordinate, which are out of bounds.
        let mut prev_y = 0;
        let mut possible: bool = false;
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
                add: if edge.is_opening {
                    edge.cost
                } else {
                    -edge.cost
                },
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
            hi = k - 1;
        }
    }

    lo
}

///////////////////////////////////////////// input handling ////////////////////////////////////////////
// runs the solution on a given input
fn run_from<R: Read, T>(io: R) -> usize
where
    T: SomeTree<MyData>,
    T: std::iter::FromIterator<Segment>,
    for<'b> &'b mut T: ModifiableTreeRef<MyData>,
    T: std::iter::FromIterator<Segment>,
{
    let br = BufReader::new(io);

    let mut words_iter = br.lines().flat_map(|row| {
        row.unwrap()
            .split_whitespace()
            .map(|x: &str| x.parse().unwrap())
            .collect::<Vec<_>>()
    });

    let m = words_iter.next().unwrap();
    let n = words_iter.next().unwrap();
    let budget = words_iter.next().unwrap();
    let p = words_iter.next().unwrap();
    print!(" p={: <6}", p);

    // skip the last test case - this solution is too slow for it
    if p > 30_000 {
        return 0;
    }
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let mut obstacles = vec![];
    for _ in 0..p {
        let e = "format not met";
        let obstacle = Obstacle {
            xlow: words_iter.next().expect(e) - 1,
            ylow: words_iter.next().expect(e) - 1,
            xhigh: words_iter.next().expect(e),
            yhigh: words_iter.next().expect(e),
            cost: words_iter.next().expect(e),
        };
        obstacles.push(obstacle);
    }

    let start = Instant::now();
    let res = solve::<T>(m as usize, n as usize, budget, obstacles);
    let duration = Instant::now().duration_since(start);
    print!(" {: <12} ", format!("{:?}", duration));
    res
}

// run the solution of a specific file,
// and print some metadata
fn run_on_file<T>(name: &str) -> Result<(), Error>
where
    T: SomeTree<MyData> + std::iter::FromIterator<Segment>,
    for<'b> &'b mut T: ModifiableTreeRef<MyData>,
    T: std::iter::FromIterator<Segment>,
{
    let current_dir = std::path::PathBuf::from_str("../grove/pyramid_base_test_files").unwrap();
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
    let real_res: usize = content
        .trim()
        .parse()
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    if computed_res == real_res {
        println!("Ok!");
    } else {
        println!("Failed!");
    }

    Ok(())
}

// run the tests in the test directory.
// you need to manually put the tests in the folder.
fn check_all_tests<T>() -> Result<(), Error>
where
    T: SomeTree<MyData> + std::iter::FromIterator<Segment>,
    for<'b> &'b mut T: ModifiableTreeRef<MyData>,
    T: std::iter::FromIterator<Segment>,
{
    let current_dir = std::path::PathBuf::from_str("../grove/pyramid_base_test_files").unwrap();
    println!("Testing files from {:?}:", current_dir);

    // sort the filenames in order, by difficulty. specifically, ordering by the number first
    // and then by file name lexicographic ordering turns out to be the correct ordering.
    let filenames: Vec<String> = fs::read_dir(current_dir.clone())?
        .filter_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();

            let (filename, filetype) = path.file_name()?.to_str()?.split_once('.')?;

            if filetype == "in" {
                Some(String::from(filename))
            } else {
                None
            }
        })
        .sorted_by(|file1, file2| {
            let num1: i32 = file1.trim_matches(char::is_alphabetic).parse().unwrap();
            let num2: i32 = file2.trim_matches(char::is_alphabetic).parse().unwrap();
            num1.cmp(&num2).then(file1.cmp(file2))
        })
        .collect();

    let start = Instant::now();
    for filename in filenames {
        run_on_file(&filename)?;
    }
    let duration = Instant::now().duration_since(start);
    println!("done all files: {: <16} overall", format!("{:?}", duration));

    Ok(())
}

fn main() -> Result<(), Error> {
    println!("starting treap\n");
    check_all_tests::<Treap<_>>()?;
    println!("done treap\n");
    println!("starting splay\n");
    check_all_tests::<SplayTree<_>>()?;
    println!("done splay\n");
    println!("starting avl\n");
    check_all_tests::<AVLTree<_>>()?;
    println!("done avl\n");
    Ok(())
}
