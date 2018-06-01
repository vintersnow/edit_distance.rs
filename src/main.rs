extern crate getopts;
use std::io::*;
use std::str::*;
use std::{env, process};
use getopts::Options;

struct StrPair<'a> {
    s1: Vec<&'a str>,
    s2: Vec<&'a str>,
    dist_mat: Vec<Vec<u32>>,
    backpoint: Vec<u8>,
    insert_cost: u32,
    del_cost: u32,
    change_cost: u32
}

#[derive(Debug)]
struct Args {
  input: Vec<String>,
  insert: u32,
  delete: u32,
  change: u32,
}

impl<'a> StrPair<'a> {

    pub fn new(s1: &'a str, s2: &'a str) -> StrPair<'a> {
        // splitすると前後に空白文字が入る. "ab" => ["", "a", "b", ""]
        let s1: Vec<&str> = s1.split("").collect();
        let s2: Vec<&str> = s2.split("").collect();

        let mut dist = Vec::with_capacity(s1.len());
        for _ in 0..s1.len() {
            let mut v: Vec<u32> = vec![0; s2.len()];
            dist.push(v);
        }
        let bp = vec![0; s1.len() + s2.len() - 2];

        StrPair {
            s1: s1,
            s2: s2,
            dist_mat: dist,
            backpoint: bp,
            insert_cost: 1,
            del_cost: 1,
            change_cost: 2 // > 0
        }
    }


    fn edit_distance(&mut self) -> u32 {

        for i in 1..self.s1.len() {
            self.dist_mat[i][0] = 1000000 as u32;
            self.dist_mat[i][1] = (i - 1) as u32;
        }
        for i in 1..self.s2.len() {
            self.dist_mat[0][i] = 1000000 as u32;
            self.dist_mat[1][i] = (i - 1) as u32;
        }

        for i in 2..self.s1.len() {
            for j in 2..self.s2.len() {
                let insert = self.dist_mat[i-1][j] + self.insert_cost;
                let delete = self.dist_mat[i][j-1] + self.del_cost;
                let change = if self.s1[i-1] == self.s2[j-1] {
                    self.dist_mat[i-1][j-1]
                } else {
                    self.dist_mat[i-1][j-1] + self.change_cost
                };

                let dd = [insert, delete, change];
                let m = dd.iter().min().unwrap();
                self.dist_mat[i][j] = (*m).clone();
            }
        }

        return self.dist_mat[self.s1.len() - 1][self.s2.len() - 1];
    }

    fn backtrace(&mut self) {
        let mut i = self.s1.len() - 1;
        let mut j = self.s2.len() - 1;
        let mut idx = i + j - 1;

        let dist = &self.dist_mat;

        while i != 1 || j != 1 {
            // action: 0=insert, 1=delete, 2=change, 3=no change
            let ins = dist[i][j-1];
            let del = dist[i-1][j];
            let cha = dist[i-1][j-1];
            let arr = [ins, del, cha];
            let mut action = arr.iter().enumerate().min_by_key(|&(_, item)| item).unwrap().0;
            if action == 2 && dist[i-1][j-1] == dist[i][j] {
                action = 3;
            }
            self.backpoint[idx] = action as u8;
            match &self.backpoint[idx] {
                0 => j = j-1,
                1 => i = i-1,
                _ => {j=j-1; i=i-1}
            }
            idx -= 1;
        }

        let (_, bp) = self.backpoint.split_at(idx+1);
        let mut s1_idx = 0;
        let mut s2_idx = 0;
        // 3-unzipが出来たらいいのに...
        let ts: Vec<(&str, (&str, &str))> = bp.iter().map(|&i| match i {
                0 => {s2_idx+=1; ("+", (" ", self.s2[s2_idx].clone()))},
                1 => {s1_idx+=1; ("-", (self.s1[s1_idx].clone(), " "))},
                2 => {s1_idx+=1; s2_idx+=1; ("#", (self.s1[s1_idx].clone(), self.s2[s2_idx].clone()))},
                _ => {s1_idx+=1; s2_idx+=1; ("=", (self.s1[s1_idx].clone(), self.s2[s2_idx].clone()))},
        }).collect();

        let (code, strs): (Vec<_>, Vec<_>) = ts.iter().cloned().unzip();
        let (s1, s2): (Vec<_>, Vec<_>) = strs.iter().cloned().unzip();

        println!("{}", code.join(" "));
        println!("{}", s1.join(" "));
        println!("{}", s2.join(" "));
    }

    #[allow(dead_code)]
    fn print_mat(&self) {
        for v in &self.dist_mat {
            println!("{:?}", v);
        }
    }
}

fn print_usage(program: &str, opts: &Options) {
  let brief = format!("Usage: {} FILE [options]", program);
  print!("{}", opts.usage(&brief));
  process::exit(0);
}

fn parse_args() -> Args {
  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();

  let mut opts = Options::new();

  opts.optflag("h", "help", "print this help menu");
  opts.optopt("i", "insert", "insert cost", "NUMBER");
  opts.optopt("d", "delete", "delete cost", "NUMBER");
  opts.optopt("c", "change", "change cost", "NUMBER");

  let matches = opts.parse(&args[1..])
    .unwrap_or_else(|f| panic!(f.to_string()));

  if matches.opt_present("h") {
    print_usage(&program, &opts);
  }

  // if matches.free.is_empty() {
  //   print_usage(&program, &opts);
  // }

  Args {
    input: matches.free.clone(),
    insert: matches.opt_str("i").map_or(1, |s| s.parse().unwrap_or_else(|f: std::num::ParseIntError| panic!("Cannot parse to integer:\n {}", f.to_string()))),
    delete: matches.opt_str("d").map_or(1, |s| s.parse().unwrap_or_else(|f: std::num::ParseIntError| panic!("Cannot parse to integer:\n {}", f.to_string()))),
    change: matches.opt_str("c").map_or(2, |s| s.parse().unwrap_or_else(|f: std::num::ParseIntError| panic!("Cannot parse to integer:\n {}", f.to_string()))),
  }
}


fn read_vec<T: FromStr>(sin: &mut StdinLock) -> Option<Vec<T>> {
    let mut s = String::new();
    sin.read_line(&mut s).ok();
    let v: Vec<T> = s.trim().split_whitespace()
        .map(|e| e.parse::<T>().ok().unwrap()).collect();
    if v.len() != 0 {
        Some(v)
    } else {
        None
    }
}

fn main() {
    let args = parse_args();
    println!("{:?}", args);
    let s = stdin();
    let mut s = s.lock();
    let s = &mut s;
    while let Some(v) = read_vec::<String>(s) {
        if v.len() != 2 {
            eprintln!("Invalid format: {:?}", v);
            continue;
        }
        let mut sp = StrPair::new(&v[0], &v[1]);
        let dist = sp.edit_distance();
        println!("dist: {}", dist);
        // sp.print_mat();
        sp.backtrace();
    }
}

//
// #[derive(Debug)]
// struct Args {
//   input: Vec<String>,
//   output: Option<String>,
//   // ...
// }
//
// fn print_usage(program: &str, opts: &Options) {
//   let brief = format!("Usage: {} FILE [options]", program);
//   print!("{}", opts.usage(&brief));
//   process::exit(0);
// }
//
// fn parse_args() -> Args {
//   let args: Vec<String> = env::args().collect();
//   let program = args[0].clone();
//
//   let mut opts = Options::new();
//   opts.optopt("o", "", "set output file name", "NAME");
//   opts.optflag("h", "help", "print this help menu");
//   // ...
//
//   let matches = opts.parse(&args[1..])
//     .unwrap_or_else(|f| panic!(f.to_string()));
//
//   if matches.opt_present("h") {
//     print_usage(&program, &opts);
//   }
//
//   // if matches.free.is_empty() {
//   //   println!("kore?");
//   //   print_usage(&program, &opts);
//   // }
//
//   Args {
//     input: matches.free.clone(),
//     output: matches.opt_str("o"),
//     // ...
//   }
// }
//
// fn main() {
//   let args = parse_args();
//   println!("{:?}", args);
// }
